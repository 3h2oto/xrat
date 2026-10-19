use crate::components::{loading::LoadingRegion, log_console::skeleton::LogConsoleSkeleton};
use dioxus::prelude::*;

#[component]
pub(super) fn Suspend() -> Element {
    rsx! {
        LoadingRegion { label: "Loading engine output…", LogConsoleSkeleton {} }
    }
}
