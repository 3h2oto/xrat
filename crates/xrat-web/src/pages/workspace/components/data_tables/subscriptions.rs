use super::{bulk::*, controls::*, state::*, values::*};
use crate::api::actions::run;
use crate::components::states::EmptyState;
use crate::ui::icon_button::IconButton;
use crate::ui::{
    button::ButtonVariant, card::Card, dialog::*, dropdown_menu::*, input::Input, label::Label,
    spinner::Spinner, table::*,
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
        ["Ref".into(), "Name".into()]
            .into_iter()
            .chain(std::iter::once("Configs".into()))
            .chain(columns.iter().map(|col| col.label.into()))
            .collect::<Vec<_>>(),
    ];
    export.extend(
        filtered
            .iter()
            .filter(|row| selected().is_empty() || selected().contains(&row.id))
            .map(|row| {
                [row.reference.clone(), row.name.clone()]
                    .into_iter()
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
        Card { class: "data-table-panel min-w-0 max-w-full rounded-[9px] shadow-none border-[color:color-mix(in_srgb,_var(--border)_50%,_var(--panel))] p-5 gap-0 narrow:pr-3.5 narrow:pb-3.5 narrow:pt-3.5 narrow:pl-3.5",
            div { class: "data-page-heading flex items-center justify-between flex-wrap mb-5.5 gap-3.5 [&_h2]:text-[17px] [&_h2]:font-semibold [&_h2]:m-0 [&_p]:mt-[5px] [&_p]:mb-0 [&_p]:text-[12px] [&_p]:leading-[1.7] [&_p]:text-muted-foreground [&_p]:mx-0",
                div {
                    h2 { "Subscriptions" }
                    p { "Manage your sources and the configurations they provide." }
                }
                if operator {
                    div { class: "data-primary-actions flex items-center gap-[7px] narrow:flex-wrap",
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
                        class: "import-paste-hint mt-[-8px] mb-6 text-center text-muted-foreground text-[12px] mx-0 [&_kbd]:inline-block [&_kbd]:border [&_kbd]:border-solid [&_kbd]:border-border [&_kbd]:rounded-[4px] [&_kbd]:[font:inherit] [&_kbd]:text-[11px] [&_kbd]:py-0.5 [&_kbd]:px-[5px]",
                        "data-import-paste": "true",
                        "Press "
                        kbd { "⌘V" }
                        " / "
                        kbd { "Ctrl+V" }
                        " to paste a subscription."
                    }
                }
            } else {
                div { class: "data-toolbar subscriptions-toolbar flex items-end justify-start flex-wrap mb-3.5 gap-3 [&_>_.data-filter]:flex-[0_0_140px] [&_>_.data-view-actions]:ml-auto",
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
                    }
                }
                if filtered.is_empty() {
                    EmptyState {
                        title: "No matching subscriptions",
                        description: "Adjust your filters or reset the table view.",
                    }
                } else {
                    TableWrapper {
                        class: "data-table-wrapper max-h-[min(50dvh,_36rem)] w-full min-w-0 overflow-auto overscroll-contain [scrollbar-gutter:stable] border-[color:color-mix(in_srgb,_var(--border)_40%,_transparent)]",
                        aria_label: "Subscriptions table, scroll for more rows or columns",
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
                                                aria_label: "Select all subscriptions on this page",
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
                                            code { "{row.reference}" }
                                        }
                                        TableCell {
                                            a {
                                                class: "data-row-name data-row-link text-[12px] font-medium min-w-40 max-w-70 overflow-hidden [text-overflow:ellipsis] block [&:hover]:underline",
                                                href: format!("/ui/configs?subscription={}", row.reference),
                                                "{row.name}"
                                            }
                                        }
                                        TableCell {
                                            span { class: "data-number tabular-nums font-mono text-[11px]",
                                                "{row.config_count}"
                                            }
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
                                            div { class: "data-row-actions flex items-center gap-[5px]",
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
                                                            DropdownMenuAction { href: format!("/ui/configs?subscription={}", row.reference),
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
                                                        href: format!("/ui/configs?subscription={}", row.reference),
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
                        class: "subscription-rename-dialog w-[min(420px,_calc(100vw_-_32px))] max-w-[calc(100vw_-_32px)] rounded-[10px] border-[color:color-mix(in_srgb,_var(--border)_55%,_var(--panel))] bg-card p-5.5",
                        initial_open: true,
                        show_close_button: false,
                        on_close: move |_| renaming.set(None),
                        DialogHeader { class: "subscription-rename-heading flex-row items-start text-left gap-3 [&_h3]:mt-px [&_h3]:mb-1.5 [&_h3]:text-[17px] [&_h3]:leading-[1.4] [&_h3]:mx-0 [&_p]:text-[12px] [&_p]:leading-[1.6] [&_p]:m-0",
                            span { class: "subscription-rename-icon grid place-items-center size-8.5 shrink-0 rounded-[8px] bg-accent text-primary [&_svg]:size-4",
                                Pencil {}
                            }
                            div {
                                DialogTitle { "Rename subscription" }
                                DialogDescription { "Give this source a name you can recognize." }
                            }
                        }
                        form {
                            class: "subscription-rename-form grid mt-5.5 gap-4.5 [&_label]:text-[12px] [&_input]:h-8.5 [&_input]:text-[13px]",
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
                            DialogFooter { class: "subscription-rename-actions flex-row justify-end pt-3.5 border-t border-solid border-t-[color:color-mix(in_srgb,_var(--border)_30%,_transparent)] gap-2",
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
