use crate::ui::icon_button::IconButton;
use crate::ui::{
    alert_dialog::*,
    avatar::{Avatar, AvatarFallback, AvatarSize},
    button::{Button, ButtonSize, ButtonVariant},
    card::Card,
    field::Field,
    form_select::{FormSelect, FormSelectOption},
    history::timestamp,
    input::{Input, InputType},
    label::Label,
    spinner::Spinner,
    states::{EmptyState, ErrorState, LoadingState},
    textarea::Textarea,
};
use crate::{Notice, Session, api};
use dioxus::prelude::*;
use icons::{
    Copy, Eye, EyeOff, KeyRound, Lock, LogOut, Plus, RefreshCw, Shield, Trash2, UserRound,
};
use xrat_web::types::{CreatedToken, PasswordRequest, TokenInfo, TokenRequest, TokenScope, User};

#[component]
pub fn Account() -> Element {
    let Session(session) = use_context();
    let Some(me) = session() else { return rsx! {} };
    let show_tokens = me.user.role.can_operate() && !me.user.must_change_password;
    rsx! {
        div { class: "self-account-page",
            AccountSummary { user: me.user.clone() }
            PasswordSettings {
                configured: me.user.configured,
                required: me.user.must_change_password,
                show_signout: !show_tokens,
            }
            if show_tokens {
                ApiTokens {}
            }
        }
    }
}

#[component]
fn SignOutButton() -> Element {
    let Session(mut session) = use_context();
    let Notice(mut notice) = use_context();
    let mut pending = use_signal(|| false);
    rsx! {
        Button {
            variant: ButtonVariant::Outline,
            disabled: pending(),
            onclick: move |_| {
                if *pending.peek() {
                    return;
                }
                pending.set(true);
                spawn(async move {
                    let csrf = session().map(|me| me.csrf_token).unwrap_or_default();
                    match api::post::<_, bool>("logout", &(), &csrf).await {
                        Ok(_) => session.set(None),
                        Err(error) => notice.set(Some(crate::Feedback::error(error))),
                    }
                    pending.set(false);
                });
            },
            if pending() {
                Spinner {}
                "Signing out…"
            } else {
                LogOut {}
                "Sign out"
            }
        }
    }
}

#[component]
fn AccountSummary(user: User) -> Element {
    let role = match user.role {
        xrat_web::types::Role::Admin => "Administrator",
        xrat_web::types::Role::Operator => "Operator",
        xrat_web::types::Role::Viewer => "Viewer",
    };
    rsx! {
        section { class: "self-account-summary", aria_label: "Current account",
            Avatar { size: AvatarSize::Lg,
                AvatarFallback { UserRound {} }
            }
            div {
                h2 { "{user.username}" }
                p { "{role}" }
            }
            span { class: "self-account-source",
                if user.configured {
                    Lock {}
                    "Managed on host"
                } else {
                    Shield {}
                    "Local account"
                }
            }
        }
    }
}

