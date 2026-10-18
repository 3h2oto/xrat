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
                class: "data-sort",
                onclick: move |_| state.write().sort_by(&column),
                "{label}"
                if active {
                    if state().descending {
                        ArrowDown {}
                    } else {
                        ArrowUp {}
                    }
                } else {
                    ArrowUpDown { class: "data-sort-idle" }
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
            DropdownMenuContent { class: "data-columns-menu",
                DropdownMenuLabel { "Visible columns" }
                p { class: "data-columns-help",
                    if subscriptions {
                        "Name, config count, and actions stay visible."
                    } else {
                        "Name, state, and actions stay visible."
                    }
                }
                for column in columns {
                    button {
                        key: "{column.key}",
                        r#type: "button",
                        role: "menuitemcheckbox",
                        tabindex: "-1",
                        aria_checked: state().columns.contains(column.key),
                        class: "data-column-option",
                        onclick: move |_| {
                            let mut view = state.write();
                            if !view.columns.remove(column.key) {
                                view.columns.insert(column.key.into());
                            } else if view.sort == column.key {
                                view.sort_by("name");
                            }
                        },
                        span { class: "data-column-check",
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
        div { class: "data-search",
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
        div { class: "data-filter",
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
        div { class: "data-filter data-latency-filter",
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
                    class: "data-test-icon",
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
        span { class: if matches!(value, CellValue::Number(..) | CellValue::Probe { .. }) { "data-number" } else { "data-text" },
            if text.is_empty() {
                span { class: "muted", "—" }
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
        div { class: "data-pagination",
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
        div { class: "data-view-actions",
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
