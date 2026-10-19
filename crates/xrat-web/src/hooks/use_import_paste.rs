use dioxus::prelude::*;
use wasm_bindgen::{JsCast, closure::Closure};

pub fn use_import_paste(input: Signal<String>, importing: Signal<bool>) {
    use_hook(move || std::rc::Rc::new(ImportPasteListener::new(input, importing)));
}

struct ImportPasteListener {
    callback: Closure<dyn FnMut(web_sys::CustomEvent)>,
}

impl ImportPasteListener {
    fn new(mut input: Signal<String>, mut importing: Signal<bool>) -> Self {
        let callback = Closure::wrap(Box::new(move |event: web_sys::CustomEvent| {
            if let Some(text) = event.detail().as_string()
                && !text.trim().is_empty()
                && !*importing.peek()
            {
                input.set(text);
                importing.set(true);
            }
        }) as Box<dyn FnMut(web_sys::CustomEvent)>);
        if let Some(window) = web_sys::window() {
            let _ = window.add_event_listener_with_callback(
                "xrat-import-paste",
                callback.as_ref().unchecked_ref(),
            );
        }
        Self { callback }
    }
}

impl Drop for ImportPasteListener {
    fn drop(&mut self) {
        if let Some(window) = web_sys::window() {
            let _ = window.remove_event_listener_with_callback(
                "xrat-import-paste",
                self.callback.as_ref().unchecked_ref(),
            );
        }
    }
}
