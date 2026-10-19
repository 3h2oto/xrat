use super::{
    history::{HistoryListener, current_page},
    routes::{page_for_path, path_for_page},
};
use dioxus::prelude::*;
use wasm_bindgen::JsValue;

pub fn use_page_route() -> Signal<String> {
    let page = use_signal(current_page);
    let _listener = use_hook(move || std::rc::Rc::new(HistoryListener::new(page)));
    use_effect(move || {
        let name = page();
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                document.set_title(&format!("{name} · XRAT"));
            }
            if let Some(path) = path_for_page(&name)
                && window
                    .location()
                    .pathname()
                    .ok()
                    .is_some_and(|current| page_for_path(&current) != name)
                && let Ok(history) = window.history()
            {
                let _ = history.push_state_with_url(&JsValue::NULL, "", Some(path));
                window.scroll_to_with_x_and_y(0.0, 0.0);
            }
        }
    });
    page
}
