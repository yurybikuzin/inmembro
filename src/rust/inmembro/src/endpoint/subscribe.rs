use super::*;

#[derive(Deserialize)]
pub struct SubscribeParams {
    /// Настройка для SSE
    keep_alive_secs: Option<u64>,

    /// Настройка для SSE
    keep_alive_text: Option<String>,

    /// Создавать ли топик при подписке на еще несуществующий топик?
    anyway: Option<bool>,
}

use axum::response::sse::Sse;
use std::time::Duration;
// https://github.com/tokio-rs/axum/blob/main/examples/sse/src/main.rs
pub async fn subscribe(
    state: Extension<SharedState>,
    params: Query<SubscribeParams>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let SubscribeParams {
        keep_alive_secs,
        keep_alive_text,
        anyway,
    } = params.0;

    let event_stream = {
        let topics = &mut state.write().await.topics;
        if let Some(topic) = topics.get(&name) {
            let event_stream = EventStream::new(topic);
            topic
                .write()
                .unwrap()
                .add_subscriber(event_stream.subscriber());

            Some(event_stream)
        } else if anyway.unwrap_or(false) {
            let topic = Arc::new(std::sync::RwLock::new(Topic::default()));
            let event_stream = EventStream::new(&topic);
            topic
                .write()
                .unwrap()
                .add_subscriber(event_stream.subscriber());

            topics.insert(name.clone(), topic);
            Some(event_stream)
        } else {
            None
        }
    };

    if let Some(event_stream) = event_stream {
        debug!(
            "did subscribe'{}' to topic'{name}'",
            event_stream
                .subscriber()
                .upgrade()
                .map(|subscriber| subscriber.id())
                .unwrap()
        );
        Sse::new(event_stream)
            .keep_alive(
                axum::response::sse::KeepAlive::new()
                    .interval(Duration::from_secs(
                        keep_alive_secs.unwrap_or_else(|| settings!(keep_alive_secs)),
                    ))
                    .text(keep_alive_text.unwrap_or_default()),
            )
            .into_response()
    } else {
        format!("failed to subscribe to non existent topic'{name}'").into_response()
    }
}
