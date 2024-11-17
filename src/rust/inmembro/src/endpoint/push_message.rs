use super::*;

#[derive(Deserialize)]
pub struct PushParams {
    message: String,
}

pub async fn push_message_by_get(
    state: Extension<SharedState>,
    params: Query<PushParams>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let PushParams { message: s } = params.0;
    match serde_json::from_str(&s) {
        Err(err) => {
            format!("failed to push to topic'{name}' message'{s}' due to error: {err}")
        }
        Ok(message) => push_message(state, name, message).await,
    }
}

pub async fn push_message_by_post(
    state: Extension<SharedState>,
    Path(name): Path<String>,
    Json(message): Json<Message>,
) -> impl IntoResponse {
    push_message(state, name, message).await
}

async fn push_message(state: Extension<SharedState>, name: String, message: Message) -> String {
    let mut ret = format!(
        "message {} was pushed to ",
        serde_json::to_string_pretty(&message).unwrap()
    );
    let topic = format!("topic'{name}'");
    let message = QueuedMessage::new(message);
    let is_newly_created = match state.write().await.topics.entry(name.clone()) {
        Entry::Occupied(e) => {
            let topic = &mut *e.get().write().unwrap();
            match &mut topic.content {
                TopicContent::Messages(messages) => {
                    messages.push_back(message, Some(&topic.config));
                }
                TopicContent::Subscribers(subscribers) => {
                    for subscriber in subscribers.iter() {
                        if let Some(subscriber) = subscriber.upgrade().as_ref() {
                            subscriber.push_back(message.clone());
                        }
                    }
                }
            }
            false
        }
        Entry::Vacant(e) => {
            e.insert(Arc::new(std::sync::RwLock::new(Topic {
                content: TopicContent::Messages(Messages::new(message)),
                ..Default::default()
            })));
            true
        }
    };
    if is_newly_created {
        ret.push_str("newly created ");
    }
    ret.push_str(&topic);
    ret
}
