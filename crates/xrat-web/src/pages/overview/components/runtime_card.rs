use super::layout::SummaryCard;
use super::{
    controls::{DashboardAction, TunControl},
    details::Fact,
};
use crate::components::runtime_status::{DaemonStatus, HealthStatus, JobsStatus};
use crate::ui::{button::ButtonVariant, icon_button::IconButton};
use crate::{
    Session, api,
    components::states::{LoadingLayout, LoadingState},
};
use dioxus::prelude::*;
use icons::{RefreshCw, Server};
use xrat_web::types::{Action, Overview, RuntimeDetails};

#[component]
pub fn RuntimeCard(data: Overview, stale: bool) -> Element {
    let Session(session) = use_context();
    let me = session();
    let operator = me.as_ref().is_some_and(|me| me.user.role.can_operate());
    let admin = me.as_ref().is_some_and(|me| me.user.role.is_admin());
    let active_name = data
        .active_config
        .and_then(|id| data.configs.iter().find(|row| row.id == id))
        .map(|row| {
            if row.name.is_empty() {
                format!("Configuration #{}", row.id)
            } else {
                row.name.clone()
            }
        })
        .unwrap_or_else(|| "No active configuration".into());
    let status = if stale {
        "Status unavailable"
    } else if data.running {
        "Connected"
    } else {
        "Disconnected"
    };
    let ready = data.configs.iter().any(|row| row.enabled && !row.deleted);
    let mut details = use_resource(move || api::get::<RuntimeDetails>("overview/runtime"));
    use_future(move || async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(15_000).await;
            details.restart();
        }
    });
    rsx! {
        SummaryCard { class: "dashboard-runtime-card gap-3 min-[1001px]:h700:min-h-0 min-[1001px]:h700:overflow-y-auto",
            div { class: "dashboard-card-heading flex items-center justify-between flex-wrap gap-2.5 [&_>_span]:text-muted-foreground [&_>_span]:text-[10px]",
                h3 {
                    Server {}
                    "Runtime"
                }
                IconButton {
                    label: "Refresh runtime",
                    variant: ButtonVariant::Ghost,
                    disabled: details.state() == UseResourceState::Pending,
                    onclick: move |_| details.restart(),
                    RefreshCw {}
                }
            }
            div {
                class: concat!(
                    "runtime-card-connection grid grid-cols-[auto_minmax(0,_1fr)] items-center pt-0.5 pb-1 px-0 ",
                    "gap-3 [&_.dashboard-connection-name]:text-end ",
                    "[&_.dashboard-connection-name]:justify-self-stretch [&_.dashboard-connection-name]:max-w-full ",
                    "[&_.dashboard-connection-name]:min-w-0 [&_.dashboard-connection-name]:whitespace-nowrap ",
                    "[&_.dashboard-connection-name]:overflow-hidden ",
                    "[&_.dashboard-connection-name]:[text-overflow:ellipsis]",
                ),
                div { class: "dashboard-connection flex items-center flex-wrap text-muted-foreground text-[12px] m-0 gap-2 [&_strong]:text-foreground [&_strong]:font-medium [&_>_span]:wrap-anywhere",
                    span { class: if data.running && !stale { "connection-dot is-connected size-[7px] flex-none rounded-[50%] bg-[color:var(--chart-1)] shadow-[0_0_0_4px_color-mix(in_srgb,_var(--chart-1)_12%,_transparent)]" } else { "connection-dot size-[7px] flex-none rounded-[50%] bg-muted-foreground/50" } }
                    strong { "{status}" }
                    if data.dashboard.as_ref().is_some_and(|meta| meta.runtime_status == "degraded") {
                        span { class: "dashboard-degraded text-destructive",
                            "Some listeners are unreachable"
                        }
                    }
                }
                super::details::ConnectionDetailsLink { data: data.clone(), stale, name: active_name }
            }
            match details() {
                None => rsx! {
                    LoadingState { label: "Checking runtime…", layout: LoadingLayout::Summary }
                },
                Some(Err(_)) => rsx! {
                    p {
                        class: "dashboard-description text-[12px] leading-[1.7] text-muted-foreground m-0",
                        role: "status",
                        "Runtime information unavailable. Use refresh to retry."
                    }
                },
                Some(Ok(runtime)) => {
                    let version = runtime
                        .engines
                        .iter()
                        .find(|engine| engine.name == runtime.engine)
                        .and_then(|engine| engine.version.as_deref());
                    let engine = if runtime.engine.is_empty() {
                        "Not selected".into()
                    } else {
                        format!(
                            "{} · {}",
                            runtime.engine,
                            version.unwrap_or("Version unavailable"),
                        )
                    };
                    let rotation = match runtime.daemon_running {
                        Some(true) if runtime.rotation_enabled => {
                            format!(
                                "Every {}",
                                super::runtime_more::duration(runtime.rotation_interval_secs),
                            )
                        }
                        Some(true) => "Off".into(),
                        Some(false) => "Inactive".into(),
                        None => "Unavailable".into(),
                    };
                    rsx! {
                        dl {
                            class: concat!(
                                "dashboard-runtime-summary grid grid-cols-[repeat(4,_minmax(0,_1fr))] gap-y-3 gap-x-4 ",
                                "text-center m-0 [@media(width<=1200px)]:grid-cols-[repeat(2,_minmax(0,_1fr))] ",
                                "[&_.dashboard-detail-fact_dt]:text-muted-foreground [&_.dashboard-detail-fact_dt]:text-[11px] ",
                                "[&_.dashboard-detail-fact_dt]:mb-1 [&_.dashboard-detail-fact_dd]:text-[12px] ",
                                "[&_.dashboard-detail-fact_dd]:wrap-anywhere [&_.dashboard-detail-fact_dd]:m-0",
                            ),
                            Fact { label: "Engine", value: engine }
                            div {
                                dt { class: "mb-1 text-[11px] text-muted-foreground", "Daemon" }
                                dd { class: "m-0", DaemonStatus { running: runtime.daemon_running, stale } }
                            }
                            Fact { label: "Rotation", value: rotation }
                            div {
                                dt { class: "mb-1 text-[11px] text-muted-foreground", "Jobs" }
                                dd { class: "m-0", JobsStatus { running: runtime.running_jobs, queued: runtime.queued_jobs, stale } }
                            }
                        }
                        ul {
                            class: concat!(
                                "runtime-card-listeners grid list-none pt-1 pb-0 m-0 px-0 gap-1 [&_li]:flex [&_li]:items-center ",
                                "[&_li]:justify-between [&_li]:flex-wrap [&_li]:gap-y-0.5 [&_li]:gap-x-2 [&_li]:min-w-0 ",
                                "[&_li_>_span]:text-muted-foreground [&_li_>_span]:text-[11px] ",
                                "[&_.runtime-listener-button]:min-w-0 [&_.runtime-listener-button]:max-w-full ",
                                "[&_.runtime-listener-button]:h-auto [&_.runtime-listener-button]:text-start ",
                                "[&_.runtime-listener-button]:whitespace-normal [&_.runtime-listener-button]:py-[3px] ",
                                "[&_.runtime-listener-button]:px-0 [&_code]:wrap-anywhere [&_code]:text-[10px]",
                            ),
                            aria_label: "Proxy listeners and PAC",
                            for inbound in &runtime.inbounds {
                                li { key: "{inbound.name}",
                                    span { "{inbound.name}" }
                                    div { class: "flex min-w-0 items-center gap-2",
                                        super::listener::ListenerLink { inbound: inbound.clone() }
                                        if inbound.reachable != Some(true) {
                                            HealthStatus { available: inbound.reachable, label: format!("{} listener", inbound.name), stale }
                                        }
                                    }
                                }
                            }
                            if !runtime.inbounds.iter().any(|inbound| inbound.name == "Shadowsocks") {
                                li {
                                    span { "Shadowsocks" }
                                    span { "—" }
                                }
                            }
                            li {
                                span { "PAC" }
                                if let Some(url) = super::listener::pac_url(runtime.pac_available) {
                                    super::listener::CopyUrl { url }
                                } else {
                                    span { "—" }
                                }
                            }
                        }
                        super::runtime_more::RuntimeMore {
                            runtime,
                            stats_enabled: data.dashboard.as_ref().map(|meta| meta.stats_enabled),
                            stale,
                        }
                    }
                }
            }
            div { class: "runtime-card-actions flex items-center justify-between flex-wrap mt-auto pt-1 gap-2",
                if operator {
                    DashboardAction {
                        action: Action::Rotate,
                        text: "Rotate now",
                        disabled: !ready || stale,
                        reason: if !ready { "Enable a configuration to rotate." } else { "Choose the next connection." },
                    }
                }
                if let Some(meta) = &data.dashboard {
                    TunControl {
                        enabled: meta.tun_enabled,
                        active: meta.tun_active,
                        available: meta.tun_available && !stale,
                        admin,
                        reason: meta.tun_reason.clone(),
                    }
                }
            }
        }
    }
}
