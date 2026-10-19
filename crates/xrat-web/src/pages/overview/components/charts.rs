use super::layout::CHART_TONES;
use super::stats::{RatePoint, bytes, label, milliseconds};
use dioxus::prelude::*;
use xrat_web::types::{LatencyMetric, ProbePoint};

#[derive(Clone, Debug, PartialEq)]
pub struct ChartPoint {
    pub id: i64,
    pub x: f64,
    pub y: Option<f64>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ChartSeries {
    pub name: String,
    pub tone: usize,
    pub points: Vec<ChartPoint>,
}

pub fn traffic_series(points: &[RatePoint]) -> Vec<ChartSeries> {
    [("Download", 0), ("Upload", 1)]
        .into_iter()
        .map(|(name, tone)| ChartSeries {
            name: name.into(),
            tone,
            points: points
                .iter()
                .map(|point| ChartPoint {
                    id: point.at as i64,
                    x: point.at,
                    y: if tone == 0 { point.down } else { point.up },
                })
                .collect(),
        })
        .collect()
}
pub fn probe_value(point: &ProbePoint, metric: LatencyMetric) -> Option<f64> {
    match metric {
        LatencyMetric::Tcp => point.tcp_ms,
        LatencyMetric::Icmp => point.icmp_ms,
        LatencyMetric::RealDelay => point.real_delay_ms,
    }
    .filter(|value| *value >= 0)
    .map(|value| value as f64)
}
pub fn probe_series(points: &[ProbePoint], metrics: &[LatencyMetric]) -> Vec<ChartSeries> {
    metrics
        .iter()
        .map(|metric| ChartSeries {
            name: label(*metric).into(),
            tone: super::stats::METRICS
                .iter()
                .position(|candidate| candidate == metric)
                .unwrap_or(0),
            points: points
                .iter()
                .filter_map(|point| {
                    crate::components::history::event_time(&point.tested_at).map(|seconds| {
                        ChartPoint {
                            id: point.id,
                            x: seconds * 1000.0,
                            y: probe_value(point, *metric),
                        }
                    })
                })
                .collect(),
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq)]
pub struct PlotBounds {
    pub width: f64,
    pub height: f64,
}

impl PlotBounds {
    pub fn right(self) -> f64 {
        self.width - 12.0
    }

    pub fn bottom(self) -> f64 {
        self.height - 26.0
    }

    pub fn x(self, value: f64, start: f64, span: f64) -> f64 {
        64.0 + (value - start) / span.max(1.0) * (self.right() - 64.0)
    }

    pub fn y(self, value: f64, maximum: f64) -> f64 {
        self.bottom() - value / maximum.max(1.0) * (self.bottom() - 12.0)
    }
}

pub fn paths(
    points: &[ChartPoint],
    start: f64,
    span: f64,
    maximum: f64,
    area: bool,
    bounds: PlotBounds,
) -> Vec<String> {
    let mut paths = Vec::new();
    let mut path = String::new();
    let mut first = 0.0;
    let mut last = 0.0;
    let finish = |paths: &mut Vec<String>, path: &mut String, first: f64, last: f64| {
        if !path.is_empty() {
            if area {
                path.push_str(&format!(
                    " L{last:.2},{bottom:.2} L{first:.2},{bottom:.2} Z",
                    bottom = bounds.bottom()
                ));
            }
            paths.push(std::mem::take(path));
        }
    };
    for point in points {
        if let Some(value) = point.y.filter(|value| value.is_finite() && *value >= 0.0)
            && point.x.is_finite()
        {
            let x = bounds.x(point.x, start, span);
            let y = bounds.y(value, maximum);
            if path.is_empty() {
                first = x;
                path.push_str(&format!("M{x:.2},{y:.2}"));
            } else {
                path.push_str(&format!(" L{x:.2},{y:.2}"));
            }
            last = x;
        } else {
            finish(&mut paths, &mut path, first, last);
        }
    }
    finish(&mut paths, &mut path, first, last);
    paths
}

fn value_label(value: f64, unit: &str) -> String {
    if unit == "bytes" {
        format!("{}/s", bytes(value))
    } else {
        milliseconds(Some(value))
    }
}
fn time_label(value: f64) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(value));
    format!(
        "{:02}:{:02}:{:02}",
        date.get_hours(),
        date.get_minutes(),
        date.get_seconds()
    )
}

