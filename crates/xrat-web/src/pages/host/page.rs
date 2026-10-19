use super::components::*;
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
            crate::components::states::EmptyState {
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
        div { class: "host-page max-w-320 mx-auto",
            if !locked {
                div { class: "host-heading flex items-start justify-end mb-3 gap-3.5 [@media(width<=800px)]:flex-wrap",
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
                p { class: "host-session flex items-center text-muted-foreground text-[11px] mb-3.5 tabular-nums gap-[7px] [&_svg]:w-[13px] [&_svg]:h-[13px]",
                    LockOpen {}
                    "Unlocked · locks in {remaining / 60}:{remaining % 60:02}"
                }
            }
            div { class: "host-gate relative",
                div {
                    class: "host-controls [&&[data-locked=true]]:[filter:blur(4px)] [&&[data-locked=true]]:[opacity:0.35] [&&[data-locked=true]]:select-none ",
                    "data-locked": locked.to_string(),
                    inert: if locked { Some("") } else { None },
                    aria_hidden: locked.to_string(),
                    div { class: "host-grid grid grid-cols-[repeat(2,_minmax(0,_1fr))] gap-4 [@media(width<=800px)]:grid-cols-[minmax(0,_1fr)]",
                        Card { class: concat!(
                            "panel host-card mb-0 p-5 gap-3 mobile:pr-4 mobile:pb-4 mobile:pt-4 mobile:pl-4 [&_p]:m-0 ",
                            "[&_>_.actions]:mt-auto [&_>_button]:mt-auto [&_>_.icon-action]:mt-auto [&_>_.icon-action]:self-start ",
                            "[&&_[data-name=Field]]:[margin-top:0] ",
                        ),
                            div { class: "host-card-heading flex items-center gap-[9px] [&_>_svg]:w-[17px] [&_>_svg]:h-[17px] [&_>_svg]:text-muted-foreground [&_h3]:text-[14px] [&_h3]:font-semibold [&_h3]:m-0",
                                Download {}
                                h3 { "Engine" }
                            }
                            p { class: "muted text-muted-foreground text-[13px] leading-[1.7]",
                                "Install or update the proxy engine on this host."
                            }
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
                        Card { class: concat!(
                            "panel host-card mb-0 p-5 gap-3 mobile:pr-4 mobile:pb-4 mobile:pt-4 mobile:pl-4 [&_p]:m-0 ",
                            "[&_>_.actions]:mt-auto [&_>_button]:mt-auto [&_>_.icon-action]:mt-auto [&_>_.icon-action]:self-start ",
                            "[&&_[data-name=Field]]:[margin-top:0] ",
                        ),
                            div { class: "host-card-heading flex items-center gap-[9px] [&_>_svg]:w-[17px] [&_>_svg]:h-[17px] [&_>_svg]:text-muted-foreground [&_h3]:text-[14px] [&_h3]:font-semibold [&_h3]:m-0",
                                RefreshCw {}
                                h3 { "Automatic rotation" }
                            }
                            p { class: "muted text-muted-foreground text-[13px] leading-[1.7]",
                                "Rotate connections automatically. Requires a running XRAT daemon."
                            }
                            HostToggle { kind: "rotation", grant, busy }
                        }
                        Card { class: concat!(
                            "panel host-card mb-0 p-5 gap-3 mobile:pr-4 mobile:pb-4 mobile:pt-4 mobile:pl-4 [&_p]:m-0 ",
                            "[&_>_.actions]:mt-auto [&_>_button]:mt-auto [&_>_.icon-action]:mt-auto [&_>_.icon-action]:self-start ",
                            "[&&_[data-name=Field]]:[margin-top:0] ",
                        ),
                            div { class: "host-card-heading flex items-center gap-[9px] [&_>_svg]:w-[17px] [&_>_svg]:h-[17px] [&_>_svg]:text-muted-foreground [&_h3]:text-[14px] [&_h3]:font-semibold [&_h3]:m-0",
                                Network {}
                                h3 { "Network capture" }
                            }
                            p { class: "muted text-muted-foreground text-[13px] leading-[1.7]",
                                "Change TUN capture on the XRAT host. Host privileges must already be provisioned."
                            }
                            HostToggle { kind: "tun", grant, busy }
                        }
                        Card { class: concat!(
                            "panel host-card mb-0 p-5 gap-3 mobile:pr-4 mobile:pb-4 mobile:pt-4 mobile:pl-4 [&_p]:m-0 ",
                            "[&_>_.actions]:mt-auto [&_>_button]:mt-auto [&_>_.icon-action]:mt-auto [&_>_.icon-action]:self-start ",
                            "[&&_[data-name=Field]]:[margin-top:0] ",
                        ),
                            div { class: "host-card-heading flex items-center gap-[9px] [&_>_svg]:w-[17px] [&_>_svg]:h-[17px] [&_>_svg]:text-muted-foreground [&_h3]:text-[14px] [&_h3]:font-semibold [&_h3]:m-0",
                                Globe {}
                                h3 { "Maintenance" }
                            }
                            p { class: "muted text-muted-foreground text-[13px] leading-[1.7]",
                                "Validate server configuration and refresh local GeoIP assets."
                            }
                            div { class: "actions flex flex-wrap items-center mt-3 gap-2",
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
                    Card { class: "panel host-danger my-5 p-5 gap-2 mobile:pr-4 mobile:pb-4 mobile:pt-4 mobile:pl-4",
                        div { class: "host-card-heading flex items-center gap-[9px] [&_>_svg]:w-[17px] [&_>_svg]:h-[17px] [&_>_svg]:text-muted-foreground [&_h3]:text-[14px] [&_h3]:font-semibold [&_h3]:m-0",
                            Shield {}
                            h3 { "Changes that need a second look" }
                        }
                        p { class: "muted text-muted-foreground text-[13px] leading-[1.7]",
                            "These actions remove stored data or replace the installed application."
                        }
                        div {
                            class: concat!(
                                "host-danger-row flex justify-between items-center border-t border-solid ",
                                "border-t-[color:color-mix(in_srgb,_var(--border)_30%,_transparent)] py-3.5 px-0 gap-5 ",
                                "[@media(width<=800px)]:gap-3 [&_strong]:text-[13px] [&_strong]:font-medium [&_p]:mt-[3px] ",
                                "[&_p]:mb-0 [&_p]:text-[12px] [&_p]:mx-0 [&_button]:text-destructive",
                            ),
                            div {
                                strong { "Purge deleted configurations" }
                                p { class: "muted text-muted-foreground text-[13px] leading-[1.7]",
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
                        div {
                            class: concat!(
                                "host-danger-row flex justify-between items-center border-t border-solid ",
                                "border-t-[color:color-mix(in_srgb,_var(--border)_30%,_transparent)] py-3.5 px-0 gap-5 ",
                                "[@media(width<=800px)]:gap-3 [&_strong]:text-[13px] [&_strong]:font-medium [&_p]:mt-[3px] ",
                                "[&_p]:mb-0 [&_p]:text-[12px] [&_p]:mx-0 [&_button]:text-destructive",
                            ),
                            div {
                                strong { "Clear event history" }
                                p { class: "muted text-muted-foreground text-[13px] leading-[1.7]",
                                    "Delete XRAT's recorded operational events."
                                }
                            }
                            IconButton {
                                label: "Clear history",
                                variant: ButtonVariant::Outline,
                                disabled: busy(),
                                onclick: move |_| pending.set(Some(Action::ClearEvents)),
                                ScrollText {}
                            }
                        }
                        div {
                            class: concat!(
                                "host-danger-row flex justify-between items-center border-t border-solid ",
                                "border-t-[color:color-mix(in_srgb,_var(--border)_30%,_transparent)] py-3.5 px-0 gap-5 ",
                                "[@media(width<=800px)]:gap-3 [&_strong]:text-[13px] [&_strong]:font-medium [&_p]:mt-[3px] ",
                                "[&_p]:mb-0 [&_p]:text-[12px] [&_p]:mx-0 [&_button]:text-destructive",
                            ),
                            div {
                                strong { "Upgrade XRAT" }
                                p { class: "muted text-muted-foreground text-[13px] leading-[1.7]",
                                    "Install the latest release on this host."
                                }
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
                    div { class: "host-lock-overlay absolute top-0 right-0 bottom-0 left-0 z-[1] flex justify-center items-start bg-[color:color-mix(in_srgb,_var(--background)_25%,_transparent)] py-15 px-4 [@media(width<=800px)]:pt-8",
                        form {
                            class: concat!(
                                "host-unlock-form w-full max-w-87.5 border border-solid border-border rounded-[12px] bg-card ",
                                "shadow-[0_12px_40px_#0002] p-6.5 [&_h2]:text-[19px] [&_>_button]:w-full [&_>_button]:mt-4.5 ",
                                "[&_>_.icon-action]:flex [&_>_.icon-action]:w-max [&_>_.icon-action]:mt-4.5 ",
                                "[&_>_.icon-action]:ml-auto",
                            ),
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
                            div { class: "host-lock-icon grid place-items-center size-9.5 bg-accent text-primary rounded-[10px] mb-4.5 [&_svg]:size-4.5",
                                Lock {}
                            }
                            h2 { "Unlock to continue" }
                            p { class: "muted text-muted-foreground text-[13px] leading-[1.7]",
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
                                p {
                                    role: "alert",
                                    class: "error text-destructive text-[14px]",
                                    "{error}"
                                }
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
