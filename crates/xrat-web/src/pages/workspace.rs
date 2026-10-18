use super::run;
use crate::ui::alert_dialog::{
    AlertDialog, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader,
    AlertDialogTitle,
};
use crate::ui::card::Card;
use crate::ui::dialog::{
    Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};
use crate::ui::field::Field;
use crate::ui::icon_button::IconButton;
use crate::ui::input::{Input, InputType};
use crate::ui::label::Label;
use crate::ui::states::{ErrorState, LoadingState};
use crate::ui::textarea::Textarea;
use crate::{Session, api, ui::button::ButtonVariant};
use dioxus::prelude::*;
use icons::{Play, RotateCcw};
use xrat_web::types::{Action, Overview};
#[component]
pub fn Workspace(section: String) -> Element {
    let Session(session) = use_context();
    let operator = session().is_some_and(|s| s.user.role.can_operate());
    let mut tick = use_signal(|| 0);
    use_future(move || async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(3000).await;
            tick += 1;
        }
    });
    let mut manual_reload = use_signal(|| false);
    let mut data = use_resource(move || async move {
        let _ = tick();
        let result = api::get::<Overview>("overview").await;
        manual_reload.set(false);
        result
    });
    let mut importing = use_signal(|| false);
    let mut input = use_signal(String::new);
    let mut name = use_signal(String::new);
    crate::hooks::use_import_paste::use_import_paste(input, importing);
    let mut pending_delete = use_signal(|| None::<i64>);
    let result = data.read();
    let Some(result) = result.as_ref() else {
        return rsx! {
            LoadingState { label: "Loading workspace…" }
        };
    };
    let overview = match result {
        Ok(data) => data,
        Err(error) => {
            return rsx! {
                ErrorState { message: error.clone() }
            };
        }
    };
    rsx! {
        if section == "Configs" {
            super::data_tables::ConfigTable {
                configs: overview.configs.clone(),
                active_config: if overview.running { overview.active_config } else { None },
                subscriptions: overview.subscriptions.clone(),
                operator,
                loading: manual_reload(),
                on_reload: move |_| {
                    manual_reload.set(true);
                    data.restart();
                },
                on_import: move |_| importing.set(true),
                pending_delete,
            }
        }
        if section == "Subscriptions" {
            super::data_tables::SubscriptionTable {
                subscriptions: overview.subscriptions.clone(),
                operator,
                loading: manual_reload(),
                on_import: move |_| importing.set(true),
                on_reload: move |_| {
                    manual_reload.set(true);
                    data.restart();
                },
            }
        }
        if section == "Logs" {
            Card { class: "log-page-panel",
                crate::ui::log_console::LogConsole {
                    id: "xrat-console",
                    lines: overview.events.iter().map(crate::ui::log_console::event_line).collect(),
                    loading: manual_reload(),
                    limit: "Latest 100 events",
                    empty_hint: "Operational events will appear here as you use XRAT.",
                    on_reload: move |_| {
                        manual_reload.set(true);
                        data.restart();
                    },
                }
            }
        }
        if importing() {
            Dialog {
                DialogContent {
                    initial_open: true,
                    on_close: move |_: MouseEvent| importing.set(false),
                    class: "w-[min(36rem,calc(100%-2rem))]",
                    DialogHeader {
                        DialogTitle {
                            if section == "Subscriptions" {
                                "Add subscription"
                            } else {
                                "Import configurations"
                            }
                        }
                        DialogDescription {
                            if section == "Subscriptions" {
                                "Paste a subscription URL or load its contents from a text file."
                            } else {
                                "Paste a subscription URL or configuration links. Local file paths are not accepted."
                            }
                        }
                    }
                    Field {
                        Label { html_for: "import-name", "Name (optional)" }
                        Input {
                            id: "import-name",
                            value: name(),
                            oninput: move |e: FormEvent| name.set(e.value()),
                        }
                    }
                    Field {
                        Label { html_for: "import-file", "Or load a local text file" }
                        Input {
                            id: "import-file",
                            r#type: InputType::File,
                            onchange: move |event: FormEvent| {
                                if let Some(file) = event.files().into_iter().next() {
                                    spawn(async move {
                                        if let Ok(text) = file.read_string().await {
                                            input.set(text);
                                        }
                                    });
                                }
                            },
                        }
                    }
                    Field {
                        Label { html_for: "import-input",
                            if section == "Subscriptions" {
                                "Subscription URL or content"
                            } else {
                                "URL or configuration content"
                            }
                        }
                        Textarea {
                            id: "import-input",
                            rows: 7,
                            value: input(),
                            oninput: move |e: FormEvent| input.set(e.value()),
                        }
                    }
                    DialogFooter {
                        IconButton {
                            label: "Cancel",
                            variant: ButtonVariant::Outline,
                            onclick: move |_| importing.set(false),
                            icons::X {}
                        }
                        IconButton {
                            label: if section == "Subscriptions" { "Add subscription" } else { "Import configurations" },
                            disabled: input().trim().is_empty(),
                            onclick: move |_| {
                                run(Action::Import {
                                    input: input(),
                                    name: (!name().is_empty()).then_some(name()),
                                });
                                input.set(String::new());
                                importing.set(false);
                            },
                            icons::Plus {}
                        }
                    }
                }
            }
        }
        if let Some(id) = pending_delete() {
            AlertDialog {
                AlertDialogContent {
                    initial_open: true,
                    on_close: move |_: MouseEvent| pending_delete.set(None),
                    class: "w-[min(32rem,calc(100%-2rem))]",
                    AlertDialogHeader {
                        AlertDialogTitle { "Delete configuration #{id}?" }
                        AlertDialogDescription { "It can be restored later from this list." }
                    }
                    AlertDialogFooter {
                        IconButton {
                            label: "Cancel",
                            variant: ButtonVariant::Outline,
                            onclick: move |_| pending_delete.set(None),
                            icons::X {}
                        }
                        IconButton {
                            label: "Delete",
                            variant: ButtonVariant::Destructive,
                            onclick: move |_| {
                                run(Action::Delete { id, hard: false });
                                pending_delete.set(None);
                            },
                            icons::Trash2 {}
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub(super) fn ConfigActions(
    id: i64,
    deleted: bool,
    mut pending_delete: Signal<Option<i64>>,
) -> Element {
    use crate::ui::dropdown_menu::{
        DropdownMenu, DropdownMenuAction, DropdownMenuActionVariant, DropdownMenuContent,
        DropdownMenuGroup, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger,
    };
    use icons::{Download, EllipsisVertical, FlaskConical, Trash2};
    rsx! {
        div { class: "flex items-center gap-2",
            if deleted {
                IconButton {
                    label: "Restore",
                    variant: ButtonVariant::Outline,
                    onclick: move |_: MouseEvent| run(Action::Restore { id }),
                    RotateCcw {}
                }
            } else {
                IconButton {
                    label: "Connect",
                    onclick: move |_: MouseEvent| run(Action::Connect { id }),
                    Play {}
                }
                DropdownMenu {
                    DropdownMenuTrigger { as_child: true,
                        IconButton {
                            label: format!("Actions for configuration #{id}"),
                            variant: ButtonVariant::Ghost,
                            EllipsisVertical {}
                        }
                    }
                    DropdownMenuContent {
                        DropdownMenuLabel { "Configuration #{id}" }
                        DropdownMenuGroup {
                            DropdownMenuAction { onclick: move |_: MouseEvent| run(Action::Test { id: Some(id) }),
                                FlaskConical {}
                                "Test connection"
                            }
                            DropdownMenuAction {
                                href: format!("/api/v1/configs/{id}/raw"),
                                download: format!("config-{id}.txt"),
                                Download {}
                                "Export"
                            }
                            DropdownMenuSeparator {}
                            DropdownMenuAction {
                                variant: DropdownMenuActionVariant::Destructive,
                                onclick: move |_: MouseEvent| pending_delete.set(Some(id)),
                                Trash2 {}
                                "Delete"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub(super) fn ConfigToggle(id: i64, enabled: bool) -> Element {
    let Session(session) = use_context();
    let crate::Notice(mut notice) = use_context();
    let mut pending = use_signal(|| false);
    rsx! {
        crate::ui::switch::Switch {
            checked: enabled,
            disabled: pending(),
            aria_label: format!("Enable configuration {id}"),
            on_checked_change: move |checked| {
                pending.set(true);
                spawn(async move {
                    let csrf = session().map(|me| me.csrf_token).unwrap_or_default();
                    let action = if checked {
                        Action::Enable { id }
                    } else {
                        Action::Disable { id }
                    };
                    let result = match api::post::<
                        _,
                        xrat_web::types::Job,
                    >("actions", &action, &csrf)
                        .await
                    {
                        Ok(job) => api::wait_for_job(&job.id).await,
                        Err(error) => Err(error),
                    };
                    if let Err(error) = result {
                        notice.set(Some(crate::Feedback::error(error)));
                    }
                    pending.set(false);
                });
            },
        }
    }
}
