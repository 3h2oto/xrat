use super::components::layout::ChartCard;
use super::components::{cards::*, charts::*, stats::*};
use crate::{
    Session, api,
    ui::button::{Button, ButtonVariant},
};
use dioxus::prelude::*;
use icons::{ArrowDown, Radio, RefreshCw};
use xrat_web::types::Overview;

#[component]
pub fn OverviewPage() -> Element {
    let Session(session) = use_context();
    let mut snapshot = use_signal(|| None::<Overview>);
    let mut error = use_signal(|| None::<String>);
    let mut history = use_signal(TrafficHistory::default);
    let mut last_seen = use_signal(|| Some(js_sys::Date::now()));
    let mut stale = use_signal(|| false);
    let mut poll = use_future(move || async move {
        loop {
            match api::get::<Overview>("overview?dashboard=true").await {
                Ok(data) if data.dashboard.is_some() => {
                    if let Some(meta) = &data.dashboard {
                        history.write().restore(meta, data.running);
                    }
                    snapshot.set(Some(data));
                    error.set(None);
                    last_seen.set(Some(js_sys::Date::now()));
                    stale.set(false);
                }
                Ok(_) => {
                    history.write().interrupt();
                    error.set(Some(
                        "The server needs to be restarted to load the new dashboard.".into(),
                    ));
                    stale.set(true);
                }
                Err(message) => {
                    history.write().interrupt();
                    error.set(Some(message));
                    stale.set(true);
                }
            }
            gloo_timers::future::TimeoutFuture::new(3000).await;
        }
    });
    use_future(move || async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(1000).await;
            if last_seen
                .peek()
                .is_some_and(|at| js_sys::Date::now() - at > 12_000.0)
                && !*stale.peek()
            {
                history.write().interrupt();
                stale.set(true);
            }
        }
    });
    let Some(data) = snapshot() else {
        let problem = error().or_else(|| {
            stale().then(|| {
                "The dashboard is taking longer than usual to load. Check the server or retry."
                    .into()
            })
        });
        return rsx! {
            if let Some(message) = problem {
                div {
                    class: concat!(
                        "dashboard-warning flex items-center justify-between border border-solid ",
                        "border-[color:color-mix(in_srgb,_var(--destructive)_40%,_var(--border))] rounded-[8px] ",
                        "bg-[color:color-mix(in_srgb,_var(--destructive)_5%,_var(--panel))] py-3 px-4 gap-4 ",
                        "[@media(width<=600px)]:flex-wrap [&_p]:text-[12px] [&_p]:leading-[1.6] [&_p]:m-0",
                    ),
                    role: "alert",
                    p { "{message}" }
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            last_seen.set(Some(js_sys::Date::now()));
                            stale.set(false);
                            error.set(None);
                            poll.restart();
                        },
                        RefreshCw {}
                        "Retry"
                    }
                }
            } else {
                super::suspend::Suspend {}
            }
        };
    };
    let Some(meta) = data.dashboard.clone() else {
        return rsx! {};
    };
    let me = session();
    let operator = me.as_ref().is_some_and(|me| me.user.role.can_operate());
    let traffic_message = if stale() {
        "Live data is unavailable. Previous samples are retained."
    } else if !data.running {
        "Connect a configuration to see live traffic."
    } else if !meta.stats_enabled {
        "Traffic statistics are disabled. Enable runtime stats in Settings."
    } else if data.traffic.is_none() {
        "Waiting for traffic statistics from the engine…"
    } else {
        "Collecting live samples. Rates need two consecutive readings."
    };
    let valid_rates = !stale() && data.running && meta.stats_enabled && data.traffic.is_some();
    let latest = history().latest().cloned().filter(|_| valid_rates);
    let traffic_series = traffic_series(&history().points);
    let probe_series = probe_series(&meta.probes, &meta.enabled_metrics);
    rsx! {
        div { class: concat!(
            "overview-dashboard grid min-w-0 tabular-nums gap-3 min-[1001px]:min-h-0 ",
            "min-[1001px]:h700:flex-1 min-[1001px]:grid min-[1001px]:grid-rows-[repeat(2,minmax(18rem,1fr))] min-[1001px]:[&:has(.dashboard-warning)]:grid-rows-[auto_repeat(2,minmax(18rem,1fr))] [&&]:[--chart-0:var(--primary)] [&&]:[--chart-1:#63857b] ",
            "[&&]:[--chart-2:#a88baf] [&&_[data-name=Card]]:min-w-0 ",
        ),
            if stale() {
                div {
                    class: concat!(
                        "dashboard-warning flex items-center justify-between border border-solid ",
                        "border-[color:color-mix(in_srgb,_var(--destructive)_40%,_var(--border))] rounded-[8px] ",
                        "bg-[color:color-mix(in_srgb,_var(--destructive)_5%,_var(--panel))] py-3 px-4 gap-4 ",
                        "[@media(width<=600px)]:flex-wrap [&_p]:text-[12px] [&_p]:leading-[1.6] [&_p]:m-0",
                    ),
                    role: "status",
                    p {
                        strong { "Updates paused. " }
                        {
                            error()
                                .unwrap_or_else(|| {
                                    "The server is taking longer than usual to respond.".into()
                                })
                        }
                        " Displayed data may be out of date."
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| poll.restart(),
                        RefreshCw {}
                        "Retry"
                    }
                }
            }
            div { class: "dashboard-summary-grid grid grid-cols-[repeat(3,_minmax(0,_1fr))] gap-3 min-[1001px]:min-h-0 [@media(width<=600px)]:grid-cols-[minmax(0,_1fr)] [@media(width>600px)_and_(width<=1000px)]:grid-cols-[repeat(2,_minmax(0,_1fr))]",
                super::components::runtime_card::RuntimeCard { data: data.clone(), stale: stale() }
                HealthCard {
                    configs: data.configs.clone(),
                    subscriptions: data.subscriptions.clone(),
                    refreshed: meta.last_subscription_refresh,
                    operator,
                    stale: stale(),
                }
                LatencyCard {
                    configs: data.configs.clone(),
                    enabled_metrics: meta.enabled_metrics.clone(),
                }
            }
            section {
                class: "dashboard-traffic min-[1001px]:h700:flex min-[1001px]:flex-col min-[1001px]:min-h-0 min-[1001px]:h700:flex-1",
                aria_label: "Live traffic and connection quality",
                div { class: "dashboard-chart-grid grid grid-cols-[minmax(0,_1.2fr)_minmax(0,_1fr)] gap-3 min-[1001px]:h700:min-h-0 min-[1001px]:h700:flex-1 [@media(width<=1000px)]:grid-cols-[minmax(0,_1fr)]",
                    ChartCard {
                        div { class: "dashboard-card-heading flex items-center justify-between flex-wrap gap-2.5 [&_>_span]:text-muted-foreground [&_>_span]:text-[10px]",
                            h3 {
                                ArrowDown {}
                                "Throughput"
                            }
                        }
                        dl {
                            class: concat!(
                                "dashboard-traffic-metrics grid grid-cols-[repeat(4,_minmax(0,_1fr))] min-h-8.5 text-center m-0 ",
                                "gap-2 [@media(width<=600px)]:grid-cols-[repeat(2,_minmax(0,_1fr))] ",
                                "[&_dt]:text-muted-foreground [&_dt]:text-[10px] [&_dt]:mb-[3px] [&_dd]:text-[13px] ",
                                "[&_dd]:font-[550] [&_dd]:tracking-[-0.3px] [&_dd]:m-0 [&_>_div]:min-w-0 [&_>_div]:flex ",
                                "[&_>_div]:flex-col [&_>_div]:items-center [&_>_div]:text-center",
                            ),
                            div {
                                dt { "Download" }
                                dd {
                                    {
                                        latest
                                            .as_ref()
                                            .and_then(|point| point.down)
                                            .map(|value| format!("{}/s", bytes(value)))
                                            .unwrap_or_else(|| "—".into())
                                    }
                                }
                            }
                            div {
                                dt { "Upload" }
                                dd {
                                    {
                                        latest
                                            .as_ref()
                                            .and_then(|point| point.up)
                                            .map(|value| format!("{}/s", bytes(value)))
                                            .unwrap_or_else(|| "—".into())
                                    }
                                }
                            }
                            div {
                                dt { "Downloaded" }
                                dd {
                                    {
                                        data.traffic
                                            .as_ref()
                                            .map(|traffic| bytes(traffic.downloaded_bytes as f64))
                                            .unwrap_or_else(|| "—".into())
                                    }
                                }
                            }
                            div {
                                dt { "Uploaded" }
                                dd {
                                    {
                                        data.traffic
                                            .as_ref()
                                            .map(|traffic| bytes(traffic.uploaded_bytes as f64))
                                            .unwrap_or_else(|| "—".into())
                                    }
                                }
                            }
                        }
                        PlotChart {
                            series: traffic_series,
                            area: true,
                            unit: "bytes",
                            empty: traffic_message,
                            chart_label: "Download and upload throughput",

                        }
                    }
                    ChartCard {
                        div { class: "dashboard-card-heading flex items-center justify-between flex-wrap gap-2.5 [&_>_span]:text-muted-foreground [&_>_span]:text-[10px]",
                            h3 {
                                Radio {}
                                "Probe history"
                            }
                        }
                        dl {
                            class: concat!(
                                "dashboard-probe-metrics grid grid-cols-[repeat(3,_minmax(0,_1fr))] min-h-8.5 content-start ",
                                "text-center m-0 gap-2 [&_dt]:text-muted-foreground [&_dt]:text-[10px] [&_dt]:mb-[3px] ",
                                "[&_dd]:text-[13px] [&_dd]:font-[550] [&_dd]:tracking-[-0.3px] [&_dd]:m-0 [&_>_div]:min-w-0 ",
                                "[&_>_div]:flex [&_>_div]:flex-col [&_>_div]:items-center [&_>_div]:text-center",
                            ),
                            for metric in &meta.enabled_metrics {
                                div { key: "{key(*metric)}",
                                    dt { {label(*metric)} }
                                    dd {
                                        {milliseconds(meta.probes.last().and_then(|point| probe_value(point, *metric)))}
                                    }
                                }
                            }
                        }
                        PlotChart {
                            series: probe_series,
                            area: false,
                            unit: "ms",
                            empty: if !data.running { "Connect a configuration to see its probe history." } else if meta.enabled_metrics.is_empty() { "Latency tests are disabled. Enable them in Settings." } else { "No successful latency measurements yet. Run a configuration test." },
                            chart_label: "Active configuration latency history",

                        }
                    }
                }
            }
        }
    }
}
