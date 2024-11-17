use super::*;

mod index;
pub use index::*;

fn head(title: Option<&str>, url_prefix: &str) -> Markup {
    let title = title.unwrap_or("InMeMBro");
    html! {
        (DOCTYPE)
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1, minimum-scale=1, user-scalable=no, viewport-fit=auto";
            link rel="stylesheet"
                href=(format!("{url_prefix}/assets/style.css"))
                type="text/css"
            ;
            link rel="icon" href=(format!("{url_prefix}/assets/favicon.svg"));
            title { (title) }
        }
    }
}
