use super::*;

#[component]
pub(crate) fn PasswordSettings(configured: bool, required: bool) -> Element {
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
        section {
            class: "self-account-section grid grid-cols-[minmax(170px,_0.7fr)_minmax(0,_1.7fr)] items-start mb-8 gap-9 [@media(width<=900px)]:grid-cols-[minmax(0,_1fr)] [@media(width<=900px)]:gap-3.5",
            aria_label: "Password settings",
            div {
                class: concat!(
                    "self-account-section-copy pt-1.5 [&_h2]:flex [&_h2]:items-center [&_h2]:text-[15px] ",
                    "[&_h2]:font-semibold [&_h2]:mt-0 [&_h2]:mb-[9px] [&_h2]:mx-0 [&_h2]:gap-2 [&_h2_svg]:w-4 ",
                    "[&_h2_svg]:h-4 [&_h2_svg]:text-muted-foreground [&_p]:max-w-[30ch] [&_p]:text-[12px] ",
                    "[&_p]:leading-[1.8] [&_p]:text-muted-foreground [&_p]:mt-0 [&_p]:mb-2.5 [&_p]:mx-0 ",
                    "[@media(width<=900px)]:[&_p]:max-w-[65ch]",
                ),
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
            }
            Card {
                class: concat!(
                    "self-account-panel rounded-[9px] shadow-none ",
                    "border-[color:color-mix(in_srgb,_var(--border)_50%,_var(--panel))] min-w-0 p-5.5 gap-0 ",
                    "narrow:pr-4 narrow:pb-4 narrow:pt-4 ",
                    "narrow:pl-4 [&_h3]:text-[13px] [&_h3]:font-semibold [&_h3]:m-0",
                ),
                if configured {
                    div { class: "self-account-managed flex items-start gap-3 [&_>_svg]:size-4.5 [&_>_svg]:text-muted-foreground [&_>_svg]:shrink-[0] [&_>_svg]:mt-px [&_p]:text-muted-foreground [&_p]:text-[12px] [&_p]:leading-[1.8] [&_p]:mt-[7px] [&_p]:mb-0 [&_p]:mx-0",
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
                        class: "self-password-form flex flex-col gap-4.5 [&_label]:text-[12px] [&&_>_[data-name=Field]]:[max-width:420px]",
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
                            p { class: "self-password-required border-l-2 border-solid border-l-primary pr-0 pl-3 text-[12px] leading-[1.8] m-0 py-0",
                                "Set a new password to continue using your account."
                            }
                        }
                        Field {
                            Label { html_for: "current-password", "Current password" }
                            div {
                                class: concat!(
                                    "self-password-input relative [&_input]:pr-9 [&_>_button]:absolute [&_>_button]:right-[5px] ",
                                    "[&_>_button]:top-[50%] [&_>_button]:transform-[translateY(-50%)] [&_>_button_svg]:w-3.5 ",
                                    "[&_>_button_svg]:h-3.5 [&_>_.icon-action]:absolute [&_>_.icon-action]:right-[5px] ",
                                    "[&_>_.icon-action]:top-[50%] [&_>_.icon-action]:transform-[translateY(-50%)]",
                                ),
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
                            div {
                                class: concat!(
                                    "self-password-input relative [&_input]:pr-9 [&_>_button]:absolute [&_>_button]:right-[5px] ",
                                    "[&_>_button]:top-[50%] [&_>_button]:transform-[translateY(-50%)] [&_>_button_svg]:w-3.5 ",
                                    "[&_>_button_svg]:h-3.5 [&_>_.icon-action]:absolute [&_>_.icon-action]:right-[5px] ",
                                    "[&_>_.icon-action]:top-[50%] [&_>_.icon-action]:transform-[translateY(-50%)]",
                                ),
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
                            p { class: "account-field-help text-muted-foreground text-[11px] leading-[1.6] m-0",
                                "Use at least 12 characters."
                            }
                        }
                        if !error().is_empty() {
                            ErrorState { message: error() }
                        }
                        div {
                            class: concat!(
                                "self-account-form-footer flex items-center justify-between flex-wrap pt-4 border-t ",
                                "border-solid border-t-[color:color-mix(in_srgb,_var(--border)_35%,_transparent)] gap-3.5 ",
                                "[&_p]:flex-1 [&_p]:min-w-42.5 [&_p]:max-w-[40ch] [&_p]:text-muted-foreground [&_p]:text-[11px] ",
                                "[&_p]:leading-[1.7] [&_p]:m-0",
                            ),
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
