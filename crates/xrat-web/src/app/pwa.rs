use std::rc::Rc;

use dioxus::prelude::*;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_futures::JsFuture;

pub(super) fn use_offline_support() {
    use_hook(|| Rc::new(register_when_loaded()));
}

struct LoadListener {
    window: web_sys::Window,
    callback: Closure<dyn FnMut(web_sys::Event)>,
}

fn register_when_loaded() -> Option<LoadListener> {
    let window = web_sys::window()?;
    let navigator = window.navigator();
    if !window.is_secure_context()
        || !js_sys::Reflect::has(navigator.as_ref(), &JsValue::from_str("serviceWorker"))
            .unwrap_or(false)
    {
        return None;
    }
    if window.document()?.ready_state() == "complete" {
        register(navigator);
        return None;
    }
    let callback = Closure::wrap(Box::new(move |_: web_sys::Event| {
        register(navigator.clone());
    }) as Box<dyn FnMut(web_sys::Event)>);
    let options = web_sys::AddEventListenerOptions::new();
    options.set_once(true);
    if let Err(error) = window.add_event_listener_with_callback_and_add_event_listener_options(
        "load",
        callback.as_ref().unchecked_ref(),
        &options,
    ) {
        warn(&error);
        return None;
    }
    Some(LoadListener { window, callback })
}

fn register(navigator: web_sys::Navigator) {
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(error) = JsFuture::from(navigator.service_worker().register("/sw.js")).await {
            warn(&error);
        }
    });
}

fn warn(error: &JsValue) {
    web_sys::console::warn_2(
        &JsValue::from_str("XRAT offline support unavailable"),
        error,
    );
}

impl Drop for LoadListener {
    fn drop(&mut self) {
        let _ = self
            .window
            .remove_event_listener_with_callback("load", self.callback.as_ref().unchecked_ref());
    }
}
