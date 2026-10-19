use crate::components::states::ErrorState;
use crate::ui::{button::ButtonVariant, icon_button::IconButton, select::*, spinner::Spinner};
use crate::{Session, api};
use dioxus::prelude::*;
use xrat_web::types::{CreatedToken, TokenInfo};

#[component]
pub(super) fn AccountTokenPicker(mut secret: Signal<String>) -> Element {
    let Session(session) = use_context();
    let mut tokens = use_resource(|| async {
        api::get::<Vec<TokenInfo>>("tokens").await.map(|tokens| {
            tokens
                .into_iter()
                .filter(|token| token.scope == "read")
                .collect::<Vec<_>>()
        })
    });
    let mut selected = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(String::new);
    let label_id = use_hook(|| crate::hooks::use_random::use_random_id_for("share-token"));
    let list = tokens.read().clone();
    let display = list
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .and_then(|tokens| tokens.iter().find(|token| token.id == selected()))
        .map(|token| token.name.clone());
    rsx! {
        div { class: "grid gap-3 mt-3",
            div { class: "flex items-center justify-between gap-2",
                label { r#for: label_id.clone(), class: "text-xs font-medium", "Read-only token" }
                IconButton { label: "Reload read-only tokens", variant: ButtonVariant::Ghost, disabled: busy() || tokens.pending(), onclick: move |_| {
                    secret.set(String::new()); selected.set(String::new()); error.set(String::new()); tokens.restart();
                }, icons::RefreshCw {} }
            }
            if tokens.pending() {
                p { role: "status", class: "flex items-center gap-2 text-xs text-muted-foreground", Spinner {} "Loading your read-only tokens…" }
            } else {
                match list {
                    Some(Ok(tokens)) if tokens.is_empty() => rsx! {
                        p { class: "text-xs text-muted-foreground leading-relaxed", "No read-only tokens yet. "
                            a { class: "text-primary underline", href: "/ui/account", target: "_blank", rel: "noopener noreferrer", "Create a read-only token in Account" }
                            ", then come back, reload this list, and select it."
                        }
                    },
                    Some(Ok(tokens)) => rsx! {
                        Select { selected_value: selected(), on_change: move |next: Option<String>| {
                            if let Some(id) = next {
                                selected.set(id.clone()); secret.set(String::new()); error.set(String::new()); busy.set(true);
                                spawn(async move {
                                    let csrf = session().map(|session| session.csrf_token).unwrap_or_default();
                                    match api::post::<_, CreatedToken>(&format!("tokens/{id}/subscription"), &(), &csrf).await {
                                        Ok(created) => secret.set(created.token),
                                        Err(message) => { selected.set(String::new()); error.set(message); }
                                    }
                                    busy.set(false);
                                });
                            }
                        },
                            SelectTrigger { id: label_id.clone(), disabled: busy(),
                                SelectValue { display_value: display, placeholder: "Select a read-only token" }
                            }
                            SelectContent { SelectGroup { aria_label: "Your read-only tokens",
                                for token in tokens {
                                    SelectOption { key: "{token.id}", value: token.id, "{token.name}" }
                                }
                            } }
                        }
                        p { class: "text-xs text-muted-foreground leading-relaxed", "Revoking the selected token also revokes its subscription links." }
                        if busy() {
                            p { role: "status", class: "flex items-center gap-2 text-xs text-muted-foreground", Spinner {} "Preparing subscription…" }
                        }
                    },
                    Some(Err(message)) => rsx! { ErrorState { message } },
                    None => rsx! {},
                }
            }
            if !error().is_empty() { ErrorState { message: error() } }
        }
    }
}
