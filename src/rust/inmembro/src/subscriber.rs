use super::*;

#[derive(Clone, Serialize)]
pub struct Subscriber {
    // для возможности идентификации подписчиков,
    // что необходимо для реализации автоматической отписки от сообщений
    id: uuid::Uuid,

    /// собственная очередь сообщений подписчика,
    /// поскольку у него может быть собственная скорость потребления сообщений
    messages: Arc<std::sync::RwLock<Messages>>,

    #[serde(skip_serializing)]
    topic_config: Weak<TopicConfig>,
    #[serde(skip_serializing)]
    topic: Weak<std::sync::RwLock<Topic>>,
}

impl Subscriber {
    pub fn id(&self) -> uuid::Uuid {
        self.id
    }
    pub fn push_back(&self, message: Arc<QueuedMessage>) {
        self.messages
            .write()
            .unwrap()
            .push_back(message, self.topic_config.upgrade().as_ref());
    }
    pub fn set_messages(&self, messages: Messages) {
        *self.messages.write().unwrap() = messages;
    }
    pub fn take_messages(&self) -> Messages {
        let mut ret = Messages::default();
        std::mem::swap(&mut ret, &mut self.messages.write().unwrap());
        ret
    }
}

use axum::response::sse::Event;
use futures::stream::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct EventStream(Arc<Subscriber>);

impl EventStream {
    pub fn new(topic: &Arc<std::sync::RwLock<Topic>>) -> Self {
        let topic_config = Arc::downgrade(&topic.read().unwrap().config);
        let topic = Arc::downgrade(topic);
        Self(Arc::new(Subscriber {
            topic,
            topic_config,
            messages: Arc::default(),
            id: uuid::Uuid::new_v4(),
        }))
    }
    pub fn subscriber(&self) -> Weak<Subscriber> {
        Arc::downgrade(&self.0)
    }
}

impl Drop for EventStream {
    fn drop(&mut self) {
        debug!("EventStream::drop: {}", self.0.id());
        if let Some(topic) = self.0.topic.upgrade() {
            debug!("EventStream::drop: will remove_subscriber");
            topic.write().unwrap().remove_subscriber(self.subscriber());
        }
    }
}

impl Stream for EventStream {
    type Item = std::result::Result<Event, std::convert::Infallible>;
    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self
            .as_ref()
            .0
            .messages
            .write()
            .unwrap()
            .pop_front(self.0.topic_config.upgrade().as_ref())
        {
            None => Poll::Pending,
            Some(message) => Poll::Ready(Some(Ok(
                Event::default().data(serde_json::to_string_pretty(message.content()).unwrap())
            ))),
        }
    }
}
