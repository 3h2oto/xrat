use super::components::*;
use crate::Session;
use crate::components::states::EmptyState;
use dioxus::prelude::*;

#[component]
pub fn EngineLogs() -> Element {
    let Session(session) = use_context();
    if !session().is_some_and(|me| me.user.role.can_operate()) {
        return rsx! {
            EmptyState {
                title: "Operator access required",
                description: "Engine output is available to operators and administrators.",
            }
        };
    }
    rsx! {
        EngineOutput {}
    }
}