#[component]
fn PasswordSettings(configured: bool, required: bool, show_signout: bool) -> Element {
    let Session(mut session) = use_context();
    let Notice(mut notice) = use_context();
    let mut current = use_signal(String::new);
    let mut new_password = use_signal(String::new);
    let mut show_current = use_signal(|| false);
    let mut show_new = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut error = use_signal(String::new);
    let valid = !current().is_empty() && (12..=1024).contains(&new_password().len());
    rsx! {
        section { class: "self-account-section", aria_label: "Password settings",
            div { class: "self-account-section-copy",
                h2 {
                    KeyRound {}
                    "Password"
                }
                p {
                    if configured {
                        "Credentials for this account are managed outside the WebUI."
                    } else {
                        "Keep your account secure with a password you don’t use elsewhere."
                    }
                }
                if show_signout {
                    div { class: "self-account-signout", SignOutButton {} }
                }
            }
            Card { class: "self-account-panel",
                if configured {
                    div { class: "self-account-managed",
                        Lock {}
                        div {
                            h3 { "Managed in server configuration" }
                            p {
                                "Update your administrator password hash on the host, then restart XRAT."
                            }
                        }
                    }
                } else {
                    form {
                        class: "self-password-form",
                        onsubmit: move |event| {
                            event.prevent_default();
                            if *saving.peek() || !valid {
                                return;
                            }
                            saving.set(true);
                            error.set(String::new());
                            let request = PasswordRequest {
                                current_password: current(),
                                new_password: new_password(),
                            };
                            spawn(async move {
                                let csrf = session().map(|me| me.csrf_token).unwrap_or_default();
                                match api::post::<_, bool>("password", &request, &csrf).await {
                                    Ok(_) => {
                                        notice
                                            .set(
                                                Some(
                                                    crate::Feedback::success("Password changed. Sign in again."),
                                                ),
                                            );
                                        session.set(None);
                                    }
                                    Err(message) => error.set(message),
                                }
                                current.set(String::new());
                                new_password.set(String::new());
                                show_current.set(false);
                                show_new.set(false);
                                saving.set(false);
                            });
                        },
                        if required {
                            p { class: "self-password-required",
                                "Set a new password to continue using your account."
                            }
                        }
                        Field {
                            Label { html_for: "current-password", "Current password" }
                            div { class: "self-password-input",
                                Input {
                                    id: "current-password",
                                    r#type: if show_current() { InputType::Text } else { InputType::Password },
                                    autocomplete: "current-password",
                                    required: true,
                                    value: current(),
                                    disabled: saving(),
                                    oninput: move |event: FormEvent| current.set(event.value()),
                                }
                                IconButton {
                                    label: if show_current() { "Hide current password" } else { "Show current password" },
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::IconXs,
                                    disabled: saving(),
                                   onclick: move |_| show_current.toggle(),
                                    if show_current() {
                                        EyeOff {}
                                    } else {
                                        Eye {}
                                    }
                                }
                            }
                        }
                        Field {
                            Label { html_for: "new-password", "New password" }
                            div { class: "self-password-input",
                                Input {
                                    id: "new-password",
                                    r#type: if show_new() { InputType::Text } else { InputType::Password },
                                    autocomplete: "new-password",
                                    minlength: 12,
                                    maxlength: 1024,
                                    required: true,
                                    value: new_password(),
                                    disabled: saving(),
                                    oninput: move |event: FormEvent| new_password.set(event.value()),
                                }
                                IconButton {
                                    label: if show_new() { "Hide new password" } else { "Show new password" },
                                    variant: ButtonVariant::Ghost,
                                    size: ButtonSize::IconXs,
                                    disabled: saving(),
                                   onclick: move |_| show_new.toggle(),
                                    if show_new() {
                                        EyeOff {}
                                    } else {
                                        Eye {}
                                    }
                                }
                            }
                            p { class: "account-field-help", "Use at least 12 characters." }
                        }
                        if !error().is_empty() {
                            ErrorState { message: error() }
                        }
                        div { class: "self-account-form-footer",
                            p {
                                "Changing your password signs you out on every device and revokes your API tokens."
                            }
                            IconButton {
                                label: "Change password",
                                button_type: "submit",
                                disabled: saving() || !valid,
                                if saving() {
                                    Spinner {}
                                } else {
                                    KeyRound {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ApiTokens() -> Element {
    let Session(session) = use_context();
    let Notice(mut notice) = use_context();
    let mut tokens = use_resource(move || async { api::get::<Vec<TokenInfo>>("tokens").await });
    let mut name = use_signal(String::new);
    let scope = use_signal(|| "read".to_string());
    let mut created = use_signal(String::new);
    let mut mutating = use_signal(|| None::<String>);
    let mut error = use_signal(String::new);
    let mut pending = use_signal(|| None::<TokenInfo>);
    let secret_id = use_hook(|| crate::hooks::use_random::use_random_id_for("created-token"));
    let data = tokens.read().clone();
    let count = data
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .map(Vec::len);
    rsx! {
        section { class: "self-account-section", aria_label: "API tokens",
            div { class: "self-account-section-copy",
                h2 {
                    Shield {}
                    "API tokens"
                }
                p { "Connect scripts and other clients to XRAT with scoped access." }
                p { "Tokens inherit your role and are revoked when your password or account changes." }
                div { class: "self-account-signout", SignOutButton {} }
            }
            Card { class: "self-account-panel self-token-panel",
                if !error().is_empty() {
                    ErrorState { message: error() }
                }
                if !created().is_empty() {
                    div { class: "self-token-reveal",
                        h3 {
                            KeyRound {}
                            "Copy your new token"
                        }
                        p {
                            "This is the only time it will be shown. Copy it, then choose Done to continue managing tokens."
                        }
                        Label { html_for: secret_id.clone(), class: "sr-only", "New API token" }
                        Textarea {
                            id: secret_id.clone(),
                            readonly: true,
                            rows: 2,
                            value: created(),
                        }
                        div { class: "self-token-reveal-actions",
                            IconButton {
                                label: "Copy API token",
                                variant: ButtonVariant::Ghost,
                                "data-copy-textarea": secret_id.clone(),
                                Copy {}
                            }
                            IconButton {
                                label: "Done",
                                variant: ButtonVariant::Ghost,
                                onclick: move |_| created.set(String::new()),
                                icons::Check {}
                            }
                        }
                    }
                }
                if created().is_empty() {
                    form {
                        class: "self-token-create",
                        onsubmit: move |event| {
                            event.prevent_default();
                            let token_name = name().trim().to_string();
                            if mutating.peek().is_some() || token_name.is_empty() || token_name.len() > 100
                                || count.is_none_or(|count| count >= 32)
                            {
                                return;
                            }
                            mutating.set(Some("create".into()));
                            error.set(String::new());
                            let request = TokenRequest {
                                name: token_name,
                                scope: if scope() == "manage" {
                                    TokenScope::Manage
                                } else {
                                    TokenScope::Read
                                },
                            };
                            spawn(async move {
                                let csrf = session().map(|me| me.csrf_token).unwrap_or_default();
                                match api::post::<_, CreatedToken>("tokens", &request, &csrf).await {
                                    Ok(token) => {
                                        created.set(token.token);
                                        name.set(String::new());
                                        tokens.restart();
                                        notice
                                            .set(
                                                Some(
                                                    crate::Feedback::success(
                                                        "Token created. Copy it now; it won’t be shown again.",
                                                    ),
                                                ),
                                            );
                                    }
                                    Err(message) => error.set(message),
                                }
                                mutating.set(None);
                            });
                        },
                        h3 { "Create a token" }
                        div { class: "self-token-fields",
                            Field {
                                Label { html_for: "token-name", "Name" }
                                Input {
                                    id: "token-name",
                                    required: true,
                                    maxlength: 100,
                                    placeholder: "For example, laptop CLI",
                                    value: name(),
                                    disabled: mutating().is_some(),
                                    oninput: move |event: FormEvent| name.set(event.value()),
                                }
                            }
                            Field {
                                Label { html_for: "token-scope", "Access" }
                                FormSelect {
                                    id: "token-scope",
                                    value: scope,
                                    disabled: mutating().is_some(),
                                    options: vec![
                                        FormSelectOption::new("read", "Read only"),
                                        FormSelectOption::new("manage", "Manage"),
                                    ],
                                }
                            }
                            IconButton {
                                label: "Create token",
                                class: "self-token-create-button",

                                button_type: "submit",
                                disabled: mutating().is_some() || tokens.pending() || count.is_none_or(|count| count >= 32)
                                    || name().trim().is_empty() || name().trim().len() > 100,
                                if mutating().as_deref() == Some("create") {
                                    Spinner {}
                                } else {
                                    Plus {}
                                }
                            }
                        }
                        p { class: "account-field-help",
                            if scope() == "manage" {
                                "Can perform operations allowed by your account’s current role."
                            } else {
                                "Can read XRAT data without making changes."
                            }
                        }
                        if count.is_some_and(|count| count >= 32) {
                            p { class: "account-field-help",
                                "You’ve reached the 32-token limit. Revoke a token before creating another."
                            }
                        }
                    }
                }
                div { class: "self-token-heading",
                    div {
                        h3 { "Your tokens" }
                        p { class: "account-field-help",
                            if let Some(count) = count {
                                "{count} of 32 tokens"
                            } else {
                                "Only tokens belonging to your account appear here."
                            }
                        }
                    }
                    IconButton {
                        label: "Reload",
                        variant: ButtonVariant::Ghost,
                        disabled: tokens.pending() || mutating().is_some(),
                        onclick: move |_| tokens.restart(),
                        RefreshCw {}
                    }
                }
                if tokens.pending() {
                    LoadingState { label: "Loading your tokens…" }
                } else {
                    match data {
                        Some(Ok(list)) => rsx! {
                            if list.is_empty() {
                                EmptyState {
                                    title: "No tokens yet",
                                    description: "Create one above when you want to connect a script or client.",
                                }
                            } else {
                                ul { class: "self-token-list",
                                    for token in list {
                                        li { key: "{token.id}",
                                            div { class: "self-token-icon", KeyRound {} }
                                            div { class: "self-token-info",
                                                strong { "{token.name}" }
                                                span {
                                                    if token.scope == "manage" {
                                                        "Manage"
                                                    } else {
                                                        "Read only"
                                                    }
                                                    " · Created {timestamp(token.created_at)}"
                                                }
                                            }
                                            IconButton {
                                                label: format!("Revoke token {}", token.name),
                                                variant: ButtonVariant::Ghost,
                                                disabled: mutating().is_some() || !created().is_empty(),
                                                onclick: {
                                                    let token = token.clone();
                                                    move |_| pending.set(Some(token.clone()))
                                                },
                                                if mutating().as_deref() == Some(token.id.as_str()) {
                                                    Spinner {}
                                                } else {
                                                    Trash2 {}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Some(Err(message)) => rsx! {
                            ErrorState { message }
                        },
                        None => rsx! {
                            LoadingState { label: "Loading your tokens…" }
                        },
                    }
                }
            }
            if let Some(token) = pending() {
                AlertDialog {
                    AlertDialogContent {
                        initial_open: true,
                        on_close: move |_| pending.set(None),
                        AlertDialogHeader {
                            AlertDialogTitle { "Revoke token?" }
                            AlertDialogDescription {
                                "Clients using “{token.name}” will lose access immediately. This cannot be undone."
                            }
                        }
                        AlertDialogFooter {
                            IconButton {
                                label: "Keep token",
                                variant: ButtonVariant::Outline,
                                onclick: move |_| pending.set(None),
                                icons::X {}
                            }
                            IconButton {
                                label: "Revoke token",
                                variant: ButtonVariant::Destructive,
                                disabled: mutating().is_some(),
                                onclick: move |_| {
                                    if mutating.peek().is_some() {
                                        return;
                                    }
                                    let id = token.id.clone();
                                    mutating.set(Some(id.clone()));
                                    error.set(String::new());
                                    pending.set(None);
                                    spawn(async move {
                                        let csrf = session().map(|me| me.csrf_token).unwrap_or_default();
                                        match api::delete(&format!("tokens/{id}"), &csrf).await {
                                            Ok(_) => {
                                                tokens.restart();
                                                notice.set(Some(crate::Feedback::success("Token revoked.")));
                                            }
                                            Err(message) => error.set(message),
                                        }
                                        mutating.set(None);
                                    });
                                },
                                Trash2 {}
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xrat_web::types::Role;

    #[test]
    fn account_summary_renders_role_names_as_text() {
        for (role, expected) in [
            (Role::Admin, "Administrator"),
            (Role::Operator, "Operator"),
            (Role::Viewer, "Viewer"),
        ] {
            let user = User {
                username: "test-user".into(),
                role,
                enabled: true,
                must_change_password: false,
                configured: false,
            };
            let mut dom = VirtualDom::new_with_props(AccountSummary, AccountSummaryProps { user });
            let mutations = dom.rebuild_to_vec();
            assert!(mutations.edits.iter().any(|edit| matches!(edit, dioxus::core::Mutation::CreateTextNode { value, .. } if value == expected)));
        }
    }
}
