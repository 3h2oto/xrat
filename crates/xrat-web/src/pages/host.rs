use crate::ui::icon_button::IconButton;
use crate::ui::{
    alert_dialog::*,
    button::ButtonVariant,
    card::Card,
    field::Field,
    form_select::{FormSelect, FormSelectOption},
    input::{Input, InputType},
    label::Label,
    spinner::Spinner,
};
use crate::{Notice, Session, api};
use dioxus::prelude::*;
use icons::{
    CircleCheck, Download, Globe, Lock, LockOpen, Network, RefreshCw, ScrollText, Shield, Trash2,
};
use xrat_web::types::{Action, Engine, HostUnlock, HostUnlockRequest};

#[component]
pub fn HostAdministration() -> Element {
    let Session(session) = use_context();
    let mut grant = use_signal(|| None::<HostUnlock>);
    let mut password = use_signal(String::new);
    let mut error = use_signal(String::new);
    let mut unlocking = use_signal(|| false);
    let busy = use_signal(|| false);
    let mut clock = use_signal(|| (js_sys::Date::now() / 1000.0) as i64);
    let engine = use_signal(|| "xray".to_string());
    let mut pending = use_signal(|| None::<Action>);
    use_future(move || async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(1000).await;
            clock.set((js_sys::Date::now() / 1000.0) as i64);
            if grant().is_some_and(|value| value.expires_at <= clock()) {
                grant.set(None);
                pending.set(None);
            }
        }
    });
    if !session().is_some_and(|me| me.user.role.is_admin()) {
        return rsx! {
            crate::ui::states::EmptyState {
                title: "Administrator access required",
                description: "Host controls are available to administrators.",
            }
        };
    }
    let locked = grant().is_none();
    let remaining = grant()
        .map(|value| (value.expires_at - clock()).max(0))
        .unwrap_or(0);
    rsx! {
        div { class: "host-page",
            if !locked {
                div { class: "host-heading",
                    IconButton {
                        label: "Lock now",
                        variant: ButtonVariant::Ghost,
                        disabled: busy(),
                        onclick: move |_| {
                            grant.set(None);
                            pending.set(None);
                            unlocking.set(true);
                            spawn(async move {
                                let csrf = session().map(|me| me.csrf_token).unwrap_or_default();
                                if let Err(message) = api::post::<_, bool>("host/lock", &(), &csrf).await {
                                    error.set(message);
                                }
                                unlocking.set(false);
                            });
                        },
                        Lock {}
                    }
                }
            }
            if !locked {
                p { class: "host-session",
                    LockOpen {}
                    "Unlocked · locks in {remaining / 60}:{remaining % 60:02}"
                }
            }
            div { class: "host-gate",
                div {
                    class: "host-controls",
                    "data-locked": locked.to_string(),
                    inert: if locked { Some("") } else { None },
                    aria_hidden: locked.to_string(),
                    div { class: "host-grid",
                        Card { class: "panel host-card",
                            div { class: "host-card-heading",
                                Download {}
                                h3 { "Engine" }
                            }
                            p { class: "muted", "Install or update the proxy engine on this host." }
                            Field {
                                Label { html_for: "host-engine", "Managed engine" }
                                FormSelect {
                                    id: "host-engine",
                                    value: engine,
                                    options: vec![
                                        FormSelectOption::new("xray", "Xray"),
                                        FormSelectOption::new("sing-box", "sing-box"),
                                        FormSelectOption::new("v2ray", "V2Ray"),
                                    ],
                                }
                            }
                            IconButton {
                                label: "Install / update",
                                variant: ButtonVariant::Outline,
                                disabled: busy(),
                                onclick: move |_| submit(
                                    Action::InstallEngine {
                                        engine: match engine().as_str() {
                                            "sing-box" => Engine::SingBox,
                                            "v2ray" => Engine::V2ray,
                                            _ => Engine::Xray,
                                        },
                                    },
                                    grant,
                                    busy,
                                ),
                                Download {}
                            }
                        }
                        Card { class: "panel host-card",
                            div { class: "host-card-heading",
                                RefreshCw {}
                                h3 { "Automatic rotation" }
                            }
                            p { class: "muted",
                                "Rotate connections automatically. Requires a running XRAT daemon."
                            }
                            HostToggle { kind: "rotation", grant, busy }
                        }
                        Card { class: "panel host-card",
                            div { class: "host-card-heading",
                                Network {}
                                h3 { "Network capture" }
                            }
                            p { class: "muted",
                                "Change TUN capture on the XRAT host. Host privileges must already be provisioned."
                            }
                            HostToggle { kind: "tun", grant, busy }
                        }
                        Card { class: "panel host-card",
                            div { class: "host-card-heading",
                                Globe {}
                                h3 { "Maintenance" }
                            }
                            p { class: "muted",
                                "Validate server configuration and refresh local GeoIP assets."
                            }
                            div { class: "actions",
                                IconButton {
                                    label: "Validate config",
                                    variant: ButtonVariant::Outline,
                                    disabled: busy(),
                                    onclick: move |_| submit(Action::Validate, grant, busy),
                                    CircleCheck {}
                                }
                                IconButton {
                                    label: "Update GeoIP",
                                    variant: ButtonVariant::Ghost,
                                    disabled: busy(),
                                    onclick: move |_| submit(Action::GeoIpUpdate, grant, busy),
                                    RefreshCw {}
                                }
                            }
                        }
                    }
                    Card { class: "panel host-danger",
                        div { class: "host-card-heading",
                            Shield {}
                            h3 { "Changes that need a second look" }
                        }
                        p { class: "muted",
                            "These actions remove stored data or replace the installed application."
                        }
                        div { class: "host-danger-row",
                            div {
                                strong { "Purge deleted configurations" }
                                p { class: "muted",
                                    "Permanently remove configurations already marked as deleted."
                                }
                            }
                            IconButton {
                                label: "Purge",
                                variant: ButtonVariant::Outline,
                                disabled: busy(),
                                onclick: move |_| pending.set(Some(Action::Purge)),
                                Trash2 {}
                            }
                        }
                        div { class: "host-danger-row",
                            div {
                                strong { "Clear event history" }
                                p { class: "muted", "Delete XRAT's recorded operational events." }
                            }
                            IconButton {
                                label: "Clear history",
                                variant: ButtonVariant::Outline,
                                disabled: busy(),
                                onclick: move |_| pending.set(Some(Action::ClearEvents)),
                                ScrollText {}
                            }
                        }
                        div { class: "host-danger-row",
                            div {
                                strong { "Upgrade XRAT" }
                                p { class: "muted", "Install the latest release on this host." }
                            }
                            IconButton {
                                label: "Upgrade",
                                variant: ButtonVariant::Outline,
                                disabled: busy(),
                                onclick: move |_| pending.set(Some(Action::Upgrade { version: None })),
                                Download {}
                            }
                        }
                    }
                }
                if locked {
                    div { class: "host-lock-overlay",
                        form {
                            class: "host-unlock-form",
                            onsubmit: move |event| {
                                event.prevent_default();
                                if unlocking() || password().is_empty() {
                                    return;
                                }
                                unlocking.set(true);
                                error.set(String::new());
                                let secret = password();
                                password.set(String::new());
                                spawn(async move {
                                    let csrf = session().map(|me| me.csrf_token).unwrap_or_default();
                                    match api::post::<
                                        _,
                                        HostUnlock,
                                    >(
                                            "host/unlock",
                                            &HostUnlockRequest {
                                                password: secret,
                                            },
                                            &csrf,
                                        )
                                        .await
                                    {
                                        Ok(value) => grant.set(Some(value)),
                                        Err(message) => {
                                            error
                                                .set(
                                                    if message.contains("not permitted") {
                                                        "Password not accepted. Try again.".into()
                                                    } else {
                                                        message
                                                    },
                                                )
                                        }
                                    }
                                    unlocking.set(false);
                                });
                            },
                            div { class: "host-lock-icon", Lock {} }
                            h2 { "Unlock to continue" }
                            p { class: "muted",
                                "Re-enter your password to access host controls for five minutes."
                            }
                            Field {
                                Label { html_for: "host-password", "Your password" }
                                Input {
                                    id: "host-password",
                                    r#type: InputType::Password,
                                    maxlength: 1024,
                                    autocomplete: "current-password",
                                    value: password(),
                                    disabled: unlocking(),
                                    oninput: move |event: FormEvent| password.set(event.value()),
                                }
                            }
                            if !error().is_empty() {
                                p { role: "alert", class: "error", "{error}" }
                            }
                            IconButton {
                                label: "Unlock host controls",
                                button_type: "submit",
                                disabled: unlocking() || password().is_empty(),
                                if unlocking() {
                                    Spinner {}
                                } else {
                                    LockOpen {}
                                }
                            }
                        }
                    }
                }
            }
            if let Some(action) = pending() {
                AlertDialog {
                    AlertDialogContent {
                        initial_open: true,
                        on_close: move |_| pending.set(None),
                        AlertDialogHeader {
                            AlertDialogTitle { "Confirm host action" }
                            AlertDialogDescription {
                                match action {
                                    Action::Purge => {
                                        "Permanently purge deleted configurations? This cannot be undone."
                                    }
                                    Action::ClearEvents => "Clear all recorded event history? This cannot be undone.",
                                    _ => {
                                        "Install the latest XRAT release on this host? The running service may restart."
                                    }
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
                                label: "Confirm",
                                variant: ButtonVariant::Destructive,
                                disabled: busy(),
                                onclick: move |_| {
                                    submit(action.clone(), grant, busy);
                                    pending.set(None);
                                },
                                icons::Check {}
                            }
                        }
                    }
                }
            }
        }
    }
}
fn submit(action: Action, mut grant: Signal<Option<HostUnlock>>, mut busy: Signal<bool>) {
    let Session(session) = consume_context();
    let Notice(mut notice) = consume_context();
    let Some(unlock) = grant() else {
        return;
    };
    if busy() {
        return;
    }
    busy.set(true);
    spawn(async move {
        let csrf = session().map(|me| me.csrf_token).unwrap_or_default();
        match api::host_action(&action, &csrf, &unlock.token).await {
            Ok(job) => notice.set(Some(crate::Feedback::job_started(&job.kind))),
            Err(error) => {
                if error.contains("not permitted") {
                    grant.set(None);
                }
                notice.set(Some(crate::Feedback::error(error)));
            }
        }
        busy.set(false);
    });
}

