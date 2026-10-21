use crate::ui::dialog::{Dialog, DialogContent, DialogDescription, DialogTitle};
use dioxus::prelude::*;
use icons::FileText;
use xrat_web::types::{ConfigRow, Overview};

#[component]
pub fn ConnectionDetailsLink(data: Overview, stale: bool, name: String) -> Element {
    let mut open = use_signal(|| false);
    let active = data
        .active_config
        .and_then(|id| data.configs.iter().find(|config| config.id == id))
        .cloned();
    let disabled = active.is_none();
    rsx! {
        button {
            r#type: "button",
            class: concat!(
                "dashboard-connection-name border-0 bg-transparent text-inherit [font:inherit] cursor-pointer ",
                "text-left wrap-anywhere underline ",
                "decoration-[color-mix(in_srgb,_var(--muted)_40%,_transparent)] decoration-dotted ",
                "underline-offset-[3px] p-0 [&:hover]:text-foreground [&:hover]:decoration-solid ",
                "[&:focus-visible]:[outline:2px_solid_var(--ring)] [&:focus-visible]:[outline-offset:4px] ",
                "[&:focus-visible]:rounded-[2px] [&:disabled]:cursor-default [&:disabled]:no-underline",
            ),
            aria_label: format!("Show connection details for {name}"),
            disabled,
            onclick: move |_| open.set(true),
            "{name}"
        }
        if open() {
            ConnectionDialog {
                config: active,
                data,
                stale,
                on_close: move |_| open.set(false),
            }
        }
    }
}

