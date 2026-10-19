use crate::ui::{
    spinner::Spinner,
    tooltip::{Tooltip, TooltipContent, TooltipPosition},
};
use dioxus::prelude::*;
use icons::{Ban, CircleCheck, CircleX, Clock};

#[derive(Clone, Copy, Default, PartialEq)]
pub enum JobStatusPresentation {
    #[default]
    Compact,
    Labelled,
}

#[component]
pub fn JobStatus(state: String, #[props(default)] presentation: JobStatusPresentation) -> Element {
    match presentation {
        JobStatusPresentation::Compact => rsx! { CompactStatus { state } },
        JobStatusPresentation::Labelled => rsx! { LabelledStatus { state } },
    }
}

#[component]
fn CompactStatus(state: String) -> Element {
    let tip = use_hook(|| crate::hooks::use_random::use_random_id_for("job-status"));
    rsx! {
        Tooltip { class: "job-status-tooltip inline-flex align-top m-0",
            span {
                class: concat!(
                    "job-status inline-flex size-6 justify-center items-center text-muted-foreground rounded-[4px] ",
                    "[&_svg]:size-4 [&&[data-status=succeeded]]:text-primary [&&[data-status=failed]]:text-destructive ",
                    "[&&[data-status=interrupted]]:text-destructive [&&[data-status=running]]:text-primary ",
                ),
                "data-status": state.clone(),
                role: "img",
                aria_label: state.clone(),
                aria_describedby: tip.clone(),
                tabindex: "0",
                match state.as_str() {
                    "running" => rsx! {
                        Spinner {}
                    },
                    "succeeded" => rsx! {
                        CircleCheck {}
                    },
                    "failed" | "interrupted" => rsx! {
                        CircleX {}
                    },
                    "cancelled" => rsx! {
                        Ban {}
                    },
                    _ => rsx! {
                        Clock {}
                    },
                }
            }
            TooltipContent { id: tip.clone(), position: TooltipPosition::Top, "{state}" }
        }
    }
}

#[component]
fn LabelledStatus(state: String) -> Element {
    let active = matches!(state.as_str(), "queued" | "running");
    let label = match state.as_str() {
        "queued" | "running" => "In progress",
        "succeeded" => "Complete",
        "cancelled" => "Cancelled",
        "interrupted" => "Interrupted",
        _ => "Failed",
    };
    rsx! {
        span {
            class: concat!(
                "tool-job-state inline-flex items-center text-[11px] text-muted-foreground gap-1.5 [&_svg]:w-3.5 ",
                "[&_svg]:h-3.5 [&&[data-state=failed]]:text-destructive [&&[data-state=interrupted]]:text-destructive ",
                "[&&[data-state=succeeded]]:text-foreground ",
            ),
            "data-state": state.clone(),
            if active {
                Spinner {}
            } else if state == "succeeded" {
                CircleCheck {}
            } else {
                CircleX {}
            }
            "{label}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_status_renders_labels_as_text() {
        for (state, expected) in [
            ("cancelled", "Cancelled"),
            ("interrupted", "Interrupted"),
            ("failed", "Failed"),
        ] {
            let mut dom = VirtualDom::new_with_props(
                JobStatus,
                JobStatusProps {
                    presentation: JobStatusPresentation::Labelled,
                    state: state.into(),
                },
            );
            let mutations = dom.rebuild_to_vec();
            assert!(mutations.edits.iter().any(|edit| matches!(edit, dioxus::core::Mutation::CreateTextNode { value, .. } if value == expected)));
        }
    }
}
