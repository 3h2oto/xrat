use super::components::*;
use crate::Session;
use dioxus::prelude::*;

#[component]
pub fn Tools(section: String) -> Element {
    let Session(session) = use_context();
    if !session().is_some_and(|session| session.user.role.can_operate()) {
        return rsx! {
            p { class: "empty text-muted-foreground leading-[1.8] text-[14px] py-9 px-3",
                "Tools require an operator or administrator account."
            }
        };
    }
    rsx! {
        for tool in [section] {
            ToolEditor { key: "{tool}", section: tool }
        }
    }
}