#[component]
fn ConnectionDialog(
    config: Option<ConfigRow>,
    data: Overview,
    stale: bool,
    on_close: EventHandler,
) -> Element {
    let subscription = config
        .as_ref()
        .and_then(|config| config.subscription_id)
        .map(|id| {
            data.subscriptions
                .iter()
                .find(|sub| sub.id == id)
                .map(|sub| sub.name.clone())
                .unwrap_or_else(|| format!("Subscription #{id}"))
        })
        .unwrap_or_else(|| "Local configuration".into());
    rsx! {
        Dialog {
            DialogContent {
                class: "dashboard-details-dialog w-[min(560px,_calc(100vw_-_32px))] max-w-[calc(100vw_-_32px)] max-h-[calc(100dvh_-_32px)] overflow-auto rounded-xl bg-card p-5 mobile:p-4",
                initial_open: true,
                on_close,
                div { class: "flex items-start gap-3 pr-7",
                    span { class: "grid size-9 shrink-0 place-items-center rounded-lg bg-primary/8 text-primary [&_svg]:size-4",
                        FileText {}
                    }
                    div { class: "min-w-0",
                        DialogTitle { class: "text-[11px] font-medium leading-4 text-muted-foreground", "Connection details" }
                        if let Some(config) = &config {
                            h4 { class: "mt-1 mb-0 text-[17px] font-semibold leading-snug tracking-tight wrap-anywhere",
                                if config.name.is_empty() { "Configuration #{config.id}" } else { "{config.name}" }
                            }
                        }
                    }
                }
                if stale {
                    div { role: "status",
                        DialogDescription { class: "mt-3 mb-0 border-l-2 border-primary/40 pl-2.5 text-[11px] leading-relaxed", "Updates paused · details may be out of date." }
                    }
                }
                if let Some(config) = config {
                    div { class: "mt-3 flex flex-wrap items-center gap-1.5 text-[10px] font-medium",
                        span { class: "rounded bg-muted/50 px-2 py-1 uppercase tracking-wide", "{config.protocol}" }
                        span { class: "rounded bg-muted/35 px-2 py-1 text-muted-foreground",
                            if let Some(tls) = &config.tls { "{config.network} + {tls}" } else { "{config.network}" }
                        }
                        span { class: "ml-auto inline-flex items-center gap-1.5 text-[11px] text-muted-foreground",
                            span { class: if data.running { "size-1.5 rounded-full bg-primary" } else { "size-1.5 rounded-full bg-muted-foreground" } }
                            if data.running { "Connected" } else { "Disconnected" }
                        }
                    }
                    div { class: "mt-3 rounded-lg border border-border/50 bg-muted/15 px-3 py-2.5",
                        div { class: "mb-1 text-[10px] text-muted-foreground", "Endpoint" }
                        code { class: "block text-[12px] leading-relaxed tabular-nums wrap-anywhere select-all",
                            {endpoint(&config.address, config.port)}
                        }
                    }
                    dl { class: concat!(
                        "mt-3 mb-0 grid grid-cols-2 gap-x-5 gap-y-3 ",
                        "[&&_dt]:text-[10px] [&&_dt]:mb-1 [&&_dd]:text-[12px] [&&_dd]:leading-relaxed [&_>_div:last-child]:col-span-full ",
                        "[@media(width<=400px)]:grid-cols-1",
                    ),
                        Fact { label: "Subscription", value: subscription }
                        Fact { label: "Reference", value: available(&config.reference) }
                        Fact {
                            label: "Location",
                            value: match (&config.country, &config.location) {
                                (Some(country), Some(location)) => format!("{country} · {location}"),
                                (Some(value), None) | (None, Some(value)) => value.clone(),
                                (None, None) => "Not recorded".into(),
                            },
                        }
                        Fact { label: "ASN", value: config.asn.clone().unwrap_or_else(|| "Not recorded".into()) }
                        Fact {
                            label: "Fronting",
                            value: config.fronting.clone().map(|fronting| format!("{fronting} · dial endpoint; origin unknown")).unwrap_or_else(|| "Not recorded".into()),
                        }
                    }
                    if let Some(meta) = &data.dashboard {
                        if !meta.enabled_metrics.is_empty() {
                            dl { class: "mt-4 mb-0 grid grid-cols-3 gap-3 border-t border-border/50 pt-3 tabular-nums",
                                for metric in &meta.enabled_metrics {
                                    div { key: "{super::stats::key(*metric)}", class: "min-w-0",
                                        dt { class: "mb-1 text-[10px] text-muted-foreground", {super::stats::label(*metric)} }
                                        dd { class: "m-0 text-[13px] font-semibold leading-5 wrap-anywhere", {test_value(&config, *metric)} }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "mt-3 flex flex-wrap items-center gap-x-3 gap-y-1.5 border-t border-border/50 pt-3 text-[10px] text-muted-foreground",
                        span { class: "font-mono tabular-nums", "#{config.id}" }
                        span { if config.enabled { "Enabled" } else { "Disabled" } }
                        span { if config.deleted { "Deleted" } else { "Not deleted" } }
                    }
                } else {
                    div { class: "mt-4 text-[12px] leading-relaxed text-muted-foreground",
                        "No active configuration yet. Connect a configuration to see its details here."
                    }
                }
            }
        }
    }
}

#[component]
pub(super) fn Fact(label: String, value: String) -> Element {
    rsx! {
        div { class: "dashboard-detail-fact [&_dt]:text-muted-foreground [&_dt]:text-[11px] [&_dt]:mb-[3px] [&_dd]:text-[12px] [&_dd]:wrap-anywhere [&_dd]:m-0",
            dt { "{label}" }
            dd { "{value}" }
        }
    }
}

pub(super) fn endpoint(host: &str, port: i64) -> String {
    if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    }
}

fn available(value: &str) -> String {
    if value.is_empty() {
        "Not recorded".into()
    } else {
        value.into()
    }
}

fn test_value(config: &ConfigRow, metric: xrat_web::types::LatencyMetric) -> String {
    let (ok, value) = config
        .test
        .as_ref()
        .map(|test| super::stats::measurement(test, metric))
        .unwrap_or_default();
    match (ok, value.filter(|value| *value >= 0)) {
        (Some(true), Some(ms)) => format!("{ms} ms"),
        (Some(false), _) => "Failed".into(),
        _ => "Not measured".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xrat_web::types::{ConfigTest, LatencyMetric};

    #[test]
    fn endpoints_handle_ipv6_without_double_brackets() {
        assert_eq!(endpoint("::1", 443), "[::1]:443");
        assert_eq!(endpoint("[::1]", 443), "[::1]:443");
        assert_eq!(endpoint("proxy.example", 443), "proxy.example:443");
    }

    #[test]
    fn config_details_distinguish_missing_failed_and_valid_latency() {
        let mut config = ConfigRow::default();
        assert_eq!(test_value(&config, LatencyMetric::Tcp), "Not measured");
        config.test = Some(ConfigTest {
            tcp_ok: Some(false),
            tcp_ms: Some(99),
            ..Default::default()
        });
        assert_eq!(test_value(&config, LatencyMetric::Tcp), "Failed");
        config.test.as_mut().unwrap().tcp_ok = Some(true);
        config.test.as_mut().unwrap().tcp_ms = Some(-1);
        assert_eq!(test_value(&config, LatencyMetric::Tcp), "Not measured");
        config.test.as_mut().unwrap().tcp_ms = Some(0);
        assert_eq!(test_value(&config, LatencyMetric::Tcp), "0 ms");
    }
}
