use super::details::{BooleanFact, Fact};
use crate::{
    api,
    ui::{
        button::ButtonVariant,
        dialog::{Dialog, DialogContent, DialogDescription, DialogTitle},
        icon_button::IconButton,
        states::LoadingState,
        tooltip::{Tooltip, TooltipContent},
    },
};
use dioxus::prelude::*;
use icons::{CircleCheck, CircleHelp, CircleX, RefreshCw, Server};
use xrat_web::types::{Overview, RuntimeDetails};

#[component]
pub fn RuntimeDialog(data: Overview, stale: bool, on_close: EventHandler<MouseEvent>) -> Element {
    let mut details = use_resource(move || api::get::<RuntimeDetails>("overview/runtime"));
    rsx! {
        Dialog {
            DialogContent {
                class: "dashboard-details-dialog dashboard-runtime-dialog",
                initial_open: true,
                on_close,
                div { class: "dashboard-details-heading",
                    Server {}
                    div {
                        DialogTitle { "Runtime" }
                        DialogDescription { "Proxy listeners, engines, and background activity." }
                    }
                    IconButton {
                        label: "Refresh runtime details",
                        variant: ButtonVariant::Ghost,
                        disabled: details.state() == UseResourceState::Pending,
                        onclick: move |_| details.restart(),
                        RefreshCw {}
                    }
                }
                if stale {
                    p { class: "dashboard-runtime-stale", role: "status",
                        "Dashboard updates are paused."
                    }
                }
                match details() {
                    None => rsx! {
                        LoadingState { label: "Checking runtime…" }
                    },
                    Some(Err(message)) => rsx! {
                        p { class: "dashboard-warning", role: "alert", "{message}" }
                    },
                    Some(Ok(runtime)) => rsx! {
                        RuntimeFacts { runtime, data }
                    },
                }
            }
        }
    }
}

#[component]
fn RuntimeFacts(runtime: RuntimeDetails, data: Overview) -> Element {
    let total = data.configs.iter().filter(|row| !row.deleted).count();
    let enabled = data
        .configs
        .iter()
        .filter(|row| row.enabled && !row.deleted)
        .count();
    let deleted = data.configs.iter().filter(|row| row.deleted).count();
    let failed = super::stats::config_distribution(&data.configs)[3];
    let active = data
        .active_config
        .and_then(|id| data.configs.iter().find(|row| row.id == id))
        .map(|config| format!("#{} {}", config.id, config.name))
        .unwrap_or_else(|| "None".into());
    let rotation = match runtime.daemon_running {
        Some(false) => "Inactive · daemon not running".into(),
        None => "Unavailable".into(),
        Some(true) if runtime.rotation_enabled => {
            format!("On · every {}", duration(runtime.rotation_interval_secs))
        }
        Some(true) => "Off".into(),
    };
    rsx! {
        div { class: "runtime-session-summary",
            span { class: "runtime-session-status",
                span { class: if runtime.process_id.is_some() { "connection-dot is-connected" } else { "connection-dot" } }
                "{runtime.status}"
            }
            strong { "{active}" }
            code { "{runtime.engine}" }
        }
        dl { class: "dashboard-details-grid",
            Fact {
                label: "Process ID",
                value: runtime.process_id.map(|id| id.to_string()).unwrap_or_else(|| "Not running".into()),
            }
            Fact {
                label: "Started",
                value: runtime
                    .started_at
                    .as_deref()
                    .map(super::view::display_date)
                    .unwrap_or_else(|| {
                        if runtime.process_id.is_some() {
                            "Not recorded".into()
                        } else {
                            "Not running".into()
                        }
                    }),
            }
            Fact {
                label: "WebUI tasks",
                value: if runtime.running_jobs == 0 && runtime.queued_jobs == 0 { "Idle".into() } else { format!("{} running · {} queued", runtime.running_jobs, runtime.queued_jobs) },
            }
            Fact {
                label: "Daemon",
                value: match runtime.daemon_running {
                    Some(true) => "Running",
                    Some(false) => "Not running",
                    None => "Unavailable",
                },
            }
            Fact { label: "Rotation", value: rotation }
            Fact {
                label: "Configurations",
                value: format!(
                    "{total} total · {enabled} enabled · {failed} failed · {deleted} deleted",
                ),
            }
            Fact {
                label: "Subscriptions",
                value: data.subscriptions.len().to_string(),
            }
            if let Some(meta) = data.dashboard {
                Fact {
                    label: "TUN capture",
                    value: if meta.tun_active { "Active" } else if meta.tun_enabled { "Enabled · inactive" } else { "Off" },
                }
                BooleanFact { label: "Traffic statistics", value: meta.stats_enabled }
            }
        }
        div { class: "runtime-services-grid",
            section { class: "dashboard-runtime-section",
                h4 { "Proxy listeners" }
                if runtime.inbounds.is_empty() {
                    p { class: "dashboard-description", "No proxy listeners for the current runtime." }
                } else {
                    ul { class: "dashboard-runtime-list",
                        for inbound in &runtime.inbounds {
                            li { key: "{inbound.name}",
                                span { "{inbound.name}" }
                                super::listener::ListenerLink { inbound: inbound.clone() }
                                HealthStatus {
                                    available: inbound.reachable,
                                    up: "Reachable",
                                    down: "Unreachable",
                                    unknown: "Not checked",
                                }
                            }
                        }
                    }
                }
            }
            section { class: "dashboard-runtime-section",
                h4 { "Installed engines" }
                ul { class: "dashboard-runtime-list",
                    for engine in &runtime.engines {
                        li { key: "{engine.name}",
                            span { "{engine.name}" }
                            code {
                                {
                                    engine
                                        .version
                                        .as_deref()
                                        .unwrap_or(
                                            if engine.available { "Version unknown" } else { "Not available" },
                                        )
                                }
                            }
                            HealthStatus {
                                available: Some(engine.available),
                                up: "Available",
                                down: "Unavailable",
                                unknown: "Unknown",
                            }
                        }
                    }
                }
            }
        }
        p { class: "dashboard-footnote",
            "Checked "
            {crate::ui::history::timestamp((runtime.sampled_at_ms / 1000.0) as i64)}
        }
    }
}

#[component]
fn HealthStatus(available: Option<bool>, up: String, down: String, unknown: String) -> Element {
    let text = match available {
        Some(true) => up,
        Some(false) => down,
        None => unknown,
    };
    rsx! {
        Tooltip { class: "runtime-health-tooltip", tabindex: "0",
            span {
                role: "img",
                aria_label: text.clone(),
                class: match available {
                    Some(true) => "runtime-health is-healthy",
                    Some(false) => "runtime-health is-unhealthy",
                    None => "runtime-health",
                },
                if available == Some(true) {
                    CircleCheck {}
                } else if available == Some(false) {
                    CircleX {}
                } else {
                    CircleHelp {}
                }
            }
            TooltipContent { "{text}" }
        }
    }
}

fn duration(seconds: u64) -> String {
    if seconds > 0 && seconds.is_multiple_of(3600) {
        format!("{}h", seconds / 3600)
    } else if seconds > 0 && seconds.is_multiple_of(60) {
        format!("{}m", seconds / 60)
    } else {
        format!("{seconds}s")
    }
}
