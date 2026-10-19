use crate::ui::{spinner::Spinner, tooltip::*};
use dioxus::prelude::*;
use icons::{CircleCheck, CircleMinus, CircleX, Clock};
use xrat_web::types::{ConfigTestProgress, TestMetric, TestMetricState};

pub fn column_state(
    progress: &[ConfigTestProgress],
    id: i64,
    column: &str,
) -> Option<TestMetricState> {
    let progress = progress.iter().find(|progress| progress.config_id == id)?;
    if progress.metrics.iter().all(|metric| {
        !matches!(
            metric.state,
            TestMetricState::Queued | TestMetricState::Running
        )
    }) {
        return None;
    }
    if column == "test" {
        return if progress
            .metrics
            .iter()
            .any(|metric| metric.state == TestMetricState::Running)
        {
            Some(TestMetricState::Running)
        } else if progress
            .metrics
            .iter()
            .any(|metric| metric.state == TestMetricState::Queued)
        {
            Some(TestMetricState::Queued)
        } else {
            None
        };
    }
    let metric = match column {
        "icmp" => TestMetric::Icmp,
        "tcp" => TestMetric::Tcp,
        "delay" | "connect" | "ttfb" | "http" => TestMetric::RealDelay,
        "download" => TestMetric::Download,
        "upload" => TestMetric::Upload,
        _ => return None,
    };
    progress
        .metrics
        .iter()
        .find(|progress| progress.metric == metric)
        .map(|progress| progress.state)
}

#[component]
pub fn TestActivityIcon(state: TestMetricState) -> Element {
    let label = match state {
        TestMetricState::Queued => "Test queued",
        TestMetricState::Running => "Testing",
        TestMetricState::Passed => "Test passed",
        TestMetricState::Failed => "Test failed",
        TestMetricState::Skipped => "Test skipped",
    };
    rsx! {
        Tooltip {
            span { class: if state == TestMetricState::Failed { "inline-flex text-destructive [&_svg]:size-4" } else { "inline-flex text-muted-foreground [&_svg]:size-4" }, role: "img", tabindex: "0", aria_label: label,
                match state {
                    TestMetricState::Queued => rsx! { Clock {} },
                    TestMetricState::Running => rsx! { Spinner {} },
                    TestMetricState::Passed => rsx! { CircleCheck {} },
                    TestMetricState::Failed => rsx! { CircleX {} },
                    TestMetricState::Skipped => rsx! { CircleMinus {} },
                }
            }
            TooltipContent { "{label}" }
        }
    }
}

#[cfg(test)]
mod tests;
