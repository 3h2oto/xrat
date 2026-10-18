use super::details::{BooleanFact, Fact};
use crate::{
    api,
    ui::{
        button::ButtonVariant,
        dialog::{Dialog, DialogContent, DialogDescription, DialogTitle},
        icon_button::IconButton,
        states::{LoadingLayout, LoadingState},
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
                class: concat!(
                    "dashboard-details-dialog dashboard-runtime-dialog w-[min(720px,_calc(100vw_-_32px))] ",
                    "max-w-[calc(100vw_-_32px)] max-h-[calc(100dvh_-_40px)] overflow-auto rounded-[12px] bg-card ",
                    "p-5 [&_>_.dashboard-footnote]:mt-3.5 [&_>_.dashboard-footnote]:text-[10px] ",
                    "[&_.dashboard-details-heading_>_.icon-action]:ml-auto ",
                    "[&_.dashboard-details-heading_>_.icon-action]:mt-[-6px] ",
                    "[&_.dashboard-details-grid]:grid-cols-[repeat(3,_minmax(0,_1fr))] ",
                    "[&_.dashboard-details-grid]:gap-y-3 [&_.dashboard-details-grid]:gap-x-4 ",
                    "[&_.dashboard-details-grid]:mt-3.5 ",
                    "[@media(width<=380px)]:[&_.dashboard-details-grid]:grid-cols-[minmax(0,_1fr)] ",
                    "[@media(width>380px)_and_(width<=640px)]:[&_.dashboard-details-grid]:grid-cols-[repeat(2,_minmax(0,_1fr))]",
                ),
                initial_open: true,
                on_close,
                div { class: "dashboard-details-heading flex items-start pr-7 gap-2.5 [&_>_svg]:size-4.5 [&_>_svg]:text-primary [&_>_svg]:shrink-[0] [&_h3]:text-[17px] [&_p]:mt-1.5 [&_p]:mb-0 [&_p]:text-[12px] [&_p]:text-muted-foreground [&_p]:mx-0",
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
                    p {
                        class: "dashboard-runtime-stale text-muted-foreground text-[11px] mt-3 mb-0 mx-0",
                        role: "status",
                        "Dashboard updates are paused."
                    }
                }
                match details() {
                    None => rsx! {
                        LoadingState { label: "Checking runtime…", layout: LoadingLayout::Summary }
                    },
                    Some(Err(message)) => rsx! {
                        p {
                            class: concat!(
                                "dashboard-warning flex items-center justify-between border border-solid ",
                                "border-[color:color-mix(in_srgb,_var(--destructive)_40%,_var(--border))] rounded-[8px] ",
                                "bg-[color:color-mix(in_srgb,_var(--destructive)_5%,_var(--panel))] py-3 px-4 gap-4 ",
                                "[@media(width<=600px)]:flex-wrap [&_p]:text-[12px] [&_p]:leading-[1.6] [&_p]:m-0",
                            ),
                            role: "alert",
                            "{message}"
                        }
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
        div {
            class: concat!(
                "runtime-session-summary flex items-center flex-wrap gap-y-2 gap-x-3 mt-4.5 pb-3.5 border-b ",
                "border-solid border-b-[color:color-mix(in_srgb,_var(--border)_30%,_transparent)] text-[12px] ",
                "[&_strong]:min-w-0 [&_strong]:wrap-anywhere [&_strong]:font-[550] [&_>_code]:ml-auto ",
                "[&_>_code]:text-muted-foreground [&_>_code]:text-[11px]",
            ),
            span { class: "runtime-session-status inline-flex items-center text-muted-foreground gap-[7px]",
                span { class: if runtime.process_id.is_some() { "connection-dot is-connected size-[7px] flex-none rounded-[50%] bg-[color:var(--chart-1)] shadow-[0_0_0_4px_color-mix(in_srgb,_var(--chart-1)_12%,_transparent)]" } else { "connection-dot size-[7px] flex-none rounded-[50%] bg-[color:var(--chart-1)] shadow-[0_0_0_4px_color-mix(in_srgb,_var(--chart-1)_12%,_transparent)]" } }
                "{runtime.status}"
            }
            strong { "{active}" }
            code { "{runtime.engine}" }
        }
        dl { class: "dashboard-details-grid grid grid-cols-[repeat(2,_minmax(0,_1fr))] gap-y-3.5 gap-x-6 mt-4.5 mb-0 mx-0 [@media(width<=540px)]:grid-cols-[minmax(0,_1fr)]",
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
        div {
            class: concat!(
                "runtime-services-grid grid grid-cols-[minmax(0,_1.3fr)_minmax(0,_0.8fr)] mt-4 gap-5 ",
                "[@media(width<=640px)]:grid-cols-[minmax(0,_1fr)] [@media(width<=640px)]:gap-3.5 ",
                "[&_.dashboard-runtime-section]:min-w-0 [&_.dashboard-runtime-section]:mt-0 ",
                "[&_.dashboard-runtime-section]:pt-3 [&_.dashboard-runtime-list]:gap-[5px] ",
                "[&_.dashboard-runtime-list_li]:grid-cols-[72px_minmax(0,_1fr)_14px] ",
                "[&_.dashboard-runtime-list_li]:text-[11px] [&_.dashboard-runtime-list_li]:min-h-8 ",
                "[&_.dashboard-runtime-list_li]:gap-1.5 [&_.runtime-health-tooltip]:justify-self-end ",
                "[&_.dashboard-runtime-list_li_>_span:first-child]:overflow-hidden ",
                "[&_.dashboard-runtime-list_li_>_span:first-child]:[text-overflow:ellipsis]",
            ),
            section { class: "dashboard-runtime-section mt-4.5 pt-3.5 border-t border-solid border-t-[color:color-mix(in_srgb,_var(--border)_35%,_transparent)] [&_h4]:text-[12px] [&_h4]:mt-0 [&_h4]:mb-2.5 [&_h4]:mx-0",
                h4 { "Proxy listeners" }
                ul {
                    class: concat!(
                        "dashboard-runtime-list list-none grid m-0 p-0 gap-2.5 [&_li]:grid ",
                        "[&_li]:grid-cols-[90px_minmax(0,_1fr)_auto] [&_li]:items-center [&_li]:text-[12px] ",
                        "[&_li]:gap-2.5 [@media(width<=540px)]:[&_li]:grid-cols-[72px_minmax(0,_1fr)] ",
                        "[&_code]:text-[11px] [&_code]:wrap-anywhere ",
                        "[@media(width<=540px)]:[&_.runtime-health]:[grid-column:2]",
                    ),
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
                    if !runtime.inbounds.iter().any(|inbound| inbound.name == "Shadowsocks") {
                        li {
                            span { "Shadowsocks" }
                            code { "—" }
                        }
                    }
                    li {
                        span { "PAC" }
                        if let Some(url) = super::listener::pac_url(runtime.pac_available) {
                            super::listener::CopyUrl { url }
                        } else {
                            code { "—" }
                        }
                    }
                }
            }
            section { class: "dashboard-runtime-section mt-4.5 pt-3.5 border-t border-solid border-t-[color:color-mix(in_srgb,_var(--border)_35%,_transparent)] [&_h4]:text-[12px] [&_h4]:mt-0 [&_h4]:mb-2.5 [&_h4]:mx-0",
                h4 { "Installed engines" }
                ul {
                    class: concat!(
                        "dashboard-runtime-list list-none grid m-0 p-0 gap-2.5 [&_li]:grid ",
                        "[&_li]:grid-cols-[90px_minmax(0,_1fr)_auto] [&_li]:items-center [&_li]:text-[12px] ",
                        "[&_li]:gap-2.5 [@media(width<=540px)]:[&_li]:grid-cols-[72px_minmax(0,_1fr)] ",
                        "[&_code]:text-[11px] [&_code]:wrap-anywhere ",
                        "[@media(width<=540px)]:[&_.runtime-health]:[grid-column:2]",
                    ),
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
        p { class: "dashboard-footnote text-muted-foreground text-[10px] leading-[1.5] m-0",
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
        Tooltip { class: "runtime-health-tooltip inline-flex", tabindex: "0",
            span {
                role: "img",
                aria_label: text.clone(),
                class: match available {
                    Some(true) => {
                        "runtime-health is-healthy inline-flex items-center text-[color:var(--chart-1)] text-[11px] gap-[5px] [&_svg]:size-[13px] [&_svg]:shrink-[0] [&.is-unhealthy]:text-destructive"
                    }
                    Some(false) => {
                        "runtime-health is-unhealthy inline-flex items-center text-[color:var(--chart-1)] text-[11px] gap-[5px] [&_svg]:size-[13px] [&_svg]:shrink-[0] [&.is-unhealthy]:text-destructive"
                    }
                    None => {
                        "runtime-health inline-flex items-center text-[color:var(--chart-1)] text-[11px] gap-[5px] [&_svg]:size-[13px] [&_svg]:shrink-[0] [&.is-unhealthy]:text-destructive"
                    }
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

pub(super) fn duration(seconds: u64) -> String {
    if seconds > 0 && seconds.is_multiple_of(3600) {
        format!("{}h", seconds / 3600)
    } else if seconds > 0 && seconds.is_multiple_of(60) {
        format!("{}m", seconds / 60)
    } else {
        format!("{seconds}s")
    }
}
