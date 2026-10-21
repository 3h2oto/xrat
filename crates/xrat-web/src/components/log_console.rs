pub mod skeleton;
use super::history::{TimeFilter, event_time, use_time_window};
use crate::ui::icon_button::IconButton;
mod parser;
#[cfg(test)]
mod tests;

use super::clipboard::CopyButton;
use crate::ui::{
    button::ButtonVariant,
    form_select::{FormSelect, FormSelectOption},
    input::Input,
    label::Label,
    switch::Switch,
};
use dioxus::prelude::*;
use icons::{ArrowUpToLine, Download, Pause, Play, Search, WrapText, X};
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
    let mut output_element = use_signal(|| None::<std::rc::Rc<MountedData>>);
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
        div { class: "log-viewer min-w-0",
            div { class: "log-viewer-topbar flex items-center justify-between mb-3.5 gap-2 mobile:flex-wrap",
                div {
                    class: "log-live-state flex items-center text-muted-foreground text-[11px] gap-[7px] [&&[data-paused=true]_.log-live-dot]:[background:var(--muted)] ",
                    "data-paused": snapshot.is_some().to_string(),
                    span { class: "log-live-dot size-1.5 rounded-[50%] bg-primary" }
                    if snapshot.is_some() {
                        "Paused"
                    } else {
                        "Live · updates every 3s"
                    }
                }
                div { class: "log-viewer-actions flex items-center flex-wrap justify-end gap-1",
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
                        class: if wrap() { "log-control-active bg-accent text-foreground" } else { "" },
                        onclick: move |_| wrap.toggle(),
                        WrapText {}
                    }
                    CopyButton {
                        target_id: copy_id.clone(),
                        icon_only: true,
                        label: "Copy logs",
                        variant: ButtonVariant::Ghost,
                        disabled: filtered.is_empty(),
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
            div {
                class: concat!(
                    "console-toolbar log-filter-toolbar log-filter-row flex items-end flex-wrap mb-3.5 pb-0 ",
                    "[&&_[data-name=SelectTrigger]]:[height:30px] [&&_[data-name=SelectTrigger]]:[font-size:12px] ",
                    "[&&_[data-name=SelectTrigger]]:[padding:0_9px] [&&_[data-name=SelectTrigger]]:[gap:8px] ",
                    "[&&_[data-name=SelectTrigger]_svg]:[width:13px] [&&_[data-name=SelectTrigger]_svg]:[height:13px] ",
                    "justify-start gap-2.5 [&_label]:block [&_label]:text-[11px] [&_label]:font-medium ",
                    "[&_label]:text-muted-foreground [&_label]:mb-[5px] [&_.history-time-filter_>_div]:min-w-35 ",
                    "[&_>_div:not(.history-time-filter):not(.log-search)]:min-w-27.5 ",
                    "[&_>_div:not(.history-time-filter):not(.log-search)]:max-w-55 [&_>_.history-time-filter]:max-w-none ",
                    "[&_>_.log-search]:flex-1 [&_>_.log-search]:max-w-none [&_>_.log-search]:min-w-55 ",
                    "mobile:[&_>_.log-search]:basis-full ",
                ),
                div {
                    class: concat!(
                        "log-search flex relative flex-1 max-w-112.5 min-w-45 mobile:max-w-none ",
                        "mobile:basis-full [&_>_svg]:absolute [&_>_svg]:left-2.5 [&_>_svg]:top-[50%] ",
                        "[&_>_svg]:transform-[translateY(-50%)] [&_>_svg]:w-3.5 [&_>_svg]:h-3.5 ",
                        "[&_>_svg]:text-muted-foreground [&_input]:pl-8 [&_input]:pr-7.5 [&_input]:h-8 ",
                        "[&_input]:text-[12px] [&_>_button]:absolute [&_>_button]:right-1.5 [&_>_button]:top-[50%] ",
                        "[&_>_button]:transform-[translateY(-50%)] [&_>_button]:grid [&_>_button]:place-items-center ",
                        "[&_>_button]:w-5.5 [&_>_button]:h-5.5 [&_>_button]:border-0 [&_>_button]:bg-transparent ",
                        "[&_>_button]:text-muted-foreground [&_>_button]:rounded-[4px] [&_>_button:hover]:bg-accent ",
                        "[&_>_button_svg]:w-3 [&_>_button_svg]:h-3 [&_>_.icon-action]:absolute ",
                        "[&_>_.icon-action]:right-[5px] [&_>_.icon-action]:top-[50%] ",
                        "[&_>_.icon-action]:transform-[translateY(-50%)]",
                    ),
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
                            size: crate::ui::button::ButtonSize::IconXs,
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
                    div { class: "log-untimed-toggle items-center cursor-pointer flex! mt-0! mb-1.5! text-[11px]! mx-0! gap-1.5",
                        Switch {
                            id: format!("{id}-untimed"),
                            aria_label: "Include untimed entries",
                            checked: unknown(),
                            on_checked_change: move |checked| unknown.set(checked),
                        }
                        label { r#for: format!("{id}-untimed"), "Include untimed" }
                    }
                }
            }
            div { class: "log-console-shell border border-solid border-[color:#303030] bg-[color:#0c0c0c] text-[color:#d6d6d6] rounded-[7px] overflow-hidden min-w-0",
                div {
                    class: concat!(
                        "log-console-heading flex items-center justify-between border-b border-solid ",
                        "border-b-[color:#2a2a2a] text-[color:#aaa] text-[10px] py-2 px-3 gap-2.5 [&_>_div]:flex ",
                        "[&_>_div]:items-center [&_>_div]:gap-3.5 mobile:[&_>_div]:gap-[9px] ",
                        "[&_button]:inline-flex [&_button]:items-center [&_button]:border-0 ",
                        "[&_button]:text-[color:#bbb] [&_button]:bg-transparent [&_button]:text-[10px] ",
                        "[&_button]:py-0.5 [&_button]:px-0 [&_button]:gap-[5px] [&_button:hover]:text-[color:#eee] ",
                        "[&_svg]:size-3",
                    ),
                    span { "Output" }
                    div {
                        span { class: "log-summary-error text-[color:#ef9d99]", "{errors} errors" }
                        span { class: "log-summary-warn text-[color:#d6ba78]", "{warnings} warnings" }
                        IconButton {
                            label: "Jump to newest log entries",
                            variant: ButtonVariant::Ghost,
                            size: crate::ui::button::ButtonSize::IconXs,
                            onclick: move |_| async move {
                                let element = output_element.peek().clone();
                                if let Some(element) = element {
                                    let _ = element.scroll(Default::default(), ScrollBehavior::Instant).await;
                                }
                            },
                            ArrowUpToLine {}
                        }
                    }
                }
                div {
                    id: id.clone(),
                    onmounted: move |event| output_element.set(Some(event.data())),
                    class: concat!(
                        "log-output h-[clamp(220px,_48dvh,_580px)] max-h-[58dvh] min-h-40 overflow-auto ",
                        "[&&]:[--log-columns:minmax(140px,_190px)_58px_minmax(80px,_120px)_minmax(0,_1fr)] ",
                        "[&&]:[scrollbar-width:thin] [&&]:[scrollbar-color:#454545_#0c0c0c] ",
                        "[&&[data-wrap=false]_.log-entry-message]:[white-space:pre] ",
                        "[&&[data-wrap=false]_.log-entry-message]:[overflow-wrap:normal] ",
                        "[@media(max-width:_900px)]:[&&]:[--log-columns:minmax(100px,_145px)_52px_80px_minmax(0,_1fr)] ",
                        "mobile:[&&]:[--log-columns:minmax(0,_1fr)_54px_minmax(0,_1fr)] ",
                        "[&&::-webkit-scrollbar-thumb]:[background-color:#555] ",
                        "[font-family:ui-monospace,_SFMono-Regular,_Consolas,_monospace] text-[12px] leading-[1.75] [cursor:text] ",
                        "[user-select:text] [overflow-anchor:auto] [scrollbar-gutter:stable] overscroll-contain ",
                        "[&:focus-visible]:[outline:2px_solid_var(--ring)] [&:focus-visible]:[outline-offset:-2px] ",
                    ),
                    "data-wrap": wrap().to_string(),
                    role: "region",
                    aria_label: "Read-only log output, newest entries first",
                    tabindex: "0",
                    div {
                        class: concat!(
                            "log-columns-head grid grid-cols-[var(--log-columns)] items-start min-w-0 sticky top-0 z-[1] ",
                            "bg-[color:#151515] text-[color:#888] [font-family:ui-sans-serif,_system-ui,_sans-serif] ",
                            "text-[10px] border-b border-solid border-b-[color:#252525] py-1.5 px-3 gap-3 ",
                            "[@media(width<=900px)]:pr-2.5 [@media(width<=900px)]:pl-2.5 [@media(width<=900px)]:gap-2 ",
                            "mobile:[&_span:last-child]:hidden ",
                            "mobile:[&_span:nth-child(3)]:text-right",
                        ),
                        aria_hidden: "true",
                        span { "Local time" }
                        span { "Level" }
                        span { "Source" }
                        span { "Message" }
                    }
                    if filtered.is_empty() {
                        div { class: "log-output-empty text-[color:#858585] text-center [font-family:ui-sans-serif,_system-ui,_sans-serif] py-10.5 px-5 [&_p]:text-[color:#b8b8b8] [&_p]:text-[13px] [&_p]:mt-0 [&_p]:mb-[7px] [&_p]:mx-0 [&_span]:text-[11px] [&_span]:leading-[1.7]",
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
                        div {
                            class: "log-entry grid grid-cols-[var(--log-columns)] items-start min-w-0 border-b border-solid border-b-[color:#1c1c1c] py-2 px-3 gap-3 [@media(width<=900px)]:pr-2.5 [@media(width<=900px)]:pl-2.5 [@media(width<=900px)]:gap-2 [&:hover]:bg-[color:#151515]",
                            key: "{key}",
                            time {
                                class: "log-entry-time text-[color:#858585] text-[10px] whitespace-normal wrap-anywhere [font-family:inherit]",
                                title: format!("Recorded: {}", line.timestamp),
                                {display_timestamp(line)}
                            }
                            span {
                                class: concat!(
                                    "log-entry-level inline-block w-fit mt-px rounded-[3px] text-[color:#a1a1a1] border border-solid ",
                                    "border-[color:#333] text-[9px] leading-[17px] py-0 px-1 [&&[data-level=error]]:[color:#f2aaa6] ",
                                    "[&&[data-level=error]]:[border-color:#673c3a] [&&[data-level=error]]:[background:#2b1918] ",
                                    "[&&[data-level=warn]]:[color:#dbc187] [&&[data-level=warn]]:[border-color:#5e5033] ",
                                    "[&&[data-level=warn]]:[background:#261f13] [&&[data-level=info]]:[color:#a9c6d6] ",
                                    "[&&[data-level=info]]:[border-color:#354954] [&&[data-level=info]]:[background:#152027] ",
                                    "[&&[data-level=debug]]:[color:#a6a0b0] [&&[data-level=trace]]:[color:#a6a0b0] ",
                                ),
                                "data-level": line.level.clone(),
                                "{level_label(&line.level)}"
                            }
                            span { class: "log-entry-source text-[color:#a5a5a5] whitespace-normal wrap-anywhere text-[10px] mobile:text-right",
                                "{line.source}"
                            }
                            pre { class: "log-entry-message border-0 [background:none] text-[color:#d8d8d8] [font:inherit] whitespace-pre-wrap wrap-anywhere min-w-0 m-0 p-0 mobile:col-span-full",
                                "{line.message}"
                            }
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
            div { class: "console-footnote log-viewer-footnote flex justify-between mt-2 mb-0 text-[10px] text-muted-foreground flex-wrap mx-0 gap-3",
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
