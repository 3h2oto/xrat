use super::{bulk::*, controls::*, state::*, values::*};
use crate::pages::run;
use crate::ui::icon_button::IconButton;
use crate::ui::{
    button::ButtonVariant, card::Card, dialog::*, dropdown_menu::*, input::Input, label::Label,
    spinner::Spinner, states::EmptyState, table::*,
};
use crate::{Notice, Session, api};
use dioxus::prelude::*;
use icons::{EllipsisVertical, Pencil, RefreshCw, Trash2};
use std::collections::BTreeSet;
use xrat_web::types::{Action, BulkOperation, Job, SubscriptionRow};

#[component]
pub fn SubscriptionTable(
    subscriptions: Vec<SubscriptionRow>,
    operator: bool,
    loading: bool,
    on_reload: EventHandler,
    on_import: EventHandler,
) -> Element {
    let Session(session) = use_context();
    let Notice(mut notice) = use_context();
    let mut state = use_table_state(true, "/ui/subscriptions");
    let mut selected = use_signal(BTreeSet::<i64>::new);
    let mut deleting = use_signal(|| None::<i64>);
    let mut renaming = use_signal(|| None::<SubscriptionRow>);
    let mut name = use_signal(String::new);
    let mut saving = use_signal(|| false);
    let filter_key = state().filter_key();
    use_effect(use_reactive!(|(filter_key,)| {
        let _ = &filter_key;
        selected.write().clear();
    }));
    let filtered = filtered_subscriptions(&subscriptions, &state());
    let valid_ids = filtered.iter().map(|row| row.id).collect::<BTreeSet<_>>();
    use_effect(use_reactive!(|(valid_ids,)| {
        if selected.peek().iter().any(|id| !valid_ids.contains(id)) {
            selected.write().retain(|id| valid_ids.contains(id));
        }
    }));
    let columns = SUB_COLUMNS
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
    let sources = subscriptions
        .iter()
        .map(|row| row.source.clone())
        .collect::<BTreeSet<_>>();
    let mut export = vec![
        std::iter::once("Name".into())
            .chain(std::iter::once("Configs".into()))
            .chain(columns.iter().map(|col| col.label.into()))
            .collect::<Vec<_>>(),
    ];
    export.extend(
        filtered
            .iter()
            .filter(|row| selected().is_empty() || selected().contains(&row.id))
            .map(|row| {
                std::iter::once(row.name.clone())
                    .chain(std::iter::once(row.config_count.to_string()))
                    .chain(
                        columns
                            .iter()
                            .map(|col| subscription_value(row, col.key).text()),
                    )
                    .collect()
            }),
    );
    let operations = vec![
        (
            BulkOperation::RefreshSubscriptions,
            filtered
                .iter()
                .filter(|row| selected().contains(&row.id) && row.source == "url")
                .map(|row| row.id)
                .collect(),
        ),
        (
            BulkOperation::DeleteSubscriptions,
            filtered
                .iter()
                .filter(|row| selected().contains(&row.id))
                .map(|row| row.id)
                .collect(),
        ),
    ];
    rsx! {
        Card { class: "data-table-panel",
            div { class: "data-page-heading",
                div {
                    h2 { "Subscriptions" }
                    p { "Manage your sources and the configurations they provide." }
                }
                if operator {
                    div { class: "data-primary-actions",
                        IconButton {
                            label: "Refresh all subscriptions",
                            variant: ButtonVariant::Outline,
                            disabled: subscriptions.is_empty() || !subscriptions.iter().any(|row| row.source == "url"),
                            onclick: move |_| run(Action::Refresh { id: None }),
                            RefreshCw {}
                        }
                        IconButton {
                            label: "Add subscription",
                            onclick: move |_| on_import.call(()),
                            icons::Plus {}
                        }
                    }
                }
            }
            if subscriptions.is_empty() {
                EmptyState {
                    title: "Wow, so empty 👀",
                    description: "Add a subscription above, or paste its URL to get started.",
                }
                if operator {
                    p {
                        class: "import-paste-hint",
                        "data-import-paste": "true",
                        "Press "
                        kbd { "⌘V" }
                        " / "
                        kbd { "Ctrl+V" }
                        " to paste a subscription."
                    }
                }
            } else {
                div { class: "data-toolbar subscriptions-toolbar",
                    SearchBox { state, subscriptions: true }
                    FilterSelect {
                        label: "Source",
                        field: "source",
                        state,
                        options: std::iter::once(("".into(), "All sources".into()))
                            .chain(sources.into_iter().map(|value| (value.clone(), source_label(&value))))
                            .collect::<Vec<_>>(),
                    }
                    ViewActions {
                        state,
                        subscriptions: true,
                        csv: csv(&export),
                        loading,
                        on_reload,
                    }
                }
                if operator {
                    BulkActions {
                        selected,
                        operations,
                        matching: filtered.iter().map(|row| row.id).collect::<Vec<_>>(),
                    }
                }
                if filtered.is_empty() {
                    EmptyState {
                        title: "No matching subscriptions",
                        description: "Adjust your filters or reset the table view.",
                    }
                } else {
                    TableWrapper {
                        class: "data-table-wrapper",
                        aria_label: "Subscriptions table, scroll for more rows or columns",
                        Table { class: "data-table",
                            TableHeader {
                                TableRow {
                                    if operator {
                                        TableHead { class: "data-selection-cell",
                                            input {
                                                r#type: "checkbox",
                                                checked: all_checked,
                                                "aria-checked": if some_checked && !all_checked { "mixed" } else if all_checked { "true" } else { "false" },
                                                aria_label: "Select all subscriptions on this page",
                                                onchange: {
                                                    let ids = page_ids.clone();
                                                    move |_| toggle_page(&mut selected.write(), &ids)
                                                },
                                            }
                                        }
                                    }
                                    SortHead {
                                        column: "name",
                                        label: "Name",
                                        state,
                                    }
                                    SortHead {
                                        column: "count",
                                        label: "Configs",
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
                                    TableHead { "Actions" }
                                }
                            }
                            TableBody {
                                for row in shown {
                                    TableRow {
                                        key: "{row.id}",
                                        data_state: if selected().contains(&row.id) { "selected" } else { "" },
                                        if operator {
                                            TableCell { class: "data-selection-cell",
                                                input {
                                                    r#type: "checkbox",
                                                    checked: selected().contains(&row.id),
                                                    aria_label: format!("Select {}", row.name),
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
                                            a {
                                                class: "data-row-name data-row-link",
                                                href: format!("/ui/configs?subscription={}", row.id),
                                                "{row.name}"
                                            }
                                        }
                                        TableCell {
                                            span { class: "data-number", "{row.config_count}" }
                                        }
                                        for column in &columns {
                                            TableCell { key: "{column.key}",
                                                Cell {
                                                    value: subscription_value(row, column.key),
                                                    date: matches!(column.key, "created" | "updated"),
                                                }
                                            }
                                        }
                                        TableCell {
                                            div { class: "data-row-actions",
                                                if operator {
                                                    IconButton {
                                                        label: format!("Refresh subscription {}", row.name),
                                                        variant: ButtonVariant::Ghost,
                                                        disabled: row.source != "url",
                                                        onclick: {
                                                            let id = row.id;
                                                            move |_| run(Action::Refresh { id: Some(id) })
                                                        },
                                                        RefreshCw {}
                                                    }
                                                    DropdownMenu { align: DropdownMenuAlign::End,
                                                        DropdownMenuTrigger { as_child: true,
                                                            IconButton {
                                                                label: format!("Actions for subscription {}", row.name),
                                                                variant: ButtonVariant::Ghost,
                                                                EllipsisVertical {}
                                                            }
                                                        }
                                                        DropdownMenuContent {
                                                            DropdownMenuAction { href: format!("/ui/configs?subscription={}", row.id),
                                                                "View configs"
                                                            }
                                                            DropdownMenuAction {
                                                                onclick: {
                                                                    let row = row.clone();
                                                                    move |_| {
                                                                        name.set(row.name.clone());
                                                                        renaming.set(Some(row.clone()));
                                                                    }
                                                                },
                                                                Pencil {}
                                                                "Rename"
                                                            }
                                                            DropdownMenuAction {
                                                                variant: DropdownMenuActionVariant::Destructive,
                                                                onclick: {
                                                                    let id = row.id;
                                                                    move |_| deleting.set(Some(id))
                                                                },
                                                                Trash2 {}
                                                                "Delete"
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    IconButton {
                                                        label: "View configurations",
                                                        variant: ButtonVariant::Ghost,
                                                        href: format!("/ui/configs?subscription={}", row.id),
                                                        icons::ListChecks {}
                                                    }
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
            if let Some(id) = deleting() {
                crate::ui::alert_dialog::AlertDialog {
                    crate::ui::alert_dialog::AlertDialogContent {
                        initial_open: true,
                        on_close: move |_| deleting.set(None),
                        crate::ui::alert_dialog::AlertDialogHeader {
                            crate::ui::alert_dialog::AlertDialogTitle { "Delete subscription?" }
                            crate::ui::alert_dialog::AlertDialogDescription {
                                "This permanently deletes the subscription and its stored configurations. This cannot be undone."
                            }
                        }
                        crate::ui::alert_dialog::AlertDialogFooter {
                            IconButton {
                                label: "Cancel",
                                variant: ButtonVariant::Outline,
                                onclick: move |_| deleting.set(None),
                                icons::X {}
                            }
                            IconButton {
                                label: "Delete subscription",
                                variant: ButtonVariant::Destructive,
                                onclick: move |_| {
                                    run(Action::DeleteSubscription { id });
                                    deleting.set(None);
                                },
                                Trash2 {}
                            }
                        }
                    }
                }
            }
            if let Some(row) = renaming() {
                Dialog {
                    DialogContent {
                        class: "subscription-rename-dialog",
                        initial_open: true,
                        show_close_button: false,
                        on_close: move |_| renaming.set(None),
                        DialogHeader { class: "subscription-rename-heading",
                            span { class: "subscription-rename-icon", Pencil {} }
                            div {
                                DialogTitle { "Rename subscription" }
                                DialogDescription { "Give this source a name you can recognize." }
                            }
                        }
                        form {
                            class: "subscription-rename-form",
                            onsubmit: move |event| {
                                event.prevent_default();
                                if *saving.peek() || name().trim().is_empty() {
                                    return;
                                }
                                saving.set(true);
                                let request = Action::RenameSubscription {
                                    id: row.id,
                                    name: name().trim().into(),
                                };
                                spawn(async move {
                                    let csrf = session().map(|me| me.csrf_token).unwrap_or_default();
                                    match api::post::<_, Job>("actions", &request, &csrf).await {
                                        Ok(_) => {
                                            renaming.set(None);
                                            notice
                                                .set(Some(crate::Feedback::job_started("rename_subscription")));
                                        }
                                        Err(error) => notice.set(Some(crate::Feedback::error(error))),
                                    };
                                    saving.set(false);
                                });
                            },
                            crate::ui::field::Field {
                                Label { html_for: "subscription-name", "Subscription name" }
                                Input {
                                    id: "subscription-name",
                                    value: name(),
                                    required: true,
                                    disabled: saving(),
                                    oninput: move |event: FormEvent| name.set(event.value()),
                                }
                            }
                            DialogFooter { class: "subscription-rename-actions",
                                IconButton {
                                    label: "Cancel",
                                    variant: ButtonVariant::Ghost,
                                    onclick: move |_| renaming.set(None),
                                    icons::X {}
                                }
                                IconButton {
                                    label: if saving() { "Saving name" } else { "Save name" },
                                    button_type: "submit",
                                    disabled: saving() || name().trim().is_empty(),
                                    if saving() {
                                        Spinner {}
                                    } else {
                                        icons::Check {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
