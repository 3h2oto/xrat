use super::{state::*, values::CellValue};
use crate::ui::icon_button::IconButton;
use crate::ui::{
    button::ButtonVariant,
    dropdown_menu::*,
    input::{Input, InputType},
    label::Label,
    select::*,
    spinner::Spinner,
    tooltip::*,
};
use dioxus::prelude::*;
use icons::{
    ArrowDown, ArrowUp, ArrowUpDown, Check, ChevronLeft, ChevronRight, CircleCheck, CircleHelp,
    CircleX, Columns3, Download, RotateCcw, Search,
};

#[component]
pub fn SortHead(column: String, label: String, mut state: Signal<TableState>) -> Element {
    let active = state().sort == column;
    rsx! {
        th { "aria-sort": if active { if state().descending { "descending" } else { "ascending" } } else { "none" },
            button {
                r#type: "button",
                class: "data-sort inline-flex items-center bg-transparent border-0 text-inherit text-[11px] py-[3px] px-0 gap-[7px] [&_svg]:size-3 [&_svg]:shrink-[0] [&:hover]:text-foreground",
                onclick: move |_| state.write().sort_by(&column),
                "{label}"
                if active {
                    if state().descending {
                        ArrowDown {}
                    } else {
                        ArrowUp {}
                    }
                } else {
                    ArrowUpDown { class: "data-sort-idle opacity-35" }
                }
            }
        }
    }
}
#[component]
pub fn ColumnPicker(mut state: Signal<TableState>, subscriptions: bool) -> Element {
    let columns = if subscriptions {
        SUB_COLUMNS
    } else {
        CONFIG_COLUMNS
    };
    rsx! {
        DropdownMenu { align: DropdownMenuAlign::End,
            DropdownMenuTrigger { as_child: true,
                IconButton {
                    label: "Choose visible columns",
                    variant: ButtonVariant::Outline,
                    Columns3 {}
                }
            }
            DropdownMenuContent { class: "data-columns-menu max-h-[min(480px,_70dvh)] overflow-y-auto",
                DropdownMenuLabel { "Visible columns" }
                p { class: "data-columns-help text-muted-foreground text-[10px] leading-[1.6] pt-0 pb-[7px] m-0 px-[7px]",
                    if subscriptions {
                        "Ref, name, config count, and actions stay visible."
                    } else {
                        "Ref, name, state, and actions stay visible."
                    }
                }
                for column in columns {
                    button {
                        key: "{column.key}",
                        r#type: "button",
                        role: "menuitemcheckbox",
                        tabindex: "-1",
                        aria_checked: state().columns.contains(column.key),
                        class: concat!(
                            "data-column-option flex items-center w-full border-0 bg-transparent text-foreground text-left text-[12px] ",
                            "rounded-[4px] py-1.5 px-[7px] gap-2 [&:hover]:bg-accent [&:focus-visible]:bg-accent ",
                            "[&&[aria-checked=true]_.data-column-check]:bg-primary ",
                            "[&&[aria-checked=true]_.data-column-check]:[color:var(--primary-foreground)] ",
                            "[&&[aria-checked=true]_.data-column-check]:border-primary ",
                        ),
                        onclick: move |_| {
                            let mut view = state.write();
                            if !view.columns.remove(column.key) {
                                view.columns.insert(column.key.into());
                            } else if view.sort == column.key {
                                view.sort_by("name");
                            }
                        },
                        span { class: "data-column-check size-3.5 border border-solid border-border rounded-[3px] [&_svg]:size-3",
                            if state().columns.contains(column.key) {
                                Check {}
                            }
                        }
                        "{column.label}"
                    }
                }
            }
        }
    }
}
#[component]
pub fn SearchBox(mut state: Signal<TableState>, subscriptions: bool) -> Element {
    rsx! {
        div {
            class: concat!(
                "data-search flex relative flex-1 min-w-45 max-w-90 [@media(width<=800px)]:max-w-none ",
                "narrow:basis-full [&_>_svg]:w-3.5 [&_>_svg]:h-3.5 ",
                "[&_>_svg]:text-muted-foreground [&_>_svg]:absolute [&_>_svg]:left-2.5 [&_>_svg]:top-[50%] ",
                "[&_>_svg]:transform-[translateY(-50%)] [&_input]:h-8 [&_input]:text-[12px] [&_input]:pl-[31px]",
            ),
            Search {}
            Input {
                aria_label: if subscriptions { "Search subscriptions" } else { "Search configurations" },
                placeholder: if subscriptions { "Search subscriptions…" } else { "Search name, address, protocol…" },
                value: state().query,
                oninput: move |event: FormEvent| {
                    let mut view = state.write();
                    view.query = event.value();
                    view.page = 1;
                },
            }
        }
    }
}
#[component]
pub fn FilterSelect(
    label: String,
    field: String,
    options: Vec<(String, String)>,
    mut state: Signal<TableState>,
) -> Element {
    let value = match field.as_str() {
        "protocol" => state().protocol,
        "state" => state().state,
        "subscription" => state().subscription,
        "test" => state().test,
        "country" => state().country,
        "source" => state().source,
        _ => state().per_page.to_string(),
    };
    let id = use_hook(|| crate::hooks::use_random::use_random_id_for("table-filter"));
    let display = options
        .iter()
        .find(|(key, _)| *key == value)
        .map(|(_, label)| label.clone())
        .unwrap_or_else(|| {
            if value.is_empty() {
                "All".into()
            } else {
                value.clone()
            }
        });
    rsx! {
        div { class: concat!(
            "data-filter min-w-27.5 max-w-50 flex-1 narrow:max-w-none [&_label]:text-[10px] ",
            "[&_label]:text-muted-foreground [&_label]:mt-0 [&_label]:mb-[5px] [&_label]:mx-0 [&_input]:h-7.5 ",
            "[&_input]:text-[11px] [&&_[data-name=SelectTrigger]]:[height:30px] ",
            "[&&_[data-name=SelectTrigger]]:[font-size:11px] [&&_[data-name=SelectTrigger]]:[padding-inline:8px] ",
        ),
            Label { html_for: id.clone(), "{label}" }
            Select {
                selected_value: value,
                on_change: move |next: Option<String>| {
                    if let Some(next) = next {
                        let mut view = state.write();
                        match field.as_str() {
                            "protocol" => view.protocol = next,
                            "state" => view.state = next,
                            "subscription" => view.subscription = next,
                            "test" => view.test = next,
                            "country" => view.country = next,
                            "source" => view.source = next,
                            _ => view.per_page = next.parse().unwrap_or(25),
                        };
                        view.page = 1;
                    }
                },
                SelectTrigger { id,
                    SelectValue { display_value: display }
                }
                SelectContent {
                    SelectGroup { aria_label: label,
                        for (key, label) in options {
                            SelectOption { key: "{key}", value: key, "{label}" }
                        }
                    }
                }
            }
        }
    }
}
#[component]
pub fn LatencyFilter(mut state: Signal<TableState>) -> Element {
    rsx! {
        div { class: concat!(
            "data-filter data-latency-filter min-w-27.5 max-w-[125px] flex-1 narrow:max-w-none [&_label]:text-[10px] ",
            "[&_label]:text-muted-foreground [&_label]:mt-0 [&_label]:mb-[5px] [&_label]:mx-0 [&_input]:h-7.5 ",
            "[&_input]:text-[11px] [&&_[data-name=SelectTrigger]]:[height:30px] ",
            "[&&_[data-name=SelectTrigger]]:[font-size:11px] [&&_[data-name=SelectTrigger]]:[padding-inline:8px] ",
        ),
            Label { html_for: "max-delay", "Max delay (ms)" }
            Input {
                id: "max-delay",
                r#type: InputType::Number,
                min: "0",
                placeholder: "Any",
                value: state().max_delay,
                oninput: move |event: FormEvent| {
                    let value = event.value();
                    if value.is_empty() || value.parse::<u64>().is_ok() {
                        let mut view = state.write();
                        view.max_delay = value;
                        view.page = 1;
                    }
                },
            }
        }
    }
}
#[component]
pub fn Cell(value: CellValue, #[props(default)] date: bool) -> Element {
    let tip = use_hook(|| crate::hooks::use_random::use_random_id_for("test-result"));
    let (status, detail) = match &value {
        CellValue::Probe {
            ok: Some(false),
            detail,
            ..
        } => (Some("failed"), detail.clone()),
        CellValue::Probe {
            ok: Some(true),
            value: None,
            ..
        } => (Some("passed"), None),
        CellValue::Probe {
            ok: None,
            value: None,
            ..
        } => (Some("untested"), None),
        CellValue::Outcome { state, detail } => (Some(*state), detail.clone()),
        _ => (None, None),
    };
    if let Some(status) = status {
        let description = detail.unwrap_or_else(|| {
            match status {
                "failed" => "Test failed",
                "passed" => "Test passed",
                _ => "No test result",
            }
            .into()
        });
        return rsx! {
            Tooltip {
                span {
                    class: "data-test-icon inline-flex items-center text-muted-foreground [&_svg]:w-4 [&_svg]:h-4 [&&[data-result=failed]]:text-destructive ",
                    "data-result": status,
                    role: "img",
                    tabindex: "0",
                    aria_label: description.clone(),
                    aria_describedby: tip.clone(),
                    match status {
                        "failed" => rsx! {
                            CircleX {}
                        },
                        "passed" => rsx! {
                            CircleCheck {}
                        },
                        _ => rsx! {
                            CircleHelp {}
                        },
                    }
                }
                TooltipContent { id: tip.clone(), position: TooltipPosition::Top, "{description}" }
            }
        };
    }
    let text = value.text();
    let text = if date && !text.is_empty() {
        crate::ui::history::event_time(&text)
            .map(|time| crate::ui::history::timestamp(time as i64))
            .unwrap_or(text)
    } else {
        text
    };
    rsx! {
        span { class: if matches!(value, CellValue::Number(..) | CellValue::Probe { .. }) { "data-number tabular-nums font-mono text-[11px]" } else { "data-text block max-w-62.5 overflow-hidden [text-overflow:ellipsis]" },
            if text.is_empty() {
                span { class: "muted text-muted-foreground text-[13px] leading-[1.7]",
                    "—"
                }
            } else {
                "{text}"
            }
        }
    }
}
#[component]
pub fn Pagination(mut state: Signal<TableState>, total: usize) -> Element {
    let pages = total.div_ceil(state().per_page).max(1);
    let page = state().page.min(pages);
    rsx! {
        div {
            class: concat!(
                "data-pagination flex items-center justify-end mt-3.5 text-muted-foreground text-[11px] ",
                "[&&_.data-filter_>_[data-name=Select]]:[min-width:60px] tabular-nums gap-2 narrow:flex-wrap ",
                "[&_>_span:first-child]:mr-auto narrow:[&_>_span:first-child]:basis-full [&_.data-filter]:flex ",
                "[&_.data-filter]:items-center [&_.data-filter]:flex-none [&_.data-filter]:min-w-0 [&_.data-filter]:w-25 ",
                "[&_.data-filter]:my-0 [&_.data-filter]:mx-2 [&_.data-filter]:gap-1.5 [&_.data-filter_label]:m-0 ",
                "[&_>_button]:py-0 [&_>_button]:px-1.5 ",
            ),
            span {
                if total == 0 {
                    "0 rows"
                } else {
                    "{(page - 1) * state().per_page + 1}–{(page * state().per_page).min(total)} of {total}"
                }
            }
            FilterSelect {
                label: "Rows",
                field: "per_page",
                options: [25, 50, 100].iter().map(|n| (n.to_string(), n.to_string())).collect::<Vec<_>>(),
                state,
            }
            span { "Page {page} of {pages}" }
            IconButton {
                label: "Previous page",
                variant: ButtonVariant::Ghost,

                disabled: page <= 1,
                onclick: move |_| state.write().page = page.saturating_sub(1).max(1),
                ChevronLeft {}
            }
            IconButton {
                label: "Next page",
                variant: ButtonVariant::Ghost,

                disabled: page >= pages,
                onclick: move |_| state.write().page = page + 1,
                ChevronRight {}
            }
        }
    }
}
#[component]
pub fn ViewActions(
    mut state: Signal<TableState>,
    subscriptions: bool,
    csv: String,
    loading: bool,
    on_reload: EventHandler,
) -> Element {
    let href = format!(
        "data:text/csv;charset=utf-8,{}",
        js_sys::encode_uri_component(&csv)
            .as_string()
            .unwrap_or_default()
    );
    rsx! {
        div { class: "data-view-actions flex items-center gap-[7px] [@media(width<=800px)]:flex-wrap",
            IconButton {
                label: "Reset table view",
                variant: ButtonVariant::Ghost,
                onclick: move |_| state.set(TableState::new(subscriptions)),
                RotateCcw {}
            }
            ColumnPicker { state, subscriptions }
            IconButton {
                label: "Reload",
                variant: ButtonVariant::Outline,
                disabled: loading,
                onclick: move |_| on_reload.call(()),
                if loading {
                    Spinner {}
                } else {
                    icons::RefreshCw {}
                }
            }
            IconButton {
                label: "Export CSV",
                variant: ButtonVariant::Ghost,
                href,
                download: if subscriptions { "subscriptions.csv" } else { "configs.csv" },
                Download {}
            }
        }
    }
}
