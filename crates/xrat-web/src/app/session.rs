use crate::{api, shell};
use dioxus::prelude::*;
use xrat_web::types::LoginResponse;

#[derive(Clone, Copy)]
pub struct Session(pub Signal<Option<LoginResponse>>);

#[component]
pub fn SessionGate(page: Signal<String>, dark: Signal<bool>) -> Element {
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
