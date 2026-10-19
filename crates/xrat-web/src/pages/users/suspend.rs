use crate::components::loading::LoadingRegion;
use crate::ui::skeleton::Skeleton;
use dioxus::prelude::*;

#[component]
pub(super) fn Suspend() -> Element {
    rsx! {
        LoadingRegion { label: "Loading accounts…",
            div { class: "overflow-x-auto",
                div { class: "min-w-120",
                    div { class: "grid grid-cols-[minmax(150px,1fr)_100px_120px_54px] gap-4 border-b border-border px-4.5 py-3",
                        for column in 0..4 { Skeleton { key: "{column}", class: "h-3 w-3/4" } }
                    }
                    for user in 0..6 {
                        div { key: "{user}", class: "grid grid-cols-[minmax(150px,1fr)_100px_120px_54px] items-center gap-4 border-b border-border/40 px-4.5 py-3",
                            div { class: "flex items-center gap-2.5",
                                Skeleton { class: "size-8 rounded-full shrink-0" }
                                div { class: "grid gap-2 flex-1", Skeleton { class: "h-3.5 w-24" } Skeleton { class: "h-2.5 w-32" } }
                            }
                            Skeleton { class: "h-5 w-16 rounded-full" }
                            div { class: "flex gap-2 items-center", Skeleton { class: "h-5 w-9 rounded-full" } Skeleton { class: "h-3 w-12" } }
                            Skeleton { class: "size-8 justify-self-end" }
                        }
                    }
                }
            }
        }
    }
}
