use super::*;

#[component]
pub(crate) fn CategoryIcon(category: String) -> Element {
    match category.as_str() {
        "runtime" => rsx! {
            Cpu {}
        },
        "dns" => rsx! {
            Globe {}
        },
        "routing" => rsx! {
            Route {}
        },
        "testing" => rsx! {
            FlaskConical {}
        },
        "subscriptions" => rsx! {
            Rss {}
        },
        "server" => rsx! {
            Server {}
        },
        "parser" => rsx! {
            FileCode {}
        },
        _ => rsx! {
            Terminal {}
        },
    }
}