#[component]
pub fn PlotChart(
    series: Vec<ChartSeries>,
    area: bool,
    unit: String,
    empty: String,
    chart_label: String,
) -> Element {
    let mut hovered = use_signal(|| None::<usize>);
    let mut viewport = use_signal(|| PlotBounds {
        width: 480.0,
        height: 196.0,
    });
    let bounds = viewport();
    let points = series
        .first()
        .map(|series| series.points.as_slice())
        .unwrap_or_default();
    let min = points.first().map(|point| point.x).unwrap_or_default();
    let max = points.last().map(|point| point.x).unwrap_or(min);
    let span = (max - min).max(1000.0);
    let maximum = series
        .iter()
        .flat_map(|series| &series.points)
        .filter_map(|point| point.y)
        .filter(|value| value.is_finite())
        .fold(1.0, f64::max)
        * 1.1;
    let present = series
        .iter()
        .flat_map(|series| &series.points)
        .any(|point| point.y.is_some());
    let focused = hovered().filter(|index| *index < points.len());
    let descriptions = points
        .iter()
        .enumerate()
        .map(|(index, point)| {
            let values = series
                .iter()
                .map(|series| {
                    format!(
                        "{} {}",
                        series.name,
                        series
                            .points
                            .get(index)
                            .and_then(|point| point.y)
                            .map(|value| value_label(value, &unit))
                            .unwrap_or_else(|| "unavailable".into())
                    )
                })
                .collect::<Vec<_>>()
                .join(" · ");
            format!(
                "{} · {values}",
                crate::components::history::timestamp((point.x / 1000.0) as i64)
            )
        })
        .collect::<Vec<_>>();
    rsx! {
        div {
            class: concat!(
                "dashboard-plot relative min-w-0 min-[1001px]:h700:flex-col ",
                "min-[1001px]:h700:min-h-0 ",
                "min-[1001px]:h700:flex-1 ",
                "min-[1001px]:h700:flex [&_>_svg]:w-full ",
                "[&_>_svg]:h-[clamp(160px,_22dvh,_260px)] [&_>_svg]:overflow-visible [&_>_svg]:block ",
                "min-[1001px]:h700:[&_>_svg]:min-h-30 ",
                "min-[1001px]:h700:[&_>_svg]:flex-1 ",
                "min-[1001px]:h700:[&_>_svg]:h-0",
            ),
            "data-name": if area { "AreaChart" } else { "LineChart" },
            if present {
                svg {
                    view_box: format!("0 0 {} {}", bounds.width, bounds.height),
                    onresize: move |event: ResizeEvent| {
                        if let Ok(size) = event.get_content_box_size() {
                            let next = PlotBounds {
                                width: size.width.round().max(120.0),
                                height: size.height.round().max(80.0),
                            };
                            if next != *viewport.peek() {
                                viewport.set(next);
                            }
                        }
                    },
                    role: "img",
                    "aria-label": chart_label,
                    onmouseleave: move |_| hovered.set(None),
                    for fraction in [0.0, 0.5, 1.0] {
                        g {
                            line {
                                x1: "64",
                                x2: format!("{}", bounds.right()),
                                y1: format!("{}", bounds.y(maximum * fraction, maximum)),
                                y2: format!("{}", bounds.y(maximum * fraction, maximum)),
                                class: "chart-grid-line stroke-[color-mix(in_srgb,_var(--border)_22%,_transparent)] [stroke-dasharray:3_5]",
                            }
                            text {
                                x: "58",
                                y: format!("{}", bounds.y(maximum * fraction, maximum) + 4.0),
                                text_anchor: "end",
                                class: "chart-axis-text fill-[var(--muted)] text-[11px] [font-family:inherit]",
                                {value_label(maximum * fraction, &unit)}
                            }
                        }
                    }
                    for series in &series {
                        g {
                            key: "{series.name}",
                            class: format!("chart-series {}", CHART_TONES[series.tone]),
                            if area {
                                for path in paths(&series.points, min, span, maximum, true, bounds) {
                                    path {
                                        d: path,
                                        class: "chart-area fill-[var(--series-color)] opacity-10",
                                    }
                                }
                            }
                            for path in paths(&series.points, min, span, maximum, false, bounds) {
                                path {
                                    d: path,
                                    class: "chart-line stroke-[var(--series-color)] stroke-[2] [stroke-linejoin:round] [stroke-linecap:round] [vector-effect:non-scaling-stroke]",
                                    fill: "none",
                                }
                            }
                            for point in &series.points {
                                if let Some(value) = point.y {
                                    circle {
                                        cx: format!("{}", bounds.x(point.x, min, span)),
                                        cy: format!("{}", bounds.y(value, maximum)),
                                        r: if series.points.len() == 1 { "3" } else { "1.6" },
                                        class: "chart-dot fill-[var(--series-color)]",
                                    }
                                }
                            }
                        }
                    }
                    for (index, point) in points.iter().enumerate() {
                        g {
                            key: "{point.id}",
                            tabindex: if index + 1 == points.len() { "0" } else { "-1" },
                            role: "img",
                            "aria-label": descriptions[index].clone(),
                            class: "chart-sample [&:focus]:[outline:none] [&:hover_.chart-crosshair]:opacity-60 [&:focus_.chart-crosshair]:opacity-60",
                            onmouseenter: move |_| hovered.set(Some(index)),
                            onclick: move |_| hovered.set(Some(index)),
                            onfocus: move |_| hovered.set(Some(index)),
                            onblur: move |_| hovered.set(None),
                            rect {
                                x: format!("{}", bounds.x(point.x, min, span) - 3.0),
                                y: "12",
                                width: "6",
                                height: format!("{}", bounds.bottom() - 12.0),
                                fill: "transparent",
                            }
                            line {
                                x1: format!("{}", bounds.x(point.x, min, span)),
                                x2: format!("{}", bounds.x(point.x, min, span)),
                                y1: "12",
                                y2: format!("{}", bounds.bottom()),
                                class: "chart-crosshair stroke-[var(--muted)] [stroke-dasharray:2_3] opacity-0 pointer-events-none",
                            }
                        }
                    }
                    text {
                        x: "64",
                        y: format!("{}", bounds.height - 5.0),
                        class: "chart-axis-text fill-[var(--muted)] text-[11px] [font-family:inherit]",
                        {time_label(min)}
                    }
                    if max > min {
                        text {
                            x: format!("{}", bounds.right()),
                            y: format!("{}", bounds.height - 5.0),
                            text_anchor: "end",
                            class: "chart-axis-text fill-[var(--muted)] text-[11px] [font-family:inherit]",
                            {time_label(max)}
                        }
                    }
                }
            } else {
                div {
                    class: concat!(
                        "dashboard-chart-empty min-h-30 grid place-items-center border-b border-dashed ",
                        "border-b-[color:color-mix(in_srgb,_var(--border)_45%,_transparent)] ",
                        "[background:linear-gradient(_transparent_98%,_color-mix(in_srgb,_var(--border)_10%,_transparent)_98%_)] ",
                        "[background-size:100%_52px] min-[1001px]:h700:min-h-16 ",
                        "min-[1001px]:h700:flex-1 [&_p]:text-center [&_p]:max-w-[34ch] ",
                        "[&_p]:text-muted-foreground [&_p]:text-[12px] [&_p]:leading-[1.8] [&_p]:m-3",
                    ),
                    p { "{empty}" }
                }
            }
            div { class: "dashboard-chart-meta shrink-0 flex items-center justify-between flex-wrap gap-y-1.5 gap-x-4 mt-2 [&_.dashboard-chart-legend]:shrink-[0] [&_.dashboard-chart-legend]:m-0",
                div { class: "dashboard-chart-legend flex flex-wrap mt-[5px] gap-3 min-[1001px]:h700:shrink-[0] [&_>_span]:inline-flex [&_>_span]:items-center [&_>_span]:text-muted-foreground [&_>_span]:text-[10px] [&_>_span]:gap-1.5",
                    for series in &series {
                        span { key: "{series.name}",
                            i {
                                class: format!(
                                    "chart-legend-dot {} inline-block size-[7px] rounded-[2px] bg-[color:var(--series-color)]",
                                    CHART_TONES[series.tone],
                                ),
                            }
                            "{series.name}"
                        }
                    }
                }
            }
            if present {
                div { class: "dashboard-chart-readout text-center w-full mt-2 mb-0 min-h-4.5 text-muted-foreground text-[10px] leading-[1.6] wrap-anywhere mx-0 min-[1001px]:h700:shrink-[0]",
                    if let Some(index) = focused {
                        "{descriptions[index]}"
                    } else {
                        "Hover, tap, or use ←/→ on a focused chart"
                    }
                }
            }
        }
    }
}
