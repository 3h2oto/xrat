use crate::components::loading::LoadingRegion;
use crate::ui::skeleton::Skeleton;
use dioxus::prelude::*;

#[component]
pub(super) fn Suspend() -> Element {
    rsx! {
        LoadingRegion { label: "Loading settings…",
            div { class: "flex flex-wrap gap-2 border-b border-border pb-4 mb-5 [@media(width<=800px)]:hidden",
                for index in 0..6 { Skeleton { key: "{index}", class: "h-8 w-28" } }
            }
            div { class: "hidden [@media(width<=800px)]:grid gap-2 mb-5", Skeleton { class: "h-3 w-20" } Skeleton { class: "h-8 w-full" } }
            div { class: "grid gap-2 mb-5", Skeleton { class: "h-6 w-48" } Skeleton { class: "h-3 w-80 max-w-full" } }
            for group in 0..2 {
                div { key: "{group}", class: "rounded-[9px] border border-border mb-5",
                    div { class: "border-b border-border p-4", Skeleton { class: "h-4 w-40" } }
                    for row in 0..3 {
                        div { key: "{row}", class: "grid grid-cols-[minmax(0,1fr)_minmax(160px,0.6fr)] mobile:grid-cols-1 gap-4 border-b border-border/40 p-4",
                            div { class: "grid gap-2", Skeleton { class: "h-4 w-40" } Skeleton { class: "h-3 w-3/4" } }
                            Skeleton { class: if row == 1 { "h-5 w-10 rounded-full justify-self-end mobile:justify-self-start" } else { "h-8 w-full" } }
                        }
                    }
                }
            }
        }
    }
}
