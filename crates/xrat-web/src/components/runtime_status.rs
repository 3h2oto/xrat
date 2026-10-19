use crate::ui::{
    spinner::Spinner,
    tooltip::{Tooltip, TooltipContent},
};
use dioxus::prelude::*;
use icons::{CircleCheck, CircleHelp, CirclePlay, CircleStop, CircleX, Clock};

#[derive(Clone, Copy, Default, PartialEq)]
enum StatusTone {
    #[default]
    Neutral,
    Positive,
    Negative,
}

#[component]
fn StatusIndicator(
    text: String,
    stale: bool,
    #[props(default)] tone: StatusTone,
    children: Element,
) -> Element {
    let label = if stale {
        format!("{text} · Updates paused")
    } else {
        text
    };
    let color = if stale {
        "text-muted-foreground"
    } else {
        match tone {
            StatusTone::Neutral => "text-muted-foreground",
            StatusTone::Positive => "text-primary",
            StatusTone::Negative => "text-destructive",
        }
    };
    rsx! {
        Tooltip { class: format!("runtime-status inline-flex m-0 {color}"), tabindex: "0", role: "img", aria_label: label.clone(),
            span { class: "inline-flex items-center justify-center gap-1.5 tabular-nums [&_svg]:size-3.5", aria_hidden: "true", {children} }
            TooltipContent { "{label}" }
        }
    }
}

#[component]
pub fn DaemonStatus(running: Option<bool>, stale: bool) -> Element {
    rsx! {
        StatusIndicator {
            text: match running { Some(true) => "Daemon running", Some(false) => "Daemon stopped", None => "Daemon unavailable" },
            tone: if running == Some(true) { StatusTone::Positive } else { StatusTone::Neutral },
            stale,
            match running {
                Some(true) => rsx! { CirclePlay {} },
                Some(false) => rsx! { CircleStop {} },
                None => rsx! { CircleHelp {} },
            }
        }
    }
}

#[component]
pub fn JobsStatus(running: usize, queued: usize, stale: bool) -> Element {
    let idle = running == 0 && queued == 0;
    rsx! {
        StatusIndicator {
            text: if idle { "Jobs idle".into() } else { format!("Jobs: {running} running · {queued} queued") },
            tone: if running > 0 { StatusTone::Positive } else { StatusTone::Neutral },
            stale,
            if idle { CircleCheck {} }
            if running > 0 { Spinner { class: if stale { "animate-none" } else { "" } } span { "{running}" } }
            if queued > 0 { Clock {} span { "{queued}" } }
        }
    }
}

#[component]
pub fn HealthStatus(available: Option<bool>, label: String, stale: bool) -> Element {
    let state = match available {
        Some(true) => "available",
        Some(false) => "unavailable",
        None => "not checked",
    };
    rsx! {
        StatusIndicator {
            text: format!("{label}: {state}"), stale,
            tone: match available { Some(true) => StatusTone::Positive, Some(false) => StatusTone::Negative, None => StatusTone::Neutral },
            match available {
                Some(true) => rsx! { CircleCheck {} },
                Some(false) => rsx! { CircleX {} },
                None => rsx! { CircleHelp {} },
            }
        }
    }
}

#[cfg(test)]
mod tests;
