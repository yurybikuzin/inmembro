use super::*;

pub async fn create_topic(
    state: Extension<SharedState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let app_state = &mut *state.write().await;
    match app_state.topics.entry(name.clone()) {
        Entry::Occupied(_) => format!("topic'{name}' already exists"),
        Entry::Vacant(e) => {
            e.insert(Arc::new(std::sync::RwLock::new(Topic::default())));
            format!("created topic'{name}'")
        }
    }
}
