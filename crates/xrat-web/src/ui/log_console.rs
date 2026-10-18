use crate::ui::icon_button::IconButton;
mod parser;
#[cfg(test)]
mod tests;

use super::{
    button::ButtonVariant,
    form_select::{FormSelect, FormSelectOption},
    history::{TimeFilter, event_time, use_time_window},
    input::Input,
    label::Label,
};
use dioxus::prelude::*;
use icons::{ArrowUpToLine, Copy, Download, Pause, Play, Search, WrapText, X};
pub use parser::engine_lines;
use std::collections::{BTreeSet, HashMap};
use std::hash::{Hash, Hasher};

#[derive(Clone, Default, PartialEq)]
pub struct LogLine {
    pub time: Option<f64>,
    pub timestamp: String,
    pub level: String,
    pub source: String,
    pub message: String,
    pub text: String,
}
pub fn event_line(event: &xrat_web::types::EventRow) -> LogLine {
    let (source, message) = event
        .message
        .split_once(" / ")
        .map(|(source, rest)| (source.to_string(), rest.to_string()))
        .unwrap_or_else(|| ("XRAT".into(), event.message.clone()));
    LogLine {
        time: event_time(&event.created_at),
        timestamp: event.created_at.clone(),
        level: parser::severity(&event.level).unwrap_or("unknown").into(),
        source,
        message,
        text: format!(
            "[{}] [{}] {}",
            event.created_at,
            event.level.to_uppercase(),
            event.message
        ),
    }
}
#[component]
pub fn LogConsole(
    lines: Vec<LogLine>,
    id: String,
    loading: bool,
    limit: String,
    empty_hint: String,
    on_reload: EventHandler,
) -> Element {
    let window = use_time_window("0.25");
    let level = use_signal(|| "all".to_string());
    let source = use_signal(|| "all".to_string());
    let mut query = use_signal(String::new);
    let mut unknown = use_signal(|| false);
    let mut wrap = use_signal(|| true);
    let mut paused = use_signal(|| None::<(Vec<LogLine>, f64)>);
    let snapshot = paused();
    let now = snapshot
        .as_ref()
        .map(|(_, now)| *now)
        .unwrap_or_else(|| js_sys::Date::now() / 1000.0);
    let mut ordered = snapshot
        .as_ref()
        .map(|(lines, _)| lines.clone())
        .unwrap_or_else(|| lines.clone());
    newest_first(&mut ordered);
    let sources = ordered
        .iter()
        .map(|line| line.source.clone())
        .collect::<BTreeSet<_>>();
    let filtered = ordered
        .iter()
        .filter(|line| {
            matches_filters(line, &query(), &level(), &source())
                && line
                    .time
                    .map(|time| window.contains_at(time, now))
                    .unwrap_or(unknown())
        })
        .collect::<Vec<_>>();
    let unknown_count = ordered.iter().filter(|line| line.time.is_none()).count();
    let entries = keyed_entries(&filtered);
    let output = filtered
        .iter()
        .map(|line| line.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let download = format!(
        "data:text/plain;charset=utf-8,{}",
        js_sys::encode_uri_component(&output)
            .as_string()
            .unwrap_or_default()
    );
    let copy_id = format!("{id}-copy");
    let errors = filtered.iter().filter(|line| line.level == "error").count();
    let warnings = filtered.iter().filter(|line| line.level == "warn").count();
    rsx! {
        div { class: "log-viewer",
            div { class: "log-viewer-topbar",
                div {
                    class: "log-live-state",
                    "data-paused": snapshot.is_some().to_string(),
                    span { class: "log-live-dot" }
                    if snapshot.is_some() {
                        "Paused"
                    } else {
                        "Live · updates every 3s"
                    }
                }
                div { class: "log-viewer-actions",
                    IconButton {
                        label: if snapshot.is_some() { "Resume live updates" } else { "Pause live updates" },
                        variant: ButtonVariant::Ghost,
                        onclick: move |_| {
                            if paused.peek().is_some() {
                                paused.set(None);
                            } else {
                                paused.set(Some((lines.clone(), js_sys::Date::now() / 1000.0)));
                            }
                        },
                        if snapshot.is_some() {
                            Play {}
                        } else {
                            Pause {}
                        }
                    }
                    IconButton {
                        label: "Reload",
                        variant: ButtonVariant::Ghost,
                        disabled: loading,
                        onclick: move |_| on_reload.call(()),
                        icons::RefreshCw {}
                    }
                    IconButton {
                        label: if wrap() { "Disable line wrapping" } else { "Enable line wrapping" },
                        variant: ButtonVariant::Ghost,
                        class: if wrap() { "log-control-active" } else { "" },
                       onclick: move |_| wrap.toggle(),
                        WrapText {}
                    }
                    IconButton {
                        label: "Copy logs",
                        variant: ButtonVariant::Ghost,
                        disabled: filtered.is_empty(),
                        "data-copy-textarea": copy_id.clone(),
                        Copy {}
                    }
                    IconButton {
                        label: "Download logs",
                        variant: ButtonVariant::Ghost,
                        href: download,
                        download: format!("{id}.log"),
                        Download {}
                    }
                }
            }
            div { class: "console-toolbar log-filter-toolbar log-filter-row",
                div { class: "log-search",
                    Search {}
                    Input {
                        aria_label: "Search log output",
                        placeholder: "Search messages, sources, or timestamps…",
                        value: query(),
                        oninput: move |event: FormEvent| query.set(event.value()),
                    }
                    if !query().is_empty() {
                        IconButton {
                            label: "Clear log search",
                            variant: ButtonVariant::Ghost,
                            size: super::button::ButtonSize::IconXs,
                            onclick: move |_| query.set(String::new()),
                            X {}
                        }
                    }
                }
                TimeFilter { window, id: format!("{id}-time") }
                div {
                    Label { html_for: format!("{id}-level"), "Level" }
                    FormSelect {
                        id: format!("{id}-level"),
                        value: level,
                        options: [
                            ("all", "All levels"),
                            ("error", "Errors"),
                            ("warn", "Warnings"),
                            ("info", "Info"),
                            ("debug", "Debug"),
                            ("trace", "Trace"),
                            ("unknown", "Unclassified"),
                        ]
                            .iter()
                            .map(|(value, label)| FormSelectOption::new(value, label))
                            .collect::<Vec<_>>(),
                    }
                }
                div {
                    Label { html_for: format!("{id}-source"), "Source" }
                    FormSelect {
                        id: format!("{id}-source"),
                        value: source,
                        options: std::iter::once(FormSelectOption::new("all", "All sources"))
                            .chain(sources.iter().map(|source| FormSelectOption::new(source, source)))
                            .collect::<Vec<_>>(),
                    }
                }
                if unknown_count > 0 {
                    label { class: "log-untimed-toggle",
                        input {
                            r#type: "checkbox",
                            checked: unknown(),
                            onchange: move |event: FormEvent| unknown.set(event.checked()),
                        }
                        "Include {unknown_count} untimed"
                    }
                }
            }
            div { class: "log-console-shell",
                div { class: "log-console-heading",
                    span { "Output" }
                    div {
                        span { class: "log-summary-error", "{errors} errors" }
                        span { class: "log-summary-warn", "{warnings} warnings" }
                        IconButton {
                            label: "Jump to newest log entries",
                            variant: ButtonVariant::Ghost,
                            size: super::button::ButtonSize::IconXs,
                            "data-log-top": id.clone(),
                            ArrowUpToLine {}
                        }
                    }
                }
                div {
                    id: id.clone(),
                    class: "log-output",
                    "data-wrap": wrap().to_string(),
                    role: "region",
                    aria_label: "Read-only log output, newest entries first",
                    tabindex: "0",
                    div { class: "log-columns-head", aria_hidden: "true",
                        span { "Local time" }
                        span { "Level" }
                        span { "Source" }
                        span { "Message" }
                    }
                    if filtered.is_empty() {
                        div { class: "log-output-empty",
                            p {
                                if ordered.is_empty() {
                                    "No logs yet"
                                } else {
                                    "No matching logs"
                                }
                            }
                            span {
                                if ordered.is_empty() {
                                    "{empty_hint}"
                                } else if unknown_count > 0 && !unknown() {
                                    "Try another time range or enable untimed entries above."
                                } else {
                                    "Try a wider time range, another level, or a different search."
                                }
                            }
                        }
                    }
                    for (key, line) in entries {
                        div { class: "log-entry", key: "{key}",
                            time {
                                class: "log-entry-time",
                                title: format!("Recorded: {}", line.timestamp),
                                {display_timestamp(line)}
                            }
                            span {
                                class: "log-entry-level",
                                "data-level": line.level.clone(),
                                "{level_label(&line.level)}"
                            }
                            span { class: "log-entry-source", "{line.source}" }
                            pre { class: "log-entry-message", "{line.message}" }
                        }
                    }
                }
            }
            textarea {
                id: copy_id,
                class: "sr-only",
                readonly: true,
                tabindex: "-1",
                aria_hidden: "true",
                value: output,
            }
            div { class: "console-footnote log-viewer-footnote",
                span { "{filtered.len()} of {ordered.len()} entries · newest first · read-only" }
                span { "{limit}" }
            }
        }
    }
}
fn keyed_entries<'a>(lines: &[&'a LogLine]) -> Vec<(u64, &'a LogLine)> {
    let mut occurrences = HashMap::<&str, usize>::new();
    lines
        .iter()
        .map(|line| {
            let count = occurrences.entry(&line.text).or_default();
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            line.text.hash(&mut hasher);
            count.hash(&mut hasher);
            *count += 1;
            (hasher.finish(), *line)
        })
        .collect()
}

fn matches_filters(line: &LogLine, query: &str, level: &str, source: &str) -> bool {
    (level == "all" || line.level == level)
        && (source == "all" || line.source == source)
        && format!(
            "{} {} {} {} {}",
            line.timestamp,
            display_timestamp(line),
            line.level,
            line.source,
            line.message
        )
        .to_lowercase()
        .contains(&query.trim().to_lowercase())
}

fn display_timestamp(line: &LogLine) -> String {
    line.time
        .and_then(super::history::local_log_timestamp)
        .unwrap_or_else(|| {
            if line.timestamp.is_empty() {
                "—".into()
            } else {
                line.timestamp.replace('T', " ")
            }
        })
}
fn level_label(level: &str) -> &str {
    match level {
        "error" => "ERROR",
        "warn" => "WARN",
        "info" => "INFO",
        "debug" => "DEBUG",
        "trace" => "TRACE",
        _ => "OTHER",
    }
}
fn newest_first(lines: &mut [LogLine]) {
    lines.sort_by(|left, right| match (left.time, right.time) {
        (Some(left), Some(right)) => right.total_cmp(&left),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });
}
