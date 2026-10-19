use crate::components::loading::LoadingRegion;
use crate::ui::skeleton::Skeleton;
use dioxus::prelude::*;

#[component]
pub(super) fn Suspend() -> Element {
    rsx! {
        LoadingRegion { label: "Getting your dashboard ready…",
            div { class: "grid gap-3 min-[1001px]:grid-rows-[repeat(2,minmax(18rem,1fr))]",
                div { class: "grid grid-cols-3 gap-3 [@media(width<=1000px)]:grid-cols-2 [@media(width<=600px)]:grid-cols-1",
                    div { class: "rounded-[10px] border border-border p-4 grid gap-4",
                        Skeleton { class: "h-4 w-32" }
                        div { class: "flex items-center gap-3", Skeleton { class: "size-10 rounded-full" } div { class: "grid gap-2 flex-1", Skeleton { class: "h-5 w-3/4" } Skeleton { class: "h-3 w-1/2" } } }
                        for index in 0..4 { div { key: "{index}", class: "flex justify-between gap-4", Skeleton { class: "h-3 w-20" } Skeleton { class: "h-3 w-24" } } }
                    }
                    div { class: "rounded-[10px] border border-border p-4 grid gap-4",
                        Skeleton { class: "h-4 w-36" }
                        div { class: "flex items-center justify-around gap-4", Skeleton { class: "size-28 rounded-full" } div { class: "grid gap-3", for index in 0..4 { Skeleton { key: "{index}", class: "h-3 w-20" } } } }
                        Skeleton { class: "h-3 w-full" }
                    }
                    div { class: "rounded-[10px] border border-border p-4 grid gap-4",
                        Skeleton { class: "h-4 w-32" }
                        div { class: "grid gap-4", for index in 0..3 { div { key: "{index}", class: "grid gap-2", Skeleton { class: "h-3 w-20" } Skeleton { class: if index == 1 { "h-5 w-3/4" } else { "h-5 w-full" } } } } }
                    }
                }
                div { class: "grid grid-cols-[minmax(0,1.2fr)_minmax(0,1fr)] gap-3 [@media(width<=1000px)]:grid-cols-1",
                    for chart in 0..2 { div { key: "{chart}", class: "rounded-[10px] border border-border p-4 grid gap-4",
                        Skeleton { class: "h-4 w-32" }
                        div { class: "grid grid-cols-3 gap-4", for metric in 0..3 { div { key: "{metric}", class: "grid gap-2", Skeleton { class: "h-3 w-3/4" } Skeleton { class: "h-5 w-full" } } } }
                        Skeleton { class: "h-44 w-full" }
                    } }
                }
            }
        }
    }
}
