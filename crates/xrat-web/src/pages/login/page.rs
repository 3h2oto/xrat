use crate::components::states::ErrorState;
use crate::ui::field::Field;
use crate::ui::input::{Input, InputType};
use crate::ui::label::Label;
use crate::ui::spinner::Spinner;
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
        main { class: "login grid grid-cols-[1fr_1fr] min-h-dvh w-full max-w-none bg-card p-0 [@media(width<=1023px)]:grid-cols-[1fr]",
            section { class: "login-content flex flex-col p-10 gap-8 mobile:pr-6 mobile:pb-6 mobile:pt-6 mobile:pl-6",
                a {
                    class: "login-brand flex items-center w-fit text-inherit no-underline font-[650] text-[16px] gap-2.5 mobile:mr-auto mobile:ml-auto [&_img]:object-contain",
                    href: "/",
                    aria_label: "XRAT home",
                    img {
                        src: "/media/xrat-logo.png",
                        alt: "",
                        width: "32",
                        height: "32",
                    }
                    "XRAT"
                }
                div { class: "login-form-container flex flex-1 items-center justify-center pt-6 pb-14 px-0 mobile:pb-6",
                    form {
                        class: concat!(
                            "login-form w-full max-w-80 [&_label]:mt-6 [&_label]:mb-2.5 [&_label]:text-[14px] [&_label]:mx-0 ",
                            "[&_input]:h-10 [&_input]:bg-card [&_>_button]:w-full [&_>_button]:mt-6 [&_>_button]:h-10 ",
                            "[&&_[data-name=Field]_label]:m-0 [&&_[data-name=Field]]:[margin-top:24px] ",
                        ),
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
                        div { class: "login-heading text-center mb-7 [&_h1]:text-[24px] [&_h1]:font-bold [&_h1]:tracking-[-0.6px] [&_h1]:leading-[1.3] [&_p]:mt-2 [&_p]:mb-0 [&_p]:text-[14px] [&_p]:leading-[1.5] [&_p]:mx-0",
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
                        div { class: "login-password relative [&_input]:pr-11",
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
                                class: "login-password-toggle absolute right-1 top-1 border-0 rounded-[4px] bg-transparent text-muted-foreground text-[12px] flex items-center justify-center size-8 p-0 [&:hover]:text-foreground [&:hover]:bg-accent [&_svg]:size-4",
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
            div { class: "login-art flex items-center justify-center bg-[color:var(--login-art)] border-l border-solid border-l-border p-16 [@media(width<=1023px)]:hidden [&_img]:w-[min(100%,_512px)] [&_img]:h-auto [&_img]:object-contain",
                img {
                    src: "/media/xrat-logo.png",
                    alt: "XRAT mascot",
                    width: "512",
                    height: "512",
                }
            }
        }
    }
}
