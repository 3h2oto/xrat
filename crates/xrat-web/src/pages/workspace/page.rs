use crate::api::actions::run;
use crate::components::states::ErrorState;
use crate::ui::alert_dialog::{
    AlertDialog, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader,
    AlertDialogTitle,
};
use crate::ui::card::Card;
use crate::ui::icon_button::IconButton;
use crate::{Session, api, ui::button::ButtonVariant};
use dioxus::prelude::*;
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
    let mut importing = use_context::<crate::components::import::ImportState>().importing;
    let mut pending_delete = use_signal(|| None::<i64>);
    let result = data.read();
    let Some(result) = result.as_ref() else {
        return rsx! {
            super::suspend::Suspend { section: section.clone() }
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
            super::components::data_tables::ConfigTable {
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
            super::components::data_tables::SubscriptionTable {
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
            Card { class: "log-page-panel min-w-0 max-w-full rounded-[9px] shadow-none border-[color:color-mix(in_srgb,_var(--border)_50%,_var(--panel))] p-5 gap-0 mobile:pr-3.5 mobile:pb-3.5 mobile:pt-3.5 mobile:pl-3.5",
                crate::components::log_console::LogConsole {
                    id: "xrat-console",
                    lines: overview.events.iter().map(crate::components::log_console::event_line).collect(),
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
