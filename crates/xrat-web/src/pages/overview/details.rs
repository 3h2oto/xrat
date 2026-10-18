use crate::ui::{
    button::ButtonVariant,
    dialog::{Dialog, DialogContent, DialogDescription, DialogTitle},
    tooltip::{Tooltip, TooltipContent},
};
use dioxus::prelude::*;
use icons::{CircleCheck, CircleX, FileText, Info};
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
pub fn RuntimeDetailsButton(data: Overview, stale: bool) -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        crate::ui::icon_button::IconButton {
            label: "Show runtime details",
            variant: ButtonVariant::Ghost,
            onclick: move |_| open.set(true),
            Info {}
        }
        if open() {
            super::runtime::RuntimeDialog { data, stale, on_close: move |_| open.set(false) }
        }
    }
}

#[component]
fn ConnectionDialog(
    config: Option<ConfigRow>,
    data: Overview,
    stale: bool,
    on_close: EventHandler<MouseEvent>,
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
                class: "dashboard-details-dialog w-[min(720px,_calc(100vw_-_32px))] max-w-[calc(100vw_-_32px)] max-h-[calc(100dvh_-_40px)] overflow-auto rounded-[12px] bg-card p-5.5",
                initial_open: true,
                on_close,
                div { class: "dashboard-details-heading flex items-start pr-7 gap-2.5 [&_>_svg]:size-4.5 [&_>_svg]:text-primary [&_>_svg]:shrink-[0] [&_h3]:text-[17px] [&_p]:mt-1.5 [&_p]:mb-0 [&_p]:text-[12px] [&_p]:text-muted-foreground [&_p]:mx-0",
                    FileText {}
                    div {
                        DialogTitle { "Connection details" }
                        DialogDescription {
                            if stale {
                                "Updates paused · these details may be out of date."
                            } else {
                                "The active configuration and its latest test results."
                            }
                        }
                    }
                }
                if let Some(config) = config {
                    h4 { class: "dashboard-details-name mt-5 mb-0 text-[14px] wrap-anywhere mx-0",
                        if config.name.is_empty() {
                            "Configuration #{config.id}"
                        } else {
                            "{config.name}"
                        }
                    }
                    dl { class: "dashboard-details-grid grid grid-cols-[repeat(2,_minmax(0,_1fr))] gap-y-3.5 gap-x-6 mt-4.5 mb-0 mx-0 [@media(width<=540px)]:grid-cols-[minmax(0,_1fr)]",
                        Fact { label: "ID", value: config.id.to_string() }
                        Fact {
                            label: "Reference",
                            value: available(&config.reference),
                        }
                        Fact { label: "Protocol", value: config.protocol.clone() }
                        Fact {
                            label: "Endpoint",
                            value: endpoint(&config.address, config.port),
                        }
                        Fact {
                            label: "Network",
                            value: config
                                .tls
                                .as_ref()
                                .map(|tls| format!("{} + {tls}", config.network))
                                .unwrap_or(config.network.clone()),
                        }
                        Fact { label: "Subscription", value: subscription }
                        Fact {
                            label: "Country",
                            value: config.country.clone().unwrap_or_else(|| "Not recorded".into()),
                        }
                        Fact {
                            label: "City / region",
                            value: config.location.clone().unwrap_or_else(|| "Not recorded".into()),
                        }
                        Fact {
                            label: "ASN",
                            value: config.asn.clone().unwrap_or_else(|| "Not recorded".into()),
                        }
                        Fact {
                            label: "Fronting",
                            value: config
                                .fronting
                                .clone()
                                .map(|fronting| format!("{fronting} · dial endpoint; origin unknown"))
                                .unwrap_or_else(|| "Not recorded".into()),
                        }
                        for metric in data.dashboard.as_ref().map(|meta| meta.enabled_metrics.clone()).unwrap_or_default() {
                            Fact {
                                key: "{super::stats::key(metric)}",
                                label: super::stats::label(metric),
                                value: test_value(&config, metric),
                            }
                        }
                        BooleanFact { label: "Enabled", value: config.enabled }
                        BooleanFact { label: "Active", value: data.running }
                        BooleanFact { label: "Deleted", value: config.deleted }
                    }
                } else {
                    div { class: "dashboard-details-empty mt-5.5 text-muted-foreground text-[12px]",
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

#[component]
pub(super) fn BooleanFact(label: String, value: bool) -> Element {
    let text = if value { "Yes" } else { "No" };
    rsx! {
        div { class: "dashboard-detail-fact [&_dt]:text-muted-foreground [&_dt]:text-[11px] [&_dt]:mb-[3px] [&_dd]:text-[12px] [&_dd]:wrap-anywhere [&_dd]:m-0",
            dt { "{label}" }
            dd {
                Tooltip { class: "runtime-boolean inline-flex", tabindex: "0",
                    span {
                        class: if value { "runtime-health is-healthy inline-flex items-center text-[color:var(--chart-1)] text-[11px] gap-[5px] [&_svg]:size-[13px] [&_svg]:shrink-[0] [&.is-unhealthy]:text-destructive" } else { "runtime-health inline-flex items-center text-[color:var(--chart-1)] text-[11px] gap-[5px] [&_svg]:size-[13px] [&_svg]:shrink-[0] [&.is-unhealthy]:text-destructive" },
                        role: "img",
                        aria_label: text,
                        if value {
                            CircleCheck {}
                        } else {
                            CircleX {}
                        }
                    }
                    TooltipContent { "{text}" }
                }
            }
        }
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
