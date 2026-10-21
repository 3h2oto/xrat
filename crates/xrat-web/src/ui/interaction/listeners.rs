use super::Interactions;
use wasm_bindgen::{JsCast, closure::Closure};

pub(super) struct GlobalListeners {
    _listeners: Vec<Listener>,
}

struct Listener {
    target: web_sys::EventTarget,
    name: &'static str,
    capture: bool,
    callback: Closure<dyn FnMut(web_sys::Event)>,
}

impl Listener {
    fn new(
        target: web_sys::EventTarget,
        name: &'static str,
        capture: bool,
        callback: impl FnMut(web_sys::Event) + 'static,
    ) -> Self {
        let callback = Closure::wrap(Box::new(callback) as Box<dyn FnMut(web_sys::Event)>);
        let _ = target.add_event_listener_with_callback_and_bool(
            name,
            callback.as_ref().unchecked_ref(),
            capture,
        );
        Self {
            target,
            name,
            capture,
            callback,
        }
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        let _ = self.target.remove_event_listener_with_callback_and_bool(
            self.name,
            self.callback.as_ref().unchecked_ref(),
            self.capture,
        );
    }
}

impl GlobalListeners {
    pub fn new(interactions: Interactions) -> Self {
        let mut listeners = Vec::new();
        if let Some(document) = super::dom::document() {
            for name in ["click", "focusin"] {
                let interactions = interactions.clone();
                listeners.push(Listener::new(
                    document.clone().into(),
                    name,
                    false,
                    move |event| interactions.outside(&event),
                ));
            }
            let keys = interactions.clone();
            listeners.push(Listener::new(
                document.clone().into(),
                "keydown",
                false,
                move |event| {
                    if let Some(event) = event.dyn_ref::<web_sys::KeyboardEvent>() {
                        keys.keydown(event);
                    }
                },
            ));
            let scroll = interactions.clone();
            listeners.push(Listener::new(
                document.into(),
                "scroll",
                true,
                move |event| scroll.scroll(&event),
            ));
        }
        if let Some(window) = web_sys::window() {
            listeners.push(Listener::new(window.into(), "resize", false, move |_| {
                interactions.close_popup(false);
                interactions.close_tooltip();
            }));
        }
        Self {
            _listeners: listeners,
        }
    }
}
