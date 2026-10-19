use crate::components::loading::LoadingRegion;
use crate::ui::skeleton::Skeleton;
use dioxus::prelude::*;

#[component]
pub(super) fn Suspend() -> Element {
    rsx! {
        LoadingRegion { label: "Loading your API tokens…",
            div { class: "self-token-list",
                for token in 0..4 {
                    div { key: "{token}", class: "flex items-center gap-2.5 border-t border-border/30 py-3.5",
                        Skeleton { class: "size-7.5 rounded-[6px] shrink-0" }
                        div { class: "flex-1 min-w-0 grid gap-2",
                            Skeleton { class: "h-3 w-36 max-w-full" }
                            Skeleton { class: "h-2.5 w-56 max-w-full" }
                        }
                        Skeleton { class: "size-8 shrink-0" }
                    }
                }
            }
        }
    }
}
