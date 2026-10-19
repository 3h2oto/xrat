use crate::ui::skeleton::Skeleton;
use dioxus::prelude::*;

#[component]
pub fn LogConsoleSkeleton() -> Element {
    rsx! {
        div { class: "log-viewer grid gap-3.5",
            div { class: "flex items-center justify-between gap-4",
                Skeleton { class: "h-3 w-36" }
                div { class: "flex gap-2", for index in 0..5 { Skeleton { key: "{index}", class: "size-8" } } }
            }
            div { class: "flex flex-wrap gap-3",
                Skeleton { class: "h-8 w-36" }
                Skeleton { class: "h-8 w-28" }
                Skeleton { class: "h-8 w-28" }
                Skeleton { class: "h-8 w-52 max-w-full" }
            }
            div { class: "rounded-[7px] border border-[#303030] bg-[#0c0c0c] p-3 min-h-80",
                div { class: "flex justify-between border-b border-[#303030] pb-3 mb-3",
                    Skeleton { class: "h-3 w-16" }
                    Skeleton { class: "h-3 w-32" }
                }
                for row in 0..9 {
                    div { key: "{row}", class: "grid grid-cols-[140px_58px_100px_minmax(0,1fr)] mobile:grid-cols-[80px_40px_minmax(0,1fr)] gap-3 py-2",
                        Skeleton { class: "h-3 w-full" }
                        Skeleton { class: "h-3 w-full" }
                        Skeleton { class: "h-3 w-full mobile:hidden" }
                        Skeleton { class: if row % 3 == 0 { "h-3 w-2/3" } else { "h-3 w-full" } }
                    }
                }
            }
            Skeleton { class: "h-3 w-64 max-w-full" }
        }
    }
}
