use super::{controls::DashboardAction, stats::*};
use crate::ui::card::Card;
use dioxus::prelude::*;
use icons::{Activity, CircleCheck, ListRestart};
use xrat_web::types::{Action, ConfigRow, LatencyMetric, SubscriptionRow};

#[component]
pub fn SubscriptionCard(
    subscriptions: Vec<SubscriptionRow>,
    refreshed: Option<i64>,
    operator: bool,
    stale: bool,
) -> Element {
    let urls = subscriptions
        .iter()
        .filter(|row| row.source == "url")
        .count();
    rsx! {
        Card { class: "dashboard-summary-card dashboard-subscriptions",
            h3 {
                ListRestart {}
                "Subscriptions"
            }
            div { class: "dashboard-big-number",
                "{subscriptions.len()}"
                span { "sources" }
            }
            p { class: "dashboard-description",
                if subscriptions.is_empty() {
                    "Your next connection starts with a subscription."
                } else {
                    "{urls} can be refreshed from their URL."
                }
            }
            div { class: "dashboard-subscription-footer",
                div {
                    span { class: "dashboard-small-label", "Last successful refresh" }
                    p {
                        if let Some(at) = refreshed {
                            {crate::ui::history::timestamp(at)}
                        } else if urls == 0 && !subscriptions.is_empty() {
                            "Local sources don’t refresh"
                        } else {
                            "Not refreshed yet"
                        }
                    }
                }
                if operator {
                    DashboardAction {
                        action: Action::Refresh { id: None },
                        text: "Update subscriptions",
                        disabled: urls == 0 || stale,
                        reason: if urls == 0 { "Add a URL subscription to refresh it." } else { "Refresh all URL subscriptions." },
                    }
                }
            }
        }
    }
}

#[component]
pub fn HealthCard(configs: Vec<ConfigRow>) -> Element {
    let counts = config_distribution(&configs);
    let total = counts.iter().sum::<usize>();
    let names = ["Real Delay", "TCP", "ICMP", "Failed", "Untested"];
    let circumference = std::f64::consts::TAU * 42.0;
    let mut offset = 0.0;
    let segments = counts
        .iter()
        .map(|count| {
            let length = if total == 0 {
                0.0
            } else {
                *count as f64 / total as f64 * circumference
            };
            let start = offset;
            offset += length;
            (length, start)
        })
        .collect::<Vec<_>>();
    let summary = names
        .iter()
        .zip(counts)
        .map(|(name, count)| format!("{name}: {count}"))
        .collect::<Vec<_>>()
        .join(", ");
    rsx! {
        Card { class: "dashboard-summary-card",
            h3 {
                CircleCheck {}
                "Config health"
            }
            div { class: "dashboard-health-body",
                div { class: "dashboard-donut",
                    svg {
                        view_box: "0 0 110 110",
                        role: "img",
                        "aria-label": format!("{total} total configs. {summary}"),
                        circle {
                            cx: "55",
                            cy: "55",
                            r: "42",
                            fill: "none",
                            class: "donut-track",
                            stroke_width: "10",
                        }
                        for (index, (length, start)) in segments.iter().enumerate() {
                            if *length > 0.0 {
                                circle {
                                    key: "{index}",
                                    cx: "55",
                                    cy: "55",
                                    r: "42",
                                    fill: "none",
                                    class: "donut-category-{index}",
                                    stroke_width: "10",
                                    stroke_dasharray: format!("{length} {circumference}"),
                                    stroke_dashoffset: format!("{}", -start),
                                    transform: "rotate(-90 55 55)",
                                }
                            }
                        }
                    }
                    div { class: "dashboard-donut-label",
                        strong { "{total}" }
                        span { "configs" }
                    }
                }
                dl { class: "dashboard-health-legend",
                    for (index, name) in names.iter().enumerate() {
                        div { key: "{name}",
                            dt {
                                span { class: "health-dot health-dot-{index}" }
                                "{name}"
                            }
                            dd { "{counts[index]}" }
                        }
                    }
                }
            }
            p { class: "dashboard-footnote",
                if total == 0 {
                    "Add a configuration to see its test health."
                } else {
                    "Each config appears once, under its strongest passing test."
                }
            }
        }
    }
}

#[component]
pub fn LatencyCard(configs: Vec<ConfigRow>, enabled_metrics: Vec<LatencyMetric>) -> Element {
    rsx! {
        Card { class: "dashboard-summary-card",
            h3 {
                Activity {}
                "Average latency"
            }
            if enabled_metrics.is_empty() {
                p { class: "dashboard-description",
                    "Enable latency tests in Settings to see averages."
                }
            } else {
                super::latency_chart::LatencyBarChart { configs, metrics: enabled_metrics }
            }
            p { class: "dashboard-footnote",
                "Latest results · enabled configs · successful samples only"
            }
        }
    }
}
