use crate::components::loading::{LoadingRegion, TableSkeleton};
use crate::ui::skeleton::Skeleton;
use dioxus::prelude::*;

#[component]
pub(super) fn Suspend() -> Element {
    rsx! {
        LoadingRegion { label: "Loading background jobs…",
            div { class: "flex flex-wrap gap-3 mb-3.5", for index in 0..3 { Skeleton { key: "{index}", class: "h-8 w-36" } } }
            TableSkeleton { columns: "grid-cols-[60px_150px_175px_90px_minmax(0,1fr)_64px]", cells: 6 }
        }
    }
}
