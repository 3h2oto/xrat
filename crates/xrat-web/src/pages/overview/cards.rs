use super::layout::{HEALTH_COLORS, SummaryCard};
use super::{controls::DashboardAction, stats::*};
use dioxus::prelude::*;
use icons::{Activity, CircleCheck};
use xrat_web::types::{Action, ConfigRow, LatencyMetric, SubscriptionRow};

#[component]
pub fn HealthCard(
    configs: Vec<ConfigRow>,
    subscriptions: Vec<SubscriptionRow>,
    refreshed: Option<i64>,
    operator: bool,
    stale: bool,
) -> Element {
    let urls = subscriptions
        .iter()
        .filter(|row| row.source == "url")
        .count();
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
        SummaryCard {
            h3 {
                CircleCheck {}
                "Config health"
            }
            div { class: "dashboard-health-body grid grid-cols-[minmax(0,_1fr)_minmax(100px,_124px)] items-center flex-1 gap-5 [@media(width<=600px)]:gap-4 [@media(width>600px)_and_(width<=1200px)]:gap-2.5",
                div { class: "dashboard-donut w-[clamp(112px,_12vw,_168px)] max-w-full justify-self-center relative [&_svg]:block [&_svg]:w-full [&_svg]:h-auto",
                    svg {
                        view_box: "0 0 110 110",
                        role: "img",
                        "aria-label": format!("{total} total configs. {summary}"),
                        circle {
                            cx: "55",
                            cy: "55",
                            r: "42",
                            fill: "none",
                            class: "donut-track stroke-[color-mix(in_srgb,_var(--muted)_18%,_transparent)]",
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
                                    class: format!("stroke-current {}", HEALTH_COLORS[index]),
                                    stroke_width: "10",
                                    stroke_dasharray: format!("{length} {circumference}"),
                                    stroke_dashoffset: format!("{}", -start),
                                    transform: "rotate(-90 55 55)",
                                }
                            }
                        }
                    }
                    div { class: "dashboard-donut-label absolute top-0 right-0 bottom-0 left-0 flex flex-col items-center justify-center pointer-events-none gap-[3px] [&_strong]:text-[30px] [&_strong]:font-semibold [&_strong]:tracking-[-0.7px] [&_span]:text-[10px] [&_span]:text-muted-foreground",
                        strong { "{total}" }
                        span { "configs" }
                    }
                }
                dl {
                    class: concat!(
                        "dashboard-health-legend grid min-w-0 m-0 gap-2 [&_>_div]:flex [&_>_div]:items-center ",
                        "[&_>_div]:justify-between [&_>_div]:text-[12px] [&_>_div]:gap-2.5 ",
                        "[@media(width<=1200px)]:[&_>_div]:gap-[5px] [&_dt]:flex [&_dt]:items-center ",
                        "[&_dt]:text-foreground [&_dt]:gap-1.5 [&_svg]:size-[13px] [&_dd]:font-medium ",
                        "[&_dd]:tabular-nums [&_dd]:m-0",
                    ),
                    for (index, name) in names.iter().enumerate() {
                        div { key: "{name}",
                            dt {
                                span { class: format!("size-[7px] rounded-full shrink-0 bg-current {}", HEALTH_COLORS[index]) }
                                "{name}"
                            }
                            dd { "{counts[index]}" }
                        }
                    }
                }
            }
            div { class: "dashboard-health-subscriptions flex items-center justify-between flex-wrap pt-1 gap-2 [&_>_div:first-child]:grid [&_>_div:first-child]:gap-[3px] [&_strong]:text-[12px] [&_strong]:font-medium",
                div {
                    strong {
                        "{subscriptions.len()} "
                        if subscriptions.len() == 1 {
                            "subscription"
                        } else {
                            "subscriptions"
                        }
                    }
                    span { class: "dashboard-small-label text-[10px] text-muted-foreground",
                        if let Some(at) = refreshed {
                            "Updated "
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
                        icon_only: true,
                        disabled: urls == 0 || stale,
                        reason: if urls == 0 { "Add a URL subscription to refresh it." } else { "Refresh all URL subscriptions." },
                    }
                }
            }
        }
    }
}

#[component]
pub fn LatencyCard(configs: Vec<ConfigRow>, enabled_metrics: Vec<LatencyMetric>) -> Element {
    rsx! {
        SummaryCard {
            h3 {
                Activity {}
                "Average latency"
            }
            if enabled_metrics.is_empty() {
                p { class: "dashboard-description text-[12px] leading-[1.7] text-muted-foreground m-0",
                    "Enable latency tests in Settings to see averages."
                }
            } else {
                super::latency_chart::LatencyBarChart { configs, metrics: enabled_metrics }
            }
        }
    }
}
