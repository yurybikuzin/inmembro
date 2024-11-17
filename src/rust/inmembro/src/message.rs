use super::*;

#[derive(Serialize)]
pub struct QueuedMessage {
    content: Message,

    #[serde(rename = "lives_for")]
    #[serde(serialize_with = "serialize_instant")]
    /// для поддержки Retention
    created_at: std::time::Instant,
}

// https://stackoverflow.com/questions/75549538/add-an-additional-field-while-serializing
use serde::Serializer;
fn serialize_instant<S: Serializer>(instant: &std::time::Instant, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&arrange_millis::get(instant.elapsed().as_millis()))
}

impl QueuedMessage {
    pub fn new(content: Message) -> Arc<Self> {
        Arc::new(Self {
            content,
            created_at: std::time::Instant::now(),
        })
    }
    pub fn content(&self) -> &'_ Message {
        &self.content
    }
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    data: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    /// для поддержки Compaction
    key: Option<String>,
}

#[derive(Default, Serialize)]
pub struct Messages(VecDeque<Arc<QueuedMessage>>);

impl Messages {
    pub fn new(message: Arc<QueuedMessage>) -> Self {
        Self([message].into_iter().collect())
    }
    pub fn push_back(&mut self, message: Arc<QueuedMessage>, config: Option<&Arc<TopicConfig>>) {
        if let Some(compaction_key) = message.content.key.as_deref() {
            if config
                .as_ref()
                .and_then(|config| config.compaction)
                .unwrap_or(false)
            {
                self.0.retain(|i| {
                    i.content
                        .key
                        .as_deref()
                        .map(|key| key != compaction_key)
                        .unwrap_or(true)
                });
            }
        }
        self.0.push_back(message);
    }
    pub fn pop_front(&mut self, config: Option<&Arc<TopicConfig>>) -> Option<Arc<QueuedMessage>> {
        loop {
            let ret = self.0.pop_front();
            match ret {
                None => {
                    break None;
                }
                Some(queued_message) => {
                    let is_found = if let Some(retention_millis) =
                        config.as_ref().and_then(|config| config.retention_millis)
                    {
                        queued_message.created_at.elapsed().as_millis() as u64 <= retention_millis
                    } else {
                        true
                    };
                    if is_found {
                        break Some(queued_message);
                    }
                }
            }
        }
    }
}
