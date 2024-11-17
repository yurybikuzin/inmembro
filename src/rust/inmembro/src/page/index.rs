use super::*;

pub fn index(app_state: &AppState) -> Markup {
    html! {
        (head(None, &app_state.url_prefix))
        body.report {
            @let topics = {
                let mut topics = app_state.topics.iter().collect::<Vec<_>>();
                topics.sort_by_key(|i| i.0);
                topics
            };
            pre {
                (serde_yml::to_string(&topics).unwrap())
            }
        }
    }
}
