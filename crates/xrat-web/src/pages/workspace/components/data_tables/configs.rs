use super::{bulk::*, controls::*, state::*, values::*};
use crate::api::actions::run;
use crate::components::states::EmptyState;
use crate::pages::workspace::{ConfigActions, ConfigToggle};
use crate::ui::icon_button::IconButton;
use crate::ui::{
    badge::{Badge, BadgeVariant},
    button::ButtonVariant,
    card::Card,
    switch::Switch,
    table::*,
};
use dioxus::prelude::*;
use icons::{FlaskConical, Plus};
use std::collections::BTreeSet;
use xrat_web::types::{Action, BulkOperation, ConfigRow, SubscriptionRow};

#[component]
pub fn ConfigTable(
    configs: Vec<ConfigRow>,
    active_config: Option<i64>,
    subscriptions: Vec<SubscriptionRow>,
    operator: bool,
    loading: bool,
    on_reload: EventHandler,
    on_import: EventHandler,
    pending_delete: Signal<Option<i64>>,
    #[props(default)] tests: Vec<xrat_web::types::ConfigTestProgress>,
) -> Element {
    let mut state = use_table_state(false, "/ui/configs");
    let mut selected = use_signal(BTreeSet::<i64>::new);
    let filter_key = state().filter_key();
    use_effect(use_reactive!(|(filter_key,)| {
        let _ = &filter_key;
        selected.write().clear();
    }));
    let filtered = filtered_configs(&configs, &subscriptions, &state(), active_config);
    let valid_ids = filtered.iter().map(|row| row.id).collect::<BTreeSet<_>>();
    use_effect(use_reactive!(|(valid_ids,)| {
        if selected.peek().iter().any(|id| !valid_ids.contains(id)) {
            selected.write().retain(|id| valid_ids.contains(id));
        }
    }));
    let columns = CONFIG_COLUMNS
        .iter()
        .filter(|col| state().columns.contains(col.key))
        .copied()
        .collect::<Vec<_>>();
    let page = state()
        .page
        .min(filtered.len().div_ceil(state().per_page).max(1));
    let max_page = filtered.len().div_ceil(state().per_page).max(1);
    use_effect(use_reactive!(|(max_page,)| {
        if state().page > max_page {
            state.write().page = max_page;
        }
    }));
    let shown = filtered
        .iter()
        .skip((page - 1) * state().per_page)
        .take(state().per_page)
        .copied()
        .collect::<Vec<_>>();
    let page_ids = shown.iter().map(|row| row.id).collect::<Vec<_>>();
    let all_checked = !page_ids.is_empty() && page_ids.iter().all(|id| selected().contains(id));
    let some_checked = page_ids.iter().any(|id| selected().contains(id));
    let protocols = configs
        .iter()
        .map(|row| row.protocol.clone())
        .collect::<BTreeSet<_>>();
    let countries = configs
        .iter()
        .filter_map(|row| row.country.clone())
        .collect::<BTreeSet<_>>();
    let operations = [
        BulkOperation::Enable,
        BulkOperation::Disable,
        BulkOperation::Test,
        BulkOperation::Restore,
        BulkOperation::Delete,
    ]
    .iter()
    .map(|op| {
        (
            *op,
            filtered
                .iter()
                .filter(|row| {
                    selected().contains(&row.id)
                        && match op {
                            BulkOperation::Restore => row.deleted,
                            BulkOperation::Enable => !row.deleted && !row.enabled,
                            BulkOperation::Disable => !row.deleted && row.enabled,
                            _ => !row.deleted,
                        }
                })
                .map(|row| row.id)
                .collect(),
        )
    })
    .collect::<Vec<_>>();
    rsx! {
        Card { class: "data-table-panel min-w-0 max-w-full rounded-[9px] shadow-none border-[color:color-mix(in_srgb,_var(--border)_50%,_var(--panel))] p-5 gap-0 narrow:pr-3.5 narrow:pb-3.5 narrow:pt-3.5 narrow:pl-3.5",
            div { class: "data-page-heading flex items-center justify-between flex-wrap mb-5.5 gap-3.5 [&_h2]:text-[17px] [&_h2]:font-semibold [&_h2]:m-0 [&_p]:mt-[5px] [&_p]:mb-0 [&_p]:text-[12px] [&_p]:leading-[1.7] [&_p]:text-muted-foreground [&_p]:mx-0",
                div {
                    h2 { "Configurations" }
                    p { "Your connections, together in one place." }
                }
                if operator {
                    div { class: "data-primary-actions flex items-center gap-[7px] narrow:flex-wrap",
                        IconButton {
                            label: "Test enabled",
                            variant: ButtonVariant::Outline,
                            disabled: !configs.iter().any(|row| row.enabled && !row.deleted),
                            onclick: move |_| run(Action::Test { id: None }),
                            FlaskConical {}
                        }
                        IconButton {
                            label: "Import configs",
                            onclick: move |_| on_import.call(()),
                            Plus {}
                        }
                    }
                }
            }
            if configs.is_empty() {
                EmptyState {
                    title: "Wow, so empty 👀",
                    description: "Import a subscription or paste configuration links to begin.",
                }
                if operator {
                    p {
                        class: "import-paste-hint mt-[-8px] mb-6 text-center text-muted-foreground text-[12px] mx-0 [&_kbd]:inline-block [&_kbd]:border [&_kbd]:border-solid [&_kbd]:border-border [&_kbd]:rounded-[4px] [&_kbd]:[font:inherit] [&_kbd]:text-[11px] [&_kbd]:py-0.5 [&_kbd]:px-[5px]",
                        "data-import-paste": "true",
                        "Press "
                        kbd { "⌘V" }
                        " / "
                        kbd { "Ctrl+V" }
                        " to paste a configuration or subscription."
                    }
                }
            } else {
                div { class: "data-toolbar flex items-center justify-between flex-wrap mb-3.5 gap-3 narrow:items-stretch",
                    SearchBox { state, subscriptions: false }
                    div { class: "flex items-center flex-wrap gap-2 narrow:justify-between",
                    if operator {
                        super::sharing::ShareControls {
                            request: xrat_web::configs::sharing::ConfigShareRequest::Filtered { filter: state().share_filter() },
                            count: filtered.len(),
                            revision: filtered.iter().map(|row| format!("{}:{}", row.id, row.updated_at)).collect::<Vec<_>>().join(","),
                        }
                    }
                    ViewActions {
                        state,
                        subscriptions: false,
                        loading,
                        on_reload,
                    }
                    }
                }
                div { class: "data-filters flex flex-wrap items-end mb-5 gap-2.5",
                    FilterSelect {
                        label: "Protocol",
                        field: "protocol",
                        state,
                        options: std::iter::once(("".into(), "All protocols".into()))
                            .chain(protocols.into_iter().map(|value| (value.clone(), value)))
                            .collect::<Vec<_>>(),
                    }
                    FilterSelect {
                        label: "State",
                        field: "state",
                        state,
                        options: vec![
                            ("".into(), "All states".into()),
                            ("enabled".into(), "Enabled".into()),
                            ("disabled".into(), "Disabled".into()),
                        ],
                    }
                    FilterSelect {
                        label: "Subscription",
                        field: "subscription",
                        state,
                        options: std::iter::once(("".into(), "All subscriptions".into()))
                            .chain(std::iter::once(("none".into(), "No subscription".into())))
                            .chain(
                                subscriptions
                                    .iter()
                                    .map(|sub| (
                                        sub.reference.clone(),
                                        format!("{} · {}", sub.reference, sub.name),
                                    )),
                            )
                            .collect::<Vec<_>>(),
                    }
                    FilterSelect {
                        label: "Test result",
                        field: "test",
                        state,
                        options: vec![
                            ("".into(), "All results".into()),
                            ("passed".into(), "Passed".into()),
                            ("failed".into(), "Failed".into()),
                            ("untested".into(), "Not tested".into()),
                        ],
                    }
                    FilterSelect {
                        label: "Country",
                        field: "country",
                        state,
                        options: std::iter::once(("".into(), "All countries".into()))
                            .chain(countries.into_iter().map(|value| (value.clone(), value)))
                            .collect::<Vec<_>>(),
                    }
                    LatencyFilter { state }
                    div { class: "data-deleted-toggle flex items-center self-end min-h-8 text-[12px] text-muted-foreground cursor-pointer whitespace-nowrap m-0 gap-[7px]",
                        Switch {
                            id: "show-deleted-configs",
                            aria_label: "Show deleted configs",
                            checked: state().show_deleted,
                            on_checked_change: move |checked| {
                                let mut next = state.write();
                                next.show_deleted = checked;
                                next.page = 1;
                                if !next.show_deleted && next.state == "deleted" {
                                    next.state.clear();
                                }
                            },
                        }
                        label { r#for: "show-deleted-configs", "Show deleted configs" }
                    }
                }
                if operator {
                    BulkActions {
                        share: rsx! {
                            super::sharing::ShareControls {
                                selected: true,
                                count: selected().len(),
                                revision: filtered.iter().filter(|row| selected().contains(&row.id)).map(|row| format!("{}:{}", row.id, row.updated_at)).collect::<Vec<_>>().join(","),
                                request: xrat_web::configs::sharing::ConfigShareRequest::Selected {
                                    ids: filtered.iter().filter(|row| selected().contains(&row.id)).map(|row| row.id).collect(),
                                },
                            }
                        },
                        selected,
                        operations,
                    }
                }
                if filtered.is_empty() {
                    EmptyState {
                        title: "No matching configurations",
                        description: "Adjust your filters or reset the table view.",
                    }
                } else {
                    TableWrapper {
                        class: "data-table-wrapper max-h-[min(50dvh,_36rem)] w-full min-w-0 overflow-auto overscroll-contain [scrollbar-gutter:stable] border-[color:color-mix(in_srgb,_var(--border)_40%,_transparent)]",
                        aria_label: "Configurations table, scroll for more rows or columns",
                        Table {
                            class: concat!(
                                "data-table max-w-none text-[12px] [&_th]:text-left [&_th]:align-middle ",
                                "[&_th]:whitespace-nowrap [&_th]:h-10 [&_th]:text-[11px] [&_th]:font-medium [&_th]:py-2.5 ",
                                "[&_th]:px-3 [&_td]:text-left [&_td]:align-middle [&_td]:whitespace-nowrap [&_td]:py-2.5 ",
                                "[&_td]:px-3 [&_.data-selection-cell]:w-8.5 [&_.data-selection-cell]:px-2.5",
                            ),
                            TableHeader {
                                TableRow {
                                    if operator {
                                        TableHead { class: concat!(
                                            "data-selection-cell [&_input]:w-3.5 [&_input]:h-3.5 [&_input]:block ",
                                            "[&_input]:[accent-color:var(--primary)] [&_input]:cursor-pointer ",
                                            "[&&_input[aria-checked=mixed]]:[appearance:none] ",
                                            "[&&_input[aria-checked=mixed]]:[border:1px_solid_var(--primary)] ",
                                            "[&&_input[aria-checked=mixed]]:[border-radius:3px] ",
                                            "[&&_input[aria-checked=mixed]]:[background:linear-gradient(var(--primary),_var(--primary))_center/8px_2px_no-repeat] ",
                                        ),
                                            input {
                                                r#type: "checkbox",
                                                checked: all_checked,
                                                "aria-checked": if some_checked && !all_checked { "mixed" } else if all_checked { "true" } else { "false" },
                                                aria_label: "Select all configurations on this page",
                                                onchange: {
                                                    let ids = page_ids.clone();
                                                    move |_| toggle_page(&mut selected.write(), &ids)
                                                },
                                            }
                                        }
                                    }
                                    SortHead {
                                        column: "ref",
                                        label: "Ref",
                                        state,
                                    }
                                    SortHead {
                                        column: "name",
                                        label: "Name",
                                        state,
                                    }
                                    for column in &columns {
                                        SortHead {
                                            key: "{column.key}",
                                            column: column.key,
                                            label: column.label,
                                            state,
                                        }
                                    }
                                    SortHead {
                                        column: "state",
                                        label: "State",
                                        state,
                                    }
                                    if operator {
                                        TableHead { "Actions" }
                                    }
                                }
                            }
                            TableBody {
                                for row in shown {
                                    TableRow {
                                        key: "{row.id}",
                                        class: if active_config == Some(row.id) {
                                            "connected-config-row [&&]:bg-primary/8 [&&:hover]:bg-primary/12 [&&[data-state=selected]]:bg-primary/16 [&_td:first-child]:shadow-[inset_3px_0_0_var(--primary)]"
                                        } else { "" },
                                        data_state: if selected().contains(&row.id) { "selected" } else { "" },
                                        if operator {
                                            TableCell { class: concat!(
                                                "data-selection-cell [&_input]:w-3.5 [&_input]:h-3.5 [&_input]:block ",
                                                "[&_input]:[accent-color:var(--primary)] [&_input]:cursor-pointer ",
                                                "[&&_input[aria-checked=mixed]]:[appearance:none] ",
                                                "[&&_input[aria-checked=mixed]]:[border:1px_solid_var(--primary)] ",
                                                "[&&_input[aria-checked=mixed]]:[border-radius:3px] ",
                                                "[&&_input[aria-checked=mixed]]:[background:linear-gradient(var(--primary),_var(--primary))_center/8px_2px_no-repeat] ",
                                            ),
                                                input {
                                                    r#type: "checkbox",
                                                    checked: selected().contains(&row.id),
                                                    aria_label: format!("Select {}", config_name(row)),
                                                    onchange: {
                                                        let id = row.id;
                                                        move |event: FormEvent| {
                                                            if event.checked() {
                                                                selected.write().insert(id);
                                                            } else {
                                                                selected.write().remove(&id);
                                                            }
                                                        }
                                                    },
                                                }
                                            }
                                        }
                                        TableCell {
                                            code { "{row.reference}" }
                                        }
                                        TableCell {
                                            div { class: "data-row-name text-[12px] font-medium min-w-40 max-w-70 overflow-hidden [text-overflow:ellipsis]",
                                                if active_config == Some(row.id) {
                                                    span { class: "sr-only", "Currently connected: " }
                                                }
                                                "{config_name(row)}"
                                            }
                                        }
                                        for column in &columns {
                                            TableCell { key: "{column.key}",
                                                Cell {
                                                    value: config_value(row, column.key, &subscriptions),
                                                    date: matches!(column.key, "tested" | "updated"),
                                                    activity: crate::components::test_progress::column_state(&tests, row.id, column.key),
                                                }
                                            }
                                        }
                                        TableCell {
                                            if row.deleted {
                                                Badge { variant: BadgeVariant::Muted,
                                                    "Deleted"
                                                }
                                            } else if operator {
                                                ConfigToggle {
                                                    id: row.id,
                                                    enabled: row.enabled,
                                                }
                                            } else {
                                                Badge { variant: BadgeVariant::Outline,
                                                    "{config_state(row)}"
                                                }
                                            }
                                        }
                                        if operator {
                                            TableCell {
                                                ConfigActions {
                                                    id: row.id,
                                                    deleted: row.deleted,
                                                    revision: row.updated_at.clone(),
                                                    pending_delete,
                                                    testing: crate::components::test_progress::column_state(&tests, row.id, "test").is_some(),
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Pagination { state, total: filtered.len() }
            }
        }
    }
}
