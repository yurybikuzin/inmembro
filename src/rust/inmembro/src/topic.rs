use super::*;

#[derive(Default, Serialize)]
pub struct Topic {
    pub content: TopicContent,
    pub config: Arc<TopicConfig>,
}

impl Topic {
    pub fn add_subscriber(&mut self, subscriber: Weak<Subscriber>) {
        let messages_for_subscirber = match &mut self.content {
            TopicContent::Subscribers(subscribers) => {
                subscribers.push(subscriber);
                None
            }
            TopicContent::Messages(ref mut messages) => {
                let mut ret = Messages::default();
                std::mem::swap(messages, &mut ret);
                Some((ret, subscriber))
            }
        };
        if let Some((messages, subscriber)) = messages_for_subscirber {
            if let Some(subscriber) = subscriber.upgrade().as_ref() {
                subscriber.set_messages(messages);
            }
            self.content = TopicContent::Subscribers([subscriber].into_iter().collect());
        };
    }
    pub fn remove_subscriber(&mut self, subscriber: Weak<Subscriber>) {
        if let Some(subscriber) = subscriber.upgrade() {
            let id = dbg!(subscriber.id());
            let messages = if let TopicContent::Subscribers(ref mut subscribers) = &mut self.content
            {
                subscribers.retain(|i| i.upgrade().is_some());
                match dbg!(subscribers.len()) {
                    0 => {
                        debug!("subscribers.is_empty()");
                        Some(Messages::default())
                    }
                    1 => {
                        if let Some(idx) = subscribers.iter().position(|i| {
                            i.upgrade()
                                .or_else(|| {
                                    warn!("failed to upgrade weak Subscriber");
                                    None
                                })
                                .map(|i| dbg!(i.id()) == id)
                                .unwrap_or(false)
                        }) {
                            debug!("found subscriber at pos'{idx}'");
                            let ret = subscribers
                                .remove(idx)
                                .upgrade()
                                .map(|subscriber| subscriber.take_messages());
                            dbg!(ret.is_some(), subscribers.len());
                            ret
                        } else {
                            debug!("subscriber not found");
                            None
                        }
                    }
                    .or_else(|| Some(Messages::default())),
                    _ => None,
                }
            } else {
                None
            };
            if let Some(messages) = messages {
                debug!(
                    "did set TopicContent::Messages({})",
                    serde_json::to_string_pretty(&messages).unwrap()
                );
                self.content = TopicContent::Messages(messages);
            }
        }
    }
}

#[derive(Serialize)]
pub enum TopicContent {
    /// нет ни одного подписчика, и сообщения копятся в одной очереди сообщений топика
    Messages(Messages),

    /// для каждого подписчика формируется своя очередь сообщений, поскольку
    /// у каждого подписчика может быть своя скорость потребления сообщений;
    /// но каждое новое поступившее сообщение помещается в очередь каждого из подписчиков
    Subscribers(Vec<Weak<Subscriber>>),
}

impl Default for TopicContent {
    fn default() -> Self {
        Self::Messages(Messages::default())
    }
}

#[derive(Default, Serialize)]
pub struct TopicConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// включен ли режим **retention** ?
    /// и если да (Some), то время жизни сообщения в миллисекундах
    pub retention_millis: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// включен ли режим **compaction** ?
    /// по умолчанию (None) считается отключенным
    pub compaction: Option<bool>,
}
