use crate::ui::feedback::Feedback;
use crate::{api, routing, shell, ui};
use dioxus::prelude::*;
use xrat_web::types::LoginResponse;

#[derive(Clone, Copy)]
pub struct Session(pub Signal<Option<LoginResponse>>);
#[derive(Clone, Copy)]
pub struct Notice(pub Signal<Option<Feedback>>);
#[component]
pub fn App() -> Element {
    let session = use_signal(|| None::<LoginResponse>);
    let notice = use_signal(|| None::<Feedback>);
    use_context_provider(|| Session(session));
    use_context_provider(|| Notice(notice));
    let dark = use_signal(|| {
        web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item("xrat-theme").ok().flatten())
            .is_some_and(|v| v == "dark")
    });
    let page = routing::use_page_route();
    rsx! {
        div { class: if dark() { "dark app min-h-dvh bg-background text-foreground flex mobile:flex-col [&&:has(.workspace-shell)]:h-dvh" } else { "app min-h-dvh bg-background text-foreground flex mobile:flex-col [&&:has(.workspace-shell)]:h-dvh" },
            SuspenseBoundary {
                fallback: |_| rsx! {
                    ui::loading::SessionSkeleton {}
                },
                SessionGate { page, dark }
            }
            ui::feedback::FeedbackView {}
        }
    }
}

#[component]
fn SessionGate(page: Signal<String>, dark: Signal<bool>) -> Element {
    let Session(mut session) = use_context();
    let _restored = use_resource(move || async move {
        if let Ok(me) = api::get::<LoginResponse>("me").await {
            session.set(Some(me));
        }
    })
    .suspend()?;
    rsx! {
        shell::AppShell { page, dark }
    }
}
