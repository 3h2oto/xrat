use crate::components::{clipboard, qr_code::QrCodeImage, states::ErrorState};
use crate::ui::{
    button::{Button, ButtonVariant},
    dialog::*,
    dropdown_menu::*,
    icon_button::IconButton,
    input::{Input, InputType},
    spinner::Spinner,
    textarea::Textarea,
};
use crate::{Notice, Session, api};
use dioxus::prelude::*;
use xrat_web::configs::sharing::{
    ConfigFilter, ConfigSharePreview, ConfigShareRequest, ConfigShareResponse,
};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Mode {
    Qr,
    Link,
    Copy,
}

pub(crate) fn use_qr_preview(
    request: ConfigShareRequest,
    revision: String,
) -> Resource<Result<ConfigSharePreview, String>> {
    let Session(session) = use_context();
    let selection = use_memo(use_reactive!(|(request, revision)| (
        request.clone(),
        revision.clone()
    )));
    use_resource(move || {
        let (request, _) = selection();
        let csrf = session()
            .map(|session| session.csrf_token)
            .unwrap_or_default();
        async move { api::post("configs/share/preview", &request, &csrf).await }
    })
}

#[component]
pub(super) fn ShareControls(
    request: ConfigShareRequest,
    count: usize,
    #[props(default)] selected: bool,
    #[props(default)] revision: String,
) -> Element {
    let mut mode = use_signal(|| None::<Mode>);
    let preview = use_qr_preview(request.clone(), revision);
    let qr_supported = preview
        .read()
        .as_ref()
        .is_some_and(|result| result.as_ref().is_ok_and(|preview| preview.qr_supported));
    let label = if selected {
        format!("Share selected · {count}")
    } else {
        format!("Share filtered · {count}")
    };
    rsx! {
        if selected {
            div { class: "flex items-center flex-wrap gap-2",
                IconButton { label: "Copy configs", variant: ButtonVariant::Outline, disabled: count == 0, onclick: move |_| mode.set(Some(Mode::Copy)), icons::Copy {} }
                IconButton { label: if qr_supported { "Show selected configs QR" } else if preview.pending() { "Checking QR capacity" } else { "QR unavailable: selected configs exceed capacity or could not be checked" }, variant: ButtonVariant::Outline, disabled: count == 0 || !qr_supported, onclick: move |_| mode.set(Some(Mode::Qr)), icons::QrCode {} }
            }
        } else {
        DropdownMenu { align: DropdownMenuAlign::End,
            DropdownMenuTrigger { as_child: true,
                Button { variant: ButtonVariant::Outline, disabled: count == 0, icons::Share2 {} "{label}" }
            }
            DropdownMenuContent {
                DropdownMenuLabel { if selected { "Checked configurations" } else { "All matches across pages" } }
                DropdownMenuAction { disabled: count == 0, onclick: move |_| mode.set(Some(Mode::Copy)), icons::Copy {} "Copy configs" }
                DropdownMenuAction { disabled: count == 0 || !qr_supported, onclick: move |_| mode.set(Some(Mode::Qr)), icons::QrCode {}
                    if preview.pending() { "Checking QR capacity…" } else if !qr_supported { if preview.read().as_ref().is_some_and(|result| result.is_err()) { "Show QR · check failed" } else { "Show QR · exceeds capacity" } } else { "Show QR" }
                }
                if !selected {
                    DropdownMenuAction { disabled: count == 0, onclick: move |_| mode.set(Some(Mode::Link)), icons::Link {} "Subscription link" }
                }
            }
        }
        }
        if let Some(mode_value) = mode() {
            ShareModal { request: request.clone(), count, mode: mode_value, on_close: move |_| mode.set(None) }
        }
    }
}

