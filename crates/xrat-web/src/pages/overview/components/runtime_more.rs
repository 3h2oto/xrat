use super::details::Fact;
use crate::components::runtime_status::HealthStatus;
use dioxus::prelude::*;
use icons::ChevronDown;
use xrat_web::types::RuntimeDetails;

#[component]
pub fn RuntimeMore(runtime: RuntimeDetails, stats_enabled: Option<bool>, stale: bool) -> Element {
    rsx! {
        details { class: "runtime-more group border-t border-border/40 pt-2 text-[11px]",
            summary { class: "flex w-fit cursor-pointer list-none items-center gap-1.5 rounded-sm text-muted-foreground hover:text-foreground focus-visible:outline-2 focus-visible:outline-ring focus-visible:outline-offset-2 [&::-webkit-details-marker]:hidden",
                "More"
                ChevronDown { class: "size-3 transition-transform group-open:rotate-180" }
            }
            div { class: "grid gap-3 pt-3",
                dl { class: "m-0 grid grid-cols-2 gap-x-4 gap-y-2 [&_dt]:text-[10px] [&_dd]:text-[11px]",
                    Fact { label: "Runtime state", value: runtime.status.clone() }
                    Fact { label: "Process ID", value: runtime.process_id.map(|id| id.to_string()).unwrap_or_else(|| "Not running".into()) }
                    Fact { label: "Started", value: runtime.started_at.as_deref().map(display_date).unwrap_or_else(|| if runtime.process_id.is_some() { "Not recorded".into() } else { "Not running".into() }) }
                    Fact { label: "Traffic statistics", value: match stats_enabled { Some(true) => "Enabled", Some(false) => "Disabled", None => "Unavailable" } }
                }
                section { aria_label: "Installed engines",
                    h4 { class: "m-0 mb-2 text-[10px] font-medium text-muted-foreground", "Installed engines" }
                    ul { class: "m-0 grid list-none gap-2 p-0",
                        for engine in &runtime.engines {
                            li { key: "{engine.name}", class: "grid grid-cols-[72px_minmax(0,1fr)_auto] items-center gap-2",
                                span { "{engine.name}" }
                                code { class: "min-w-0 text-[10px] wrap-anywhere", {engine.version.as_deref().unwrap_or(if engine.available { "Version unknown" } else { "Not available" })} }
                                HealthStatus { available: Some(engine.available), label: engine.name.clone(), stale }
                            }
                        }
                        if runtime.engines.is_empty() { li { class: "text-muted-foreground", "No engines reported" } }
                    }
                }
                p { class: "m-0 text-[10px] text-muted-foreground",
                    if stale { "Updates paused · " }
                    "Checked "
                    {crate::components::history::timestamp((runtime.sampled_at_ms / 1000.0) as i64)}
                }
            }
        }
    }
}

pub(super) fn duration(seconds: u64) -> String {
    if seconds > 0 && seconds.is_multiple_of(3600) {
        format!("{}h", seconds / 3600)
    } else if seconds > 0 && seconds.is_multiple_of(60) {
        format!("{}m", seconds / 60)
    } else {
        format!("{seconds}s")
    }
}

pub fn display_date(raw: &str) -> String {
    raw.trim()
        .parse::<i64>()
        .ok()
        .map(|value| value as f64)
        .or_else(|| crate::components::history::event_time(raw))
        .filter(|value| value.is_finite())
        .map(|value| crate::components::history::timestamp(value as i64))
        .unwrap_or_else(|| "Time unavailable".into())
}
