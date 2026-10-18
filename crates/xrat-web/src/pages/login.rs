use crate::ui::field::Field;
use crate::ui::input::{Input, InputType};
use crate::ui::label::Label;
use crate::ui::spinner::Spinner;
use crate::ui::states::ErrorState;
use crate::{Session, api, ui::button::Button};
use dioxus::prelude::*;
use icons::{Eye, EyeOff};
use xrat_web::types::{LoginRequest, LoginResponse};
#[component]
pub fn Login() -> Element {
    let Session(mut session) = use_context();
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut show_password = use_signal(|| false);
    rsx! {
        main { class: "login",
            section { class: "login-content",
                a {
                    class: "login-brand",
                    href: "/",
                    aria_label: "XRAT home",
                    img {
                        src: "/xrat-logo.png",
                        alt: "",
                        width: "32",
                        height: "32",
                    }
                    "XRAT"
                }
                div { class: "login-form-container",
                    form {
                        class: "login-form",
                        onsubmit: move |event| {
                            event.prevent_default();
                            busy.set(true);
                            error.set(String::new());
                            spawn(async move {
                                let result = api::post::<
                                    _,
                                    LoginResponse,
                                >(
                                        "login",
                                        &LoginRequest {
                                            username: username(),
                                            password: password(),
                                        },
                                        "",
                                    )
                                    .await;
                                password.set(String::new());
                                busy.set(false);
                                match result {
                                    Ok(me) => session.set(Some(me)),
                                    Err(message) => error.set(message),
                                }
                            });
                        },
                        div { class: "login-heading",
                            h1 { "Sign in to XRAT" }
                        }
                        Field {
                            Label { html_for: "username", "Username" }
                            Input {
                                id: "username",
                                name: "username",
                                placeholder: "Your username",
                                autocomplete: "username",
                                required: true,
                                maxlength: 64,
                                value: username(),
                                oninput: move |e: FormEvent| username.set(e.value()),
                            }
                        }
                        Label { html_for: "password", "Password" }
                        div { class: "login-password",
                            Input {
                                id: "password",
                                name: "password",
                                r#type: if show_password() { InputType::Text } else { InputType::Password },
                                autocomplete: "current-password",
                                required: true,
                                maxlength: 1024,
                                value: password(),
                                oninput: move |e: FormEvent| password.set(e.value()),
                            }
                            button {
                                class: "login-password-toggle",
                                r#type: "button",
                                aria_label: if show_password() { "Hide password" } else { "Show password" },
                                aria_pressed: show_password(),
                                onclick: move |_| show_password.set(!show_password()),
                                if show_password() {
                                    EyeOff {}
                                } else {
                                    Eye {}
                                }
                            }
                        }
                        if !error().is_empty() {
                            ErrorState { message: error() }
                        }
                        Button { button_type: "submit", disabled: busy(),
                            if busy() {
                                Spinner {}
                                "Signing in…"
                            } else {
                                "Sign in"
                            }
                        }
                    }
                }
            }
            div { class: "login-art",
                img {
                    src: "/xrat-logo.png",
                    alt: "XRAT mascot",
                    width: "512",
                    height: "512",
                }
            }
        }
    }
}