#[component]
pub(crate) fn ShareModal(
    request: ConfigShareRequest,
    count: usize,
    mode: Mode,
    on_close: EventHandler,
) -> Element {
    let Session(session) = use_context();
    let Notice(mut notice) = use_context();
    let snapshot = use_hook(|| request);
    let count = use_hook(|| count);
    let mut mode = use_signal(|| mode);
    let mut secret = use_signal(String::new);
    let mut legacy = use_signal(|| false);
    let text_id = use_hook(|| crate::hooks::use_random::use_random_id_for("share-text"));
    let request = snapshot.clone();
    let mut raw = use_resource(move || {
        let request = request.clone();
        async move {
            if mode() == Mode::Link {
                return Ok(None);
            }
            let csrf = session()
                .map(|session| session.csrf_token)
                .unwrap_or_default();
            let response =
                api::post::<_, ConfigShareResponse>("configs/share", &request, &csrf).await?;
            if mode() == Mode::Copy {
                match clipboard::copy(&response.raw).await {
                    Ok(()) => {
                        notice.set(Some(crate::Feedback::success(format!(
                            "Copied {} configs.",
                            response.count
                        ))));
                        on_close.call(());
                    }
                    Err(message) => notice.set(Some(crate::Feedback::error(message))),
                }
            }
            Ok::<_, String>(Some(response))
        }
    });
    let filter = match &snapshot {
        ConfigShareRequest::Filtered { filter } => Some(filter),
        _ => None,
    };
    let url = if let Some(filter) = filter {
        let origin = web_sys::window()
            .and_then(|window| window.location().origin().ok())
            .unwrap_or_default();
        subscription_url(&origin, filter, &secret(), legacy())
    } else {
        String::new()
    };
    let result = raw.read().clone();
    rsx! {
        Dialog {
            DialogContent { initial_open: true, class: "w-[min(480px,calc(100%-2rem))] rounded-xl p-5", on_close: move |_| on_close.call(()),
                DialogHeader { class: "text-left pr-6 mb-4",
                    DialogTitle { class: "text-base", if mode() == Mode::Qr { "Configuration QR" } else if mode() == Mode::Link { "Subscription link" } else { "Copy configurations" } }
                    if mode() != Mode::Qr {
                        DialogDescription { class: "text-xs leading-relaxed", "{count} configurations · " if filter.is_some() { "All filter matches across pages" } else { "Checked rows" } }
                    }
                }
                if let Some(filter) = filter {
                    p { class: "text-xs text-muted-foreground leading-relaxed mb-4", "{filter_summary(filter)}" }
                    if filter.show_deleted {
                        p { role: "status", class: "text-xs text-destructive mb-4", "Deleted configurations are included when they match these filters." }
                    }
                }
                if mode() == Mode::Link {
                    p { class: "text-xs text-muted-foreground leading-relaxed mb-4", "Updates automatically whenever your client refreshes. Anyone with this URL can use its credential." }
                    div { class: "flex gap-2 mb-3",
                        button { r#type: "button", class: "text-xs px-3 py-2 rounded-md border aria-pressed:bg-accent aria-pressed:border-primary/50", aria_pressed: !legacy(), onclick: move |_| { legacy.set(false); secret.set(String::new()); }, "Account token" }
                        button { r#type: "button", class: "text-xs px-3 py-2 rounded-md border aria-pressed:bg-accent aria-pressed:border-primary/50", aria_pressed: legacy(), onclick: move |_| { legacy.set(true); secret.set(String::new()); }, "Legacy key" }
                    }
                    if legacy() {
                        label { class: "sr-only", r#for: format!("{text_id}-credential"), "Server API key" }
                        Input { id: format!("{text_id}-credential"), r#type: InputType::Password, placeholder: "Paste server key", value: secret(), oninput: move |event: FormEvent| secret.set(event.value()) }
                    } else {
                        super::share_token::AccountTokenPicker { secret }
                    }
                    if !url.is_empty() {
                        QrCodeImage { payload: url.clone(), label: "QR code for the subscription URL", filename: "xrat-subscription.svg" }
                        label { class: "sr-only", r#for: text_id.clone(), "Subscription URL" }
                        Textarea { id: text_id.clone(), readonly: true, rows: 4, value: url, class: "font-mono text-xs mt-3 break-all" }
                        Button { class: "mt-3", "data-copy-textarea": text_id.clone(), icons::Copy {} span { "Copy subscription link" } }
                    }
                } else {
                    if raw.pending() {
                        div { role: "status", class: "flex items-center justify-center gap-2 py-10 text-sm text-muted-foreground", Spinner {} "Preparing configs…" }
                    } else {
                        match result {
                            Some(Ok(Some(response))) if response.count > 0 => rsx! {
                                if mode() == Mode::Qr {
                                    QrCodeImage { payload: response.raw.clone() }
                                }
                                if mode() == Mode::Copy {
                                    p { class: "text-xs text-muted-foreground my-3", "{response.count} configs · Snapshot of the current raw links" }
                                    label { class: "sr-only", r#for: text_id.clone(), "Raw configurations" }
                                    Textarea { id: text_id.clone(), readonly: true, rows: 3, value: response.raw.clone(), class: "font-mono text-xs break-all" }
                                }
                                div { class: "flex items-center flex-wrap gap-2 mt-3",
                                    if mode() == Mode::Copy {
                                        Button { variant: ButtonVariant::Outline, "data-copy-textarea": text_id.clone(), icons::Copy {} span { "Copy configs" } }
                                    }
                                    if filter.is_some() {
                                        Button { variant: ButtonVariant::Ghost, onclick: move |_| mode.set(Mode::Link), icons::Link {} "Subscription link" }
                                    }
                                }
                            },
                            Some(Ok(_)) => rsx! { p { role: "status", class: "text-sm text-muted-foreground py-6", "No configurations match these filters anymore." } },
                            Some(Err(message)) => rsx! {
                                ErrorState { message }
                                IconButton { label: "Try again", variant: ButtonVariant::Outline, onclick: move |_| raw.restart(), icons::RefreshCw {} }
                            },
                            None => rsx! {},
                        }
                    }
                }
            }
        }
    }
}

fn subscription_url(origin: &str, filter: &ConfigFilter, secret: &str, legacy: bool) -> String {
    if secret.trim().is_empty() {
        return String::new();
    }
    let credential =
        serde_urlencoded::to_string([(if legacy { "key" } else { "token" }, secret.trim())])
            .unwrap_or_default();
    let filters = serde_urlencoded::to_string(filter).unwrap_or_default();
    format!(
        "{}/b64?{credential}&enabled=false&{filters}",
        origin.trim_end_matches('/')
    )
}

fn filter_summary(filter: &ConfigFilter) -> String {
    let mut labels = Vec::new();
    for (label, value) in [
        ("Search", &filter.q),
        ("Protocol", &filter.protocol),
        ("State", &filter.state),
        ("Subscription", &filter.subscription),
        ("Test", &filter.test),
        ("Country", &filter.country),
    ] {
        if !value.is_empty() {
            labels.push(format!("{label}: {value}"));
        }
    }
    if let Some(max) = filter.max_delay {
        labels.push(format!("Max delay: {max} ms"));
    }
    if labels.is_empty() {
        labels.push("All configurations".into());
    }
    labels.push(
        if filter.show_deleted {
            "Including deleted"
        } else {
            "Excluding deleted"
        }
        .into(),
    );
    labels.join(" · ")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn subscription_links_encode_credentials_and_exact_filters_without_pagination() {
        let filter = ConfigFilter {
            q: "a & b".into(),
            show_deleted: true,
            state: "disabled".into(),
            ..Default::default()
        };
        let url = subscription_url("https://xrat.example", &filter, "abc&?=+", false);
        assert!(url.contains("token=abc%26%3F%3D%2B"));
        assert!(url.contains("enabled=false"));
        assert!(url.contains("show_deleted=true"));
        assert!(url.contains("state=disabled"));
        assert!(!url.contains("page="));
        assert!(
            subscription_url("https://xrat.example", &filter, "secret", true)
                .contains("?key=secret&")
        );
        assert!(subscription_url("https://xrat.example", &filter, " ", false).is_empty());
    }
}
