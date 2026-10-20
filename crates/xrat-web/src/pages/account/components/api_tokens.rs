use super::*;

#[component]
pub(crate) fn ApiTokens() -> Element {
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
        section {
            class: "self-account-section grid grid-cols-[minmax(170px,_0.7fr)_minmax(0,_1.7fr)] items-start mb-8 gap-9 [@media(width<=900px)]:grid-cols-[minmax(0,_1fr)] [@media(width<=900px)]:gap-3.5",
            aria_label: "API tokens",
            div {
                class: concat!(
                    "self-account-section-copy pt-1.5 [&_h2]:flex [&_h2]:items-center [&_h2]:text-[15px] ",
                    "[&_h2]:font-semibold [&_h2]:mt-0 [&_h2]:mb-[9px] [&_h2]:mx-0 [&_h2]:gap-2 [&_h2_svg]:w-4 ",
                    "[&_h2_svg]:h-4 [&_h2_svg]:text-muted-foreground [&_p]:max-w-[30ch] [&_p]:text-[12px] ",
                    "[&_p]:leading-[1.8] [&_p]:text-muted-foreground [&_p]:mt-0 [&_p]:mb-2.5 [&_p]:mx-0 ",
                    "[@media(width<=900px)]:[&_p]:max-w-[65ch]",
                ),
                h2 {
                    Shield {}
                    "API tokens"
                }
                p { "Connect scripts and other clients to XRAT with scoped access." }
                p { "Tokens inherit your role and are revoked when your password or account changes." }
            }
            Card {
                class: concat!(
                    "self-account-panel self-token-panel rounded-[9px] shadow-none ",
                    "border-[color:color-mix(in_srgb,_var(--border)_50%,_var(--panel))] min-w-0 p-5.5 gap-0 ",
                    "narrow:pr-4 narrow:pb-4 narrow:pt-4 ",
                    "narrow:pl-4 [&_h3]:text-[13px] [&_h3]:font-semibold [&_h3]:m-0",
                ),
                if !error().is_empty() {
                    ErrorState { message: error() }
                }
                if !created().is_empty() {
                    div {
                        class: concat!(
                            "self-token-reveal border border-solid ",
                            "border-[color:color-mix(in_srgb,_var(--primary)_35%,_var(--border))] rounded-[6px] bg-accent ",
                            "mb-4.5 p-4 [&_h3]:flex [&_h3]:items-center [&_h3]:gap-[7px] [&_h3_svg]:w-[15px] ",
                            "[&_h3_svg]:h-[15px] [&_p]:text-[11px] [&_p]:leading-[1.7] [&_p]:text-muted-foreground ",
                            "[&_p]:mt-2 [&_p]:mb-3 [&_p]:mx-0 [&_textarea]:font-mono [&_textarea]:text-[12px] ",
                            "[&_textarea]:resize-none [&_textarea]:wrap-anywhere",
                        ),
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
                        div { class: "self-token-reveal-actions flex items-center justify-between mt-2 gap-2",
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
                        class: "self-token-create border-b border-solid border-b-[color:color-mix(in_srgb,_var(--border)_35%,_transparent)] mb-5.5 pb-5",
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
                        div { class: concat!(
                            "self-token-fields [&_label]:text-[12px] grid grid-cols-[minmax(0,_1fr)_minmax(0,_0.7fr)_auto] items-end ",
                            "mt-4 gap-2.5 narrow:gap-1.5 [&_>_div]:min-w-0 [&_input]:h-8 [&&_[data-name=SelectTrigger]]:[height:32px] ",
                        ),
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
                                class: "self-token-create-button h-8",

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
                        p { class: "account-field-help text-muted-foreground text-[11px] leading-[1.6] m-0",
                            if scope() == "manage" {
                                "Can perform operations allowed by your account’s current role."
                            } else {
                                "Can read XRAT data without making changes."
                            }
                        }
                        if count.is_some_and(|count| count >= 32) {
                            p { class: "account-field-help text-muted-foreground text-[11px] leading-[1.6] m-0",
                                "You’ve reached the 32-token limit. Revoke a token before creating another."
                            }
                        }
                    }
                }
                div { class: "self-token-heading flex items-center justify-between mb-4 gap-3",
                    div {
                        h3 { "Your tokens" }
                        p { class: "account-field-help text-muted-foreground text-[11px] leading-[1.6] m-0",
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
                    super::super::suspend::Suspend {}
                } else {
                    match data {
                        Some(Ok(list)) => rsx! {
                            if list.is_empty() {
                                EmptyState {
                                    title: "No tokens yet",
                                    description: "Create one above when you want to connect a script or client.",
                                }
                            } else {
                                ul { class: "self-token-list list-none m-0 p-0 [&_li]:flex [&_li]:items-center [&_li]:border-t [&_li]:border-solid [&_li]:border-t-[color:color-mix(in_srgb,_var(--border)_30%,_transparent)] [&_li]:py-3.5 [&_li]:px-0 [&_li]:gap-2.5",
                                    for token in list {
                                        li { key: "{token.id}",
                                            div { class: "self-token-icon grid place-items-center shrink-0 size-7.5 rounded-[6px] bg-accent text-muted-foreground [&_svg]:size-3.5",
                                                KeyRound {}
                                            }
                                            div {
                                                class: concat!(
                                                    "self-token-info flex-1 min-w-0 [&_strong]:block [&_strong]:text-[12px] [&_strong]:font-medium ",
                                                    "[&_strong]:wrap-anywhere [&_span]:block [&_span]:mt-1 [&_span]:text-[10px] ",
                                                    "[&_span]:text-muted-foreground [&_span]:tabular-nums [&_span]:leading-[1.7] ",
                                                    "narrow:[&_span]:max-w-[22ch]",
                                                ),
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
                            super::super::suspend::Suspend {}
                        },
                    }
                }
            }
            if let Some(token) = pending() {
                AlertDialog {
                    AlertDialogContent {
                        class: "w-[420px] max-w-[calc(100%-2rem)] p-5",
                        initial_open: true,
                        on_close: move |_| pending.set(None),
                        AlertDialogHeader { class: "gap-2 pr-6",
                            div { class: "mb-2 flex size-10 items-center justify-center rounded-full bg-destructive/10 text-destructive [&_svg]:size-5",
                                Trash2 {}
                            }
                            AlertDialogTitle { "Revoke API token?" }
                            AlertDialogDescription {
                                "Clients and subscription links using this token will stop working immediately."
                            }
                        }
                        div { class: "my-5 min-w-0 rounded-lg border bg-muted/30 px-3 py-2.5",
                            p { class: "m-0 text-sm font-medium wrap-anywhere", "{token.name}" }
                            p { class: "mt-1 mb-0 text-xs text-muted-foreground",
                                if token.scope == "manage" { "Manage access" } else { "Read-only access" }
                            }
                        }
                        AlertDialogFooter { class: "gap-2",
                            Button {
                                variant: ButtonVariant::Outline,
                                onclick: move |_| pending.set(None),
                                "Cancel"
                            }
                            Button {
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
                                "Revoke token"
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

    #[component]
    fn SummaryWithContext(user: User) -> Element {
        let session = use_signal(|| None);
        let notice = use_signal(|| None);
        use_context_provider(|| Session(session));
        use_context_provider(|| Notice(notice));
        rsx! { AccountSummary { user } }
    }

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
            let mut dom =
                VirtualDom::new_with_props(SummaryWithContext, SummaryWithContextProps { user });
            let mutations = dom.rebuild_to_vec();
            assert!(mutations.edits.iter().any(|edit| matches!(edit, dioxus::core::Mutation::CreateTextNode { value, .. } if value == expected)));
        }
    }
}
