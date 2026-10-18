use crate::{
    Notice, Session, api,
    ui::{
        button::{Button, ButtonSize, ButtonVariant},
        dialog::*,
        field::Field,
        input::{Input, InputType},
        label::Label,
        spinner::Spinner,
        switch::Switch,
        tooltip::*,
    },
};
use dioxus::prelude::*;
use icons::{Download, Lock, RefreshCw};
use xrat_web::types::{Action, HostUnlock, HostUnlockRequest, Job};

#[component]
pub fn DashboardAction(
    action: Action,
    text: String,
    disabled: bool,
    reason: String,
    #[props(default)] icon_only: bool,
) -> Element {
    let Session(session) = use_context();
    let Notice(mut notice) = use_context();
    let mut pending = use_signal(|| false);
    let refresh_subscriptions = matches!(action, Action::Refresh { .. });
    rsx! {
        Tooltip { class: "dashboard-action-tooltip whitespace-normal m-0",
            Button {
                variant: ButtonVariant::Outline,
                size: if icon_only { ButtonSize::IconSm } else { ButtonSize::Default },
                aria_label: text.clone(),
                disabled: disabled || pending(),
                onclick: move |_| {
                    if *pending.peek() || disabled {
                        return;
                    }
                    pending.set(true);
                    let action = action.clone();
                    spawn(async move {
                        let csrf = session().map(|me| me.csrf_token).unwrap_or_default();
                        match api::post::<_, Job>("actions", &action, &csrf).await {
                            Ok(job) => {
                                notice.set(Some(crate::Feedback::job_started(&job.kind)));
                                if let Err(error) = api::wait_for_job(&job.id).await {
                                    notice.set(Some(crate::Feedback::error(error)));
                                }
                            }
                            Err(error) => notice.set(Some(crate::Feedback::error(error))),
                        }
                        pending.set(false);
                    });
                },
                if pending() {
                    Spinner {}
                } else if refresh_subscriptions {
                    Download {}
                } else {
                    RefreshCw {}
                }
                if icon_only {
                    span { class: "sr-only", "{text}" }
                } else {
                    "{text}"
                }
            }
            TooltipContent { position: TooltipPosition::Bottom,
                if pending() {
                    "In progress. Follow this operation in Jobs."
                } else {
                    "{reason}"
                }
            }
        }
    }
}

#[component]
pub fn TunControl(
    enabled: bool,
    active: bool,
    available: bool,
    admin: bool,
    reason: Option<String>,
) -> Element {
    let Session(session) = use_context();
    let Notice(mut notice) = use_context();
    let mut requested = use_signal(|| None::<bool>);
    let mut password = use_signal(String::new);
    let mut pending = use_signal(|| false);
    let mut error = use_signal(String::new);
    rsx! {
        div { class: "dashboard-tun-control",
            Tooltip { class: "dashboard-action-tooltip whitespace-normal m-0",
                div { class: "dashboard-tun-switch flex items-center text-[12px] gap-[9px] [&_small]:text-muted-foreground [&_small]:text-[11px]",
                    span { "TUN" }
                    Switch {
                        checked: enabled,
                        disabled: !available || !admin || pending(),
                        aria_label: "TUN capture",
                        on_checked_change: move |next| {
                            password.set(String::new());
                            error.set(String::new());
                            requested.set(Some(next));
                        },
                    }
                    small {
                        if pending() {
                            "Applying…"
                        } else if active {
                            "Active"
                        } else if enabled {
                            "Enabled · inactive"
                        } else {
                            "Off"
                        }
                    }
                }
                TooltipContent { position: TooltipPosition::Bottom,
                    {
                        reason
                            .unwrap_or_else(|| {
                                if !admin {
                                    "Administrator access is required.".into()
                                } else {
                                    "Password confirmation required. Applying TUN may reconnect the active session."
                                        .into()
                                }
                            })
                    }
                }
            }
            if let Some(next) = requested() {
                Dialog {
                    DialogContent {
                        initial_open: true,
                        on_close: move |_| {
                            requested.set(None);
                            password.set(String::new());
                        },
                        DialogHeader {
                            DialogTitle {
                                if next {
                                    "Enable TUN?"
                                } else {
                                    "Disable TUN?"
                                }
                            }
                            DialogDescription {
                                "Re-enter your password to authorize this host change. Applying it may reconnect your active session. Engine support and host privileges will be checked."
                            }
                        }
                        form {
                            onsubmit: move |event| {
                                event.prevent_default();
                                if *pending.peek() || password.peek().is_empty() || !available || !admin {
                                    return;
                                }
                                let secret = password();
                                password.set(String::new());
                                pending.set(true);
                                error.set(String::new());
                                spawn(async move {
                                    let csrf = session().map(|me| me.csrf_token).unwrap_or_default();
                                    let result = async {
                                        let grant: HostUnlock = api::post(
                                                "host/unlock",
                                                &HostUnlockRequest {
                                                    password: secret,
                                                },
                                                &csrf,
                                            )
                                            .await?;
                                        let result = api::host_action(
                                                &Action::Tun { enabled: next },
                                                &csrf,
                                                &grant.token,
                                            )
                                            .await;
                                        let _ = api::post::<_, bool>("host/lock", &(), &csrf).await;
                                        let job = result?;
                                        notice.set(Some(crate::Feedback::job_started(&job.kind)));
                                        requested.set(None);
                                        api::wait_for_job(&job.id).await
                                    }
                                        .await;
                                    if let Err(message) = result {
                                        error.set(message.clone());
                                        notice.set(Some(crate::Feedback::error(message)));
                                    }
                                    pending.set(false);
                                });
                            },
                            Field {
                                Label { html_for: "overview-tun-password", "Password" }
                                Input {
                                    id: "overview-tun-password",
                                    r#type: InputType::Password,
                                    autocomplete: "current-password",
                                    required: true,
                                    value: password(),
                                    disabled: pending(),
                                    oninput: move |event: FormEvent| password.set(event.value()),
                                }
                            }
                            if !error().is_empty() {
                                p {
                                    class: "error text-destructive text-[14px]",
                                    role: "alert",
                                    "{error}"
                                }
                            }
                            DialogFooter {
                                Button {
                                    variant: ButtonVariant::Outline,
                                    disabled: pending(),
                                    onclick: move |_| {
                                        requested.set(None);
                                        password.set(String::new());
                                    },
                                    "Cancel"
                                }
                                Button {
                                    button_type: "submit",
                                    disabled: pending() || password().is_empty(),
                                    if pending() {
                                        Spinner {}
                                        "Authorizing…"
                                    } else {
                                        Lock {}
                                        "Confirm change"
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
