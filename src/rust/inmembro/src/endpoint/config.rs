use super::*;

pub async fn get_config(
    _state: Extension<SharedState>,
    _config: Query<TopicConfig>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    name
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct TopicConfig {
    retention_millis: Option<u64>,
    compaction: Option<bool>,
    commit: Option<bool>,
}

pub async fn set_config(
    _state: Extension<SharedState>,
    Path(name): Path<String>,
    Json(_config): Json<TopicConfig>,
) -> impl IntoResponse {
    name
}
