use crate::components::loading::{LoadingRegion, TableSkeleton};
use crate::components::log_console::skeleton::LogConsoleSkeleton;
use crate::ui::skeleton::Skeleton;
use dioxus::prelude::*;

#[component]
pub(super) fn Suspend(section: String) -> Element {
    rsx! {
        LoadingRegion { label: format!("Loading {}…", section.to_lowercase()),
            if section == "Logs" {
                div { class: "log-page-panel rounded-[9px] border border-border p-5 mobile:p-3.5", LogConsoleSkeleton {} }
            } else {
                div { class: "data-table-panel rounded-[9px] border border-border p-5 mobile:p-3.5",
                    div { class: "flex justify-between gap-4 mb-5",
                        div { class: "grid gap-2", Skeleton { class: "h-5 w-36" } Skeleton { class: "h-3 w-64 max-w-full" } }
                        Skeleton { class: "h-8 w-28" }
                    }
                    div { class: "flex flex-wrap gap-3 mb-4", Skeleton { class: "h-8 w-64 max-w-full" } for index in 0..3 { Skeleton { key: "{index}", class: "h-8 w-28" } } }
                    if section == "Configs" {
                        TableSkeleton { columns: "grid-cols-[24px_40px_minmax(160px,1fr)_90px_100px_100px_100px]", cells: 7 }
                    } else {
                        TableSkeleton { columns: "grid-cols-[minmax(160px,1fr)_minmax(180px,1.5fr)_90px_140px_100px]", cells: 5 }
                    }
                    div { class: "flex justify-between mt-4", Skeleton { class: "h-3 w-32" } Skeleton { class: "h-8 w-40" } }
                }
            }
        }
    }
}
