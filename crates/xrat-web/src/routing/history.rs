use super::routes::page_for_path;
use dioxus::prelude::*;
use wasm_bindgen::{JsCast, closure::Closure};

pub(super) fn current_page() -> String {
    let path = web_sys::window()
        .and_then(|window| window.location().pathname().ok())
        .unwrap_or_else(|| "/".into());
    page_for_path(&path).into()
}
pub(super) struct HistoryListener {
    callback: Closure<dyn FnMut(web_sys::Event)>,
}
impl HistoryListener {
    pub(super) fn new(mut page: Signal<String>) -> Self {
        let callback = Closure::wrap(Box::new(move |_: web_sys::Event| page.set(current_page()))
            as Box<dyn FnMut(web_sys::Event)>);
        if let Some(window) = web_sys::window() {
            let _ = window
                .add_event_listener_with_callback("popstate", callback.as_ref().unchecked_ref());
        }
        Self { callback }
    }
}
impl Drop for HistoryListener {
    fn drop(&mut self) {
        if let Some(window) = web_sys::window() {
            let _ = window.remove_event_listener_with_callback(
                "popstate",
                self.callback.as_ref().unchecked_ref(),
            );
        }
    }
}
