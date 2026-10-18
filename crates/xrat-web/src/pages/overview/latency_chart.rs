use super::{
    charts::PlotBounds,
    layout::CHART_TONES,
    stats::{METRICS, health, key, label, milliseconds},
};
use dioxus::prelude::*;
use xrat_web::types::{ConfigRow, LatencyMetric};

#[component]
pub fn LatencyBarChart(configs: Vec<ConfigRow>, metrics: Vec<LatencyMetric>) -> Element {
    let mut focused = use_signal(|| None::<usize>);
    let mut viewport = use_signal(|| PlotBounds {
        width: 400.0,
        height: 136.0,
    });
    let bounds = viewport();
    let bars = metrics
        .iter()
        .map(|metric| (*metric, health(&configs, *metric)))
        .collect::<Vec<_>>();
    let maximum = bars
        .iter()
        .filter_map(|(_, result)| result.average)
        .fold(1.0, f64::max)
        * 1.25;
    let width = (bounds.right() - 48.0) / bars.len().max(1) as f64;
    let values = bars
        .iter()
        .map(|(_, result)| result.average)
        .collect::<Vec<_>>();
    let labels = metrics
        .iter()
        .map(|metric| label(*metric))
        .collect::<Vec<_>>();
    let description = focused()
        .and_then(|index| bars.get(index))
        .map(|(metric, result)| {
            format!(
                "{}: {} · {} successful samples",
                label(*metric),
                milliseconds(result.average),
                result.samples
            )
        });
    rsx! {
        div {
            class: "dashboard-plot dashboard-latency-chart relative min-w-0 flex flex-col flex-1 min-h-45 [&_>_svg]:w-full [&_>_svg]:h-0 [&_>_svg]:overflow-visible [&_>_svg]:block [&_>_svg]:min-h-40 [&_>_svg]:flex-1",
            "data-name": "BarChart",
            "data-chart-values": serde_json::to_string(&values).unwrap_or_default(),
            "data-chart-labels": serde_json::to_string(&labels).unwrap_or_default(),
            svg {
                role: "img",
                "aria-label": "Average successful latency by enabled test type",
                view_box: format!("0 0 {} {}", bounds.width, bounds.height),
                onresize: move |event: ResizeEvent| {
                    if let Ok(size) = event.get_content_box_size() {
                        let next = PlotBounds {
                            width: size.width.round().max(180.0),
                            height: size.height.round().max(90.0),
                        };
                        if next != *viewport.peek() {
                            viewport.set(next);
                        }
                    }
                },
                for fraction in [0.0, 0.5, 1.0] {
                    g {
                        line {
                            x1: "48",
                            x2: format!("{}", bounds.right()),
                            y1: format!("{}", bounds.y(maximum * fraction, maximum)),
                            y2: format!("{}", bounds.y(maximum * fraction, maximum)),
                            class: "chart-grid-line stroke-[color-mix(in_srgb,_var(--border)_22%,_transparent)] [stroke-dasharray:3_5]",
                        }
                        text {
                            x: "42",
                            y: format!("{}", bounds.y(maximum * fraction, maximum) + 4.0),
                            text_anchor: "end",
                            class: "chart-axis-text fill-[var(--muted)] text-[11px] [font-family:inherit]",
                            {format!("{:.0} ms", maximum * fraction)}
                        }
                    }
                }
                for (index, (metric, result)) in bars.iter().enumerate() {
                    g {
                        key: "{key(*metric)}",
                        tabindex: if index == 0 { "0" } else { "-1" },
                        class: format!(
                            "chart-sample latency-column {} [&:focus]:[outline:none] [&:hover_.chart-crosshair]:opacity-60 [&:focus_.chart-crosshair]:opacity-60 [&:hover_.latency-column-bar]:opacity-100 [&:focus_.latency-column-bar]:opacity-100 text-[color:var(--series-color)]",
                            CHART_TONES[METRICS.iter().position(|candidate| candidate == metric).unwrap_or(0)],
                        ),
                        role: "img",
                        "aria-label": format!(
                            "{}: {}, {} successful samples",
                            label(*metric),
                            milliseconds(result.average),
                            result.samples,
                        ),
                        onmouseenter: move |_| focused.set(Some(index)),
                        onmouseleave: move |_| focused.set(None),
                        onfocus: move |_| focused.set(Some(index)),
                        onblur: move |_| focused.set(None),
                        onclick: move |_| focused.set(Some(index)),
                        rect {
                            x: format!("{}", 48.0 + width * index as f64 + width * 0.1),
                            y: "0",
                            width: format!("{}", width * 0.8),
                            height: format!("{}", bounds.height),
                            fill: "transparent",
                        }
                        if let Some(value) = result.average {
                            rect {
                                x: format!("{}", 48.0 + width * index as f64 + width * 0.26),
                                y: format!("{}", bounds.y(value, maximum)),
                                width: format!("{}", width * 0.48),
                                height: format!("{}", (bounds.bottom() - bounds.y(value, maximum)).max(0.0)),
                                rx: "4",
                                class: "latency-column-bar fill-current opacity-85",
                            }
                        }
                        text {
                            x: format!("{}", 48.0 + width * (index as f64 + 0.5)),
                            y: format!("{}", bounds.y(result.average.unwrap_or_default(), maximum) - 7.0),
                            text_anchor: "middle",
                            class: "latency-column-value text-[10px] fill-[var(--foreground)]",
                            {milliseconds(result.average)}
                        }
                        text {
                            x: format!("{}", 48.0 + width * (index as f64 + 0.5)),
                            y: format!("{}", bounds.height - 5.0),
                            text_anchor: "middle",
                            class: "chart-axis-text fill-[var(--muted)] text-[11px] [font-family:inherit]",
                            {label(*metric)}
                        }
                    }
                }
            }
            div {
                class: "dashboard-latency-readout mt-1 min-h-3.5 text-[10px] text-muted-foreground text-center",
                aria_live: "polite",
                {description.unwrap_or_else(|| "Hover or focus a bar for sample counts".into())}
            }
        }
    }
}
