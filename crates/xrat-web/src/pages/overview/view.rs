use super::{cards::*, charts::*, controls::*, stats::*};
use crate::{
    Session, api,
    ui::{
        button::{Button, ButtonVariant},
        card::Card,
        states::LoadingState,
    },
};
use dioxus::prelude::*;
use icons::{Activity, ArrowDown, Radio, RefreshCw};
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
                div { class: "dashboard-warning", role: "alert",
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
                LoadingState { label: "Getting your dashboard ready…" }
            }
        };
    };
    let Some(meta) = data.dashboard.clone() else {
        return rsx! {};
    };
    let me = session();
    let username = me
        .as_ref()
        .map(|me| me.user.username.clone())
        .unwrap_or_default();
    let operator = me.as_ref().is_some_and(|me| me.user.role.can_operate());
    let admin = me.as_ref().is_some_and(|me| me.user.role.is_admin());
    let active = data
        .active_config
        .and_then(|id| data.configs.iter().find(|row| row.id == id));
    let active_name = active
        .map(|row| {
            if row.name.is_empty() {
                format!("Configuration #{}", row.id)
            } else {
                row.name.clone()
            }
        })
        .unwrap_or_else(|| "No active configuration".into());
    let ready = data.configs.iter().any(|row| row.enabled && !row.deleted);
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
        div { class: "overview-dashboard",
            if stale() {
                div { class: "dashboard-warning", role: "status",
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
            Card { class: "dashboard-welcome",
                div { class: "dashboard-welcome-top",
                    h2 { "Hey {username}, ready for some tunneling?" }
                    LiveClock {}
                }
                div { class: "dashboard-welcome-bottom",
                    div { class: "dashboard-connection",
                        span { class: if data.running { "connection-dot is-connected" } else { "connection-dot" } }
                        strong {
                            if stale() {
                                "Status unavailable"
                            } else if data.running {
                                "Connected"
                            } else {
                                "Disconnected"
                            }
                        }
                        span { "·" }
                        super::details::ConnectionDetailsLink {
                            data: data.clone(),
                            stale: stale(),
                            name: active_name,
                        }
                        if meta.runtime_status == "degraded" {
                            span { class: "dashboard-degraded", "Some inbound listeners are unreachable" }
                        }
                    }
                    div { class: "dashboard-quick-actions",
                        if operator {
                            DashboardAction {
                                action: xrat_web::types::Action::Rotate,
                                text: "Rotate now",
                                disabled: !ready || stale(),
                                reason: if !ready { "Enable a configuration to rotate." } else { "Choose the next connection." },
                            }
                        }
                        super::details::RuntimeDetailsButton { data: data.clone(), stale: stale() }
                        TunControl {
                            enabled: meta.tun_enabled,
                            active: meta.tun_active,
                            available: meta.tun_available && !stale(),
                            admin,
                            reason: meta.tun_reason.clone(),
                        }
                    }
                }
            }
            div { class: "dashboard-summary-grid",
                SubscriptionCard {
                    subscriptions: data.subscriptions.clone(),
                    refreshed: meta.last_subscription_refresh,
                    operator,
                    stale: stale(),
                }
                HealthCard { configs: data.configs.clone() }
                LatencyCard {
                    configs: data.configs.clone(),
                    enabled_metrics: meta.enabled_metrics.clone(),
                }
            }
            section {
                class: "dashboard-traffic",
                aria_label: "Live traffic and connection quality",
                div { class: "dashboard-section-title",
                    h2 {
                        Activity {}
                        "Traffic & connection quality"
                    }
                    span {
                        if stale() {
                            "Offline"
                        } else {
                            "Updates every 3 seconds"
                        }
                    }
                }
                div { class: "dashboard-chart-grid",
                    Card { class: "dashboard-chart-card",
                        div { class: "dashboard-card-heading",
                            h3 {
                                ArrowDown {}
                                "Throughput"
                            }
                            span { "Current session" }
                        }
                        dl { class: "dashboard-traffic-metrics",
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
                            footer: rsx! {
                                p { class: "dashboard-footnote", "Last 5 minutes · collected by XRAT · resets per connection" }
                            },
                        }
                    }
                    Card { class: "dashboard-chart-card",
                        div { class: "dashboard-card-heading",
                            h3 {
                                Radio {}
                                "Probe history"
                            }
                            span { "Active configuration" }
                        }
                        dl { class: "dashboard-probe-metrics",
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
                            footer: rsx! {
                                p { class: "dashboard-footnote dashboard-chart-footer",
                                    span { "Last 30 tests · gaps = failed / missing" }
                                    if let Some(point) = meta.probes.last() {
                                        span { class: "dashboard-last-tested",
                                            "Last tested "
                                            {display_date(&point.tested_at)}
                                        }
                                    }
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn LiveClock() -> Element {
    let mut tick = use_signal(|| 0_u64);
    use_future(move || async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(1000).await;
            tick += 1;
        }
    });
    let _ = tick();
    let date = js_sys::Date::new_0();
    let clock = format!(
        "{:02}:{:02}:{:02}",
        date.get_hours(),
        date.get_minutes(),
        date.get_seconds()
    );
    let calendar = date
        .to_locale_date_string("en-GB", &js_sys::Object::new())
        .as_string()
        .unwrap_or_default();
    rsx! {
        div {
            class: "dashboard-clock",
            aria_label: "Current local time and date",
            time { "{clock}" }
            span { "{calendar}" }

        }
    }
}

pub fn display_date(raw: &str) -> String {
    raw.trim()
        .parse::<i64>()
        .ok()
        .map(|value| value as f64)
        .or_else(|| crate::ui::history::event_time(raw))
        .filter(|value| value.is_finite())
        .map(|value| crate::ui::history::timestamp(value as i64))
        .unwrap_or_else(|| "Time unavailable".into())
}