#[component]
fn HostToggle(
    kind: &'static str,
    mut grant: Signal<Option<HostUnlock>>,
    busy: Signal<bool>,
) -> Element {
    let Session(session) = use_context();
    let Notice(mut notice) = use_context();
    let mut pending = use_signal(|| false);
    let mut tick = use_signal(|| 0);
    use_future(move || async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(3000).await;
            tick += 1;
        }
    });
    let mut status = use_resource(move || async move {
        let _ = tick();
        api::get::<xrat_web::types::Settings>("settings").await
    });
    let path = if kind == "rotation" {
        "runtime.rotation.enabled"
    } else {
        "runtime.tun.enabled"
    };
    let checked = status
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .and_then(|settings| settings.items.iter().find(|item| item.path == path))
        .and_then(|setting| setting.value.parse::<bool>().ok());
    rsx! {
        div { class: "feature-toggle-row",
            span { class: "muted",
                if pending() {
                    Spinner {}
                    "Updating…"
                } else {
                    match checked {
                        Some(true) => "Enabled",
                        Some(false) => "Disabled",
                        None => "State unavailable",
                    }
                }
            }
            crate::ui::switch::Switch {
                checked: checked.unwrap_or(false),
                disabled: checked.is_none() || busy() || pending() || grant().is_none(),
                aria_label: if kind == "rotation" { "Automatic rotation" } else { "TUN capture" },
                on_checked_change: move |enabled| {
                    let Some(unlock) = grant() else {
                        return;
                    };
                    pending.set(true);
                    spawn(async move {
                        let csrf = session().map(|me| me.csrf_token).unwrap_or_default();
                        let action = if kind == "rotation" {
                            Action::Rotation { enabled }
                        } else {
                            Action::Tun { enabled }
                        };
                        let result = match api::host_action(&action, &csrf, &unlock.token).await {
                            Ok(job) => api::wait_for_job(&job.id).await,
                            Err(error) => Err(error),
                        };
                        if let Err(error) = result {
                            if error.contains("not permitted") {
                                grant.set(None);
                            }
                            notice.set(Some(crate::Feedback::error(error)));
                        }
                        status.restart();
                        pending.set(false);
                    });
                },
            }
        }
    }
}
