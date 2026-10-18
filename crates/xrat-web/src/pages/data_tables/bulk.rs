use crate::ui::icon_button::IconButton;
use crate::ui::{alert_dialog::*, button::ButtonVariant, dropdown_menu::*, spinner::Spinner};
use crate::{Notice, Session, api};
use dioxus::prelude::*;
use icons::{Check, ListChecks, Trash2, X};
use std::collections::BTreeSet;
use xrat_web::types::{Action, BulkOperation, Job};

#[component]
pub fn BulkActions(
    mut selected: Signal<BTreeSet<i64>>,
    operations: Vec<(BulkOperation, Vec<i64>)>,
    matching: Vec<i64>,
) -> Element {
    let Session(session) = use_context();
    let Notice(mut notice) = use_context();
    let mut busy = use_signal(|| false);
    let mut pending = use_signal(|| None::<(BulkOperation, Vec<i64>)>);
    let submit = use_callback(move |(operation, ids): (BulkOperation, Vec<i64>)| {
        if *busy.peek() {
            return;
        }
        busy.set(true);
        pending.set(None);
        spawn(async move {
            let csrf = session().map(|me| me.csrf_token).unwrap_or_default();
            match api::post::<_, Job>("actions", &Action::Bulk { operation, ids }, &csrf).await {
                Ok(_) => {
                    selected.write().clear();
                    notice.set(Some(crate::Feedback::success(format!(
                        "{} started. Follow progress in Jobs.",
                        operation.label()
                    ))));
                }
                Err(error) => notice.set(Some(crate::Feedback::error(error))),
            }
            busy.set(false);
        });
    });
    rsx! {
        if !selected().is_empty() {
            div { class: "data-selection-bar",
                span {
                    ListChecks {}
                    "{selected().len()} selected"
                }
                if selected().len() != matching.len() {
                    IconButton {
                        label: "Select all {matching.len()} matching",
                        variant: ButtonVariant::Ghost,
                        disabled: busy() || matching.len() > 500,
                        onclick: {
                            let matching = matching.clone();
                            move |_| selected.set(matching.iter().copied().collect())
                        },
                        icons::ListChecks {}
                    }
                }
                DropdownMenu {
                    DropdownMenuTrigger { as_child: true,
                        IconButton {
                            label: "Bulk actions",
                            variant: ButtonVariant::Outline,
                            if busy() {
                                Spinner {}
                            } else {
                                Check {}
                            }
                        }
                    }
                    DropdownMenuContent {
                        DropdownMenuLabel { "Apply to eligible selected rows" }
                        for (operation, ids) in operations {
                            DropdownMenuAction {
                                disabled: busy() || ids.is_empty() || ids.len() > 500,
                                onclick: {
                                    let ids = ids.clone();
                                    move |_| {
                                        if matches!(
                                            operation,
                                            BulkOperation::Delete | BulkOperation::DeleteSubscriptions
                                        ) {
                                            pending.set(Some((operation, ids.clone())));
                                        } else {
                                            submit.call((operation, ids.clone()));
                                        }
                                    }
                                },
                                if matches!(operation, BulkOperation::Delete | BulkOperation::DeleteSubscriptions) {
                                    Trash2 {}
                                }
                                "{operation.label()} ({ids.len()})"
                            }
                        }
                    }
                }
                IconButton {
                    label: "Clear selection",
                    variant: ButtonVariant::Ghost,
                    disabled: busy(),
                   onclick: move |_| selected.write().clear(),
                    X {}
                }
                if selected().len() > 500 || matching.len() > 500 {
                    small { "Bulk actions support up to 500 rows at a time." }
                }
            }
        }
        if let Some((operation, ids)) = pending() {
            AlertDialog {
                AlertDialogContent { initial_open: true, on_close: move |_| pending.set(None),
                    AlertDialogHeader {
                        AlertDialogTitle { "{operation.label()}?" }
                        AlertDialogDescription {
                            if operation == BulkOperation::DeleteSubscriptions {
                                "Permanently delete {ids.len()} subscriptions and their stored configurations? This cannot be undone."
                            } else {
                                "Move {ids.len()} configurations to deleted? You can restore them later."
                            }
                        }
                    }
                    AlertDialogFooter {
                        IconButton {
                            label: "Cancel",
                            variant: ButtonVariant::Outline,
                            onclick: move |_| pending.set(None),
                            icons::X {}
                        }
                        IconButton {
                            label: "Delete {ids.len()} rows",
                            variant: ButtonVariant::Destructive,
                            disabled: busy(),
                            onclick: {
                                let ids = ids.clone();
                                move |_| submit.call((operation, ids.clone()))
                            },
                            Trash2 {}
                        }
                    }
                }
            }
        }
    }
}

pub fn toggle_page(selected: &mut BTreeSet<i64>, ids: &[i64]) {
    if !ids.is_empty() && ids.iter().all(|id| selected.contains(id)) {
        for id in ids {
            selected.remove(id);
        }
    } else {
        selected.extend(ids.iter().copied());
    }
}
