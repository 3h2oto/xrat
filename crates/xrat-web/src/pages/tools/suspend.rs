use crate::components::loading::LoadingRegion;
use crate::ui::skeleton::Skeleton;
use dioxus::prelude::*;

#[component]
pub(super) fn Suspend(section: String, #[props(default = false)] starting: bool) -> Element {
    let lines = if section == "GeoIP lookup" { 6 } else { 10 };
    rsx! {
        LoadingRegion { label: if starting { "Starting tool job…" } else { "Waiting for tool results…" },
            div { class: "tool-result rounded-[7px] border border-border bg-muted/20 p-4 grid gap-3 min-h-52",
                for line in 0..lines {
                    div { key: "{line}", class: if section == "Parse configs" && line % 4 != 0 { "pl-5" } else { "" },
                        Skeleton { class: match line % 4 { 0 => "h-3 w-3/4", 1 => "h-3 w-full", 2 => "h-3 w-1/2", _ => "h-3 w-2/3" } }
                    }
                }
            }
        }
    }
}
