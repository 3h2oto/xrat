use dioxus::prelude::*;
use wasm_bindgen::{JsCast, closure::Closure};

pub fn use_import_paste(input: Signal<String>, importing: Signal<bool>) {
    let interactions = use_context::<crate::ui::interaction::Interactions>();
    use_hook(move || std::rc::Rc::new(ImportPasteListener::new(input, importing, interactions)));
}

struct ImportPasteListener {
    document: Option<web_sys::Document>,
    callback: Closure<dyn FnMut(web_sys::ClipboardEvent)>,
}

impl ImportPasteListener {
    fn new(
        mut input: Signal<String>,
        mut importing: Signal<bool>,
        interactions: crate::ui::interaction::Interactions,
    ) -> Self {
        let document = web_sys::window().and_then(|window| window.document());
        let callback = Closure::wrap(Box::new(move |event: web_sys::ClipboardEvent| {
            if event.default_prevented() || *importing.peek() {
                return;
            }
            let editable = event.target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                .and_then(|target| target.closest("input,textarea,select,[contenteditable]:not([contenteditable=\"false\"]),[role=\"textbox\"]").ok().flatten())
                .is_some();
            if editable || interactions.overlay_open() {
                return;
            }
            let Some(text) = event
                .clipboard_data()
                .and_then(|data| data.get_data("text/plain").ok())
                .filter(|text| !text.trim().is_empty())
            else {
                return;
            };
            event.prevent_default();
            input.set(text);
            importing.set(true);
        }) as Box<dyn FnMut(web_sys::ClipboardEvent)>);
        if let Some(document) = &document {
            let _ = document
                .add_event_listener_with_callback("paste", callback.as_ref().unchecked_ref());
        }
        Self { document, callback }
    }
}

impl Drop for ImportPasteListener {
    fn drop(&mut self) {
        if let Some(document) = &self.document {
            let _ = document.remove_event_listener_with_callback(
                "paste",
                self.callback.as_ref().unchecked_ref(),
            );
        }
    }
}
