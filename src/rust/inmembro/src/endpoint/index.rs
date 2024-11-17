use super::*;

pub async fn index(state: Extension<SharedState>) -> impl IntoResponse {
    page::index(&*state.read().await)
}
