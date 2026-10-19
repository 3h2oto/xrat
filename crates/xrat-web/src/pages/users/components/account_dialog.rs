use super::*;

#[component]
pub(crate) fn AccountDialog(
    user: Option<User>,
    reset_password: bool,
    busy: bool,
    error: String,
    names: Vec<String>,
    current_username: String,
    on_save: EventHandler<SaveUserRequest>,
    on_close: EventHandler<()>,
) -> Element {
    let mut username = use_signal(|| {
        user.as_ref()
            .map(|user| user.username.clone())
            .unwrap_or_default()
    });
    let mut password = use_signal(String::new);
    let role = use_signal(|| {
        user.as_ref()
            .map(|user| user.role.as_str())
            .unwrap_or("viewer")
            .to_string()
    });
    let mut enabled = use_signal(|| user.as_ref().is_none_or(|user| user.enabled));
    let mut show_password = use_signal(|| false);
    let mut local_error = use_signal(String::new);
    let creating = user.is_none();
    let title = if creating {
        "Create an account"
    } else if reset_password {
        "Reset password"
    } else {
        "Edit account"
    };
    let self_account = username() == current_username;
    rsx! {
        Dialog {
            DialogContent {
                initial_open: true,
                class: "account-dialog w-[min(440px,calc(100vw-2rem))] rounded-[12px] p-6 [@media(width<=600px)]:pr-5 [@media(width<=600px)]:pb-5 [@media(width<=600px)]:pt-5 [@media(width<=600px)]:pl-5",
                close_on_backdrop_click: !busy,
                on_close: move |_| on_close.call(()),
                DialogHeader {
                    DialogTitle { "{title}" }
                    DialogDescription {
                        if creating {
                            "Give someone access to your XRAT workspace."
                        } else if reset_password {
                            "Set a temporary password. The user must replace it at the next sign-in."
                        } else {
                            "Change this account's role and access."
                        }
                    }
                }
                form {
                    class: concat!(
                        "account-editor [&_>_.error]:mt-3.5 [&_.login-password-toggle]:top-0.5 ",
                        "[&_.login-password_>_.icon-action]:absolute [&_.login-password_>_.icon-action]:right-[5px] ",
                        "[&_.login-password_>_.icon-action]:top-[50%] ",
                        "[&_.login-password_>_.icon-action]:transform-[translateY(-50%)] [&&_[data-name=Field]]:[margin-top:18px] ",
                        "[&&_[data-name=Field]]:[gap:7px] ",
                    ),
                    onsubmit: move |event| {
                        event.prevent_default();
                        if busy {
                            return;
                        }
                        if creating && names.contains(&username()) {
                            local_error.set("An account with this username already exists.".into());
                            return;
                        }
                        local_error.set(String::new());
                        on_save
                            .call(SaveUserRequest {
                                username: username(),
                                role: role().parse().unwrap_or(Role::Viewer),
                                enabled: enabled(),
                                password: if creating || reset_password {
                                    Some(password())
                                } else {
                                    None
                                },
                            });
                        password.set(String::new());
                    },
                    Field {
                        Label { html_for: "account-username", "Username" }
                        Input {
                            id: "account-username",
                            required: true,
                            minlength: 3,
                            maxlength: 64,
                            readonly: !creating,
                            disabled: busy,
                            value: username(),
                            autocomplete: "off",
                            oninput: move |event: FormEvent| username.set(event.value()),
                        }
                        if creating {
                            p { class: "account-field-help text-muted-foreground text-[11px] leading-[1.6] m-0",
                                "3–64 lowercase letters, numbers, dots, underscores, or hyphens."
                            }
                        }
                    }
                    if !reset_password {
                        Field {
                            Label { html_for: "account-role", "Role" }
                            FormSelect {
                                id: "account-role",
                                value: role,
                                disabled: busy,
                                options: vec![
                                    FormSelectOption::new("viewer", "Viewer"),
                                    FormSelectOption::new("operator", "Operator"),
                                    FormSelectOption::new("admin", "Admin"),
                                ],
                            }
                            p { class: "account-field-help text-muted-foreground text-[11px] leading-[1.6] m-0",
                                "{role_help(&role())}"
                            }
                        }
                        if !creating {
                            div { class: "account-access-field flex items-center justify-between mt-5 gap-4",
                                Label { html_for: "account-enabled", "Account enabled" }
                                Switch {
                                    id: "account-enabled",
                                    checked: enabled(),
                                    disabled: busy || self_account,
                                    on_checked_change: move |checked| enabled.set(checked),
                                }
                            }
                        }
                    }
                    if creating || reset_password {
                        Field {
                            Label { html_for: "account-password", "Temporary password" }
                            div { class: "login-password relative [&_input]:pr-11",
                                Input {
                                    id: "account-password",
                                    r#type: if show_password() { InputType::Text } else { InputType::Password },
                                    value: password(),
                                    autocomplete: "new-password",
                                    required: true,
                                    minlength: 12,
                                    maxlength: 1024,
                                    disabled: busy,
                                    oninput: move |event: FormEvent| password.set(event.value()),
                                }
                                IconButton {
                                    label: if show_password() { "Hide password" } else { "Show password" },
                                    variant: ButtonVariant::Ghost,
                                    size: crate::ui::button::ButtonSize::IconXs,
                                    disabled: busy,
                                    aria_pressed: show_password(),
                                    onclick: move |_| show_password.toggle(),
                                    if show_password() {
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
                    }
                    if !creating {
                        p { class: "account-session-note flex items-start text-muted-foreground text-[11px] leading-[1.6] mt-5 mb-0 mx-0 gap-[7px] [&_svg]:size-[13px] [&_svg]:mt-0.5 [&_svg]:shrink-[0]",
                            KeyRound {}
                            "Saving signs this account out and revokes its API tokens."
                        }
                    }
                    if !local_error().is_empty() {
                        p {
                            class: "error text-destructive text-[14px]",
                            role: "alert",
                            "{local_error}"
                        }
                    }
                    if !error.is_empty() {
                        p {
                            class: "error text-destructive text-[14px]",
                            role: "alert",
                            "{error}"
                        }
                    }
                    div { class: "account-dialog-actions flex justify-end mt-6 gap-1.5",
                        IconButton {
                            label: "Cancel",
                            variant: ButtonVariant::Ghost,
                            disabled: busy,
                            onclick: move |_| on_close.call(()),
                            icons::X {}
                        }
                        IconButton {
                            label: if busy { "Saving account" } else if creating { "Create account" } else if reset_password { "Reset password" } else { "Save changes" },
                            button_type: "submit",
                            disabled: busy,
                            if busy {
                                Spinner {}
                            } else if creating {
                                Plus {}
                            } else if reset_password {
                                KeyRound {}
                            } else {
                                Save {}
                            }
                        }
                    }
                }
            }
        }
    }
}
pub(crate) fn role_label(role: Role) -> &'static str {
    match role {
        Role::Viewer => "Viewer",
        Role::Operator => "Operator",
        Role::Admin => "Admin",
    }
}
pub(crate) fn role_help(role: &str) -> &'static str {
    match role {
        "operator" => "Manage configurations, connections, tests, and tools.",
        "admin" => "All operator permissions, plus accounts, settings, and host controls.",
        _ => "Inspect configurations and runtime activity without making changes.",
    }
}
