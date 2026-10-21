use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use std::{cell::RefCell, rc::Rc};
use tw_merge::tw_merge;
use wasm_bindgen::JsCast;

struct ResizeDrag {
    handle: web_sys::Element,
    wrapper: web_sys::HtmlElement,
    body: web_sys::HtmlElement,
    user_select: String,
    pointer: i32,
    start_x: f64,
    width: f64,
    minimum: f64,
    maximum: f64,
}

impl Drop for ResizeDrag {
    fn drop(&mut self) {
        let _ = self
            .body
            .style()
            .set_property("user-select", &self.user_select);
        if self.handle.has_pointer_capture(self.pointer) {
            let _ = self.handle.release_pointer_capture(self.pointer);
        }
    }
}

fn limit(wrapper: &web_sys::Element, attribute: &str, fallback: f64) -> f64 {
    wrapper
        .get_attribute(attribute)
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| value.is_finite())
        .unwrap_or(fallback)
}

fn finish(drag: &Rc<RefCell<Option<ResizeDrag>>>, pointer: i32, persist: bool) {
    if drag
        .borrow()
        .as_ref()
        .is_none_or(|drag| drag.pointer != pointer)
    {
        return;
    }
    let finished = drag.borrow_mut().take();
    if persist && let Some(drag) = &finished {
        let width = drag
            .wrapper
            .query_selector("aside")
            .ok()
            .flatten()
            .map(|aside| aside.get_bounding_client_rect().width());
        if let Some(width) = width
            && let Some(storage) =
                web_sys::window().and_then(|window| window.local_storage().ok().flatten())
        {
            let _ = storage.set_item("sidenav-width", &width.to_string());
        }
    }
    drop(finished);
}

pub(super) fn resize_handle(class: Option<String>) -> Element {
    let mut handle = use_signal(|| None::<web_sys::HtmlElement>);
    let drag = use_hook(|| Rc::new(RefCell::new(None::<ResizeDrag>)));
    use_drop({
        let drag = drag.clone();
        move || {
            let finished = drag.borrow_mut().take();
            drop(finished);
        }
    });
    rsx! {
        div {
            class: tw_merge!("absolute top-0 right-0 w-1 h-full z-50 cursor-col-resize hover:bg-sidenav-border transition-colors touch-none", class.as_deref().unwrap_or("")),
            onmounted: move |event| {
                let node = event.data().downcast::<web_sys::Element>().cloned().and_then(|node| node.dyn_into::<web_sys::HtmlElement>().ok());
                if let Some(wrapper) = node.as_ref().and_then(|node| node.closest("[data-name=SidenavWrapper]").ok().flatten()).and_then(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
                    && let Some(width) = web_sys::window().and_then(|window| window.local_storage().ok().flatten()).and_then(|storage| storage.get_item("sidenav-width").ok().flatten()).and_then(|value| value.parse::<f64>().ok()).filter(|value| value.is_finite()) {
                    let minimum = limit(&wrapper, "data-min-width", 160.0);
                    let maximum = limit(&wrapper, "data-max-width", 480.0).max(minimum);
                    let _ = wrapper.style().set_property("--sidenav-width", &format!("{}px", width.clamp(minimum, maximum)));
                }
                handle.set(node);
            },
            onpointerdown: {
                let drag = drag.clone();
                move |event: PointerEvent| {
                    if drag.borrow().is_some() { return; }
                    if !event.is_primary() || event.trigger_button() != Some(MouseButton::Primary) { return; }
                    let node = handle.peek().clone();
                    let Some(node) = node else { return };
                    let Some(wrapper) = node.closest("[data-name=SidenavWrapper]").ok().flatten().and_then(|node| node.dyn_into::<web_sys::HtmlElement>().ok()) else { return };
                    let Some(body) = web_sys::window().and_then(|window| window.document()).and_then(|document| document.body()) else { return };
                    if node.set_pointer_capture(event.pointer_id()).is_err() { return; }
                    event.prevent_default();
                    let minimum = limit(&wrapper, "data-min-width", 160.0);
                    let maximum = limit(&wrapper, "data-max-width", 480.0).max(minimum);
                    let width = wrapper.query_selector("aside").ok().flatten().map_or(256.0, |aside| aside.get_bounding_client_rect().width());
                    let user_select = body.style().get_property_value("user-select").unwrap_or_default();
                    let _ = body.style().set_property("user-select", "none");
                    *drag.borrow_mut() = Some(ResizeDrag { handle: node.into(), wrapper, body, user_select, pointer: event.pointer_id(), start_x: event.client_coordinates().x, width, minimum, maximum });
                }
            },
            onpointermove: {
                let drag = drag.clone();
                move |event: PointerEvent| {
                    if let Some(drag) = drag.borrow().as_ref() && drag.pointer == event.pointer_id() {
                        let width = (drag.width + event.client_coordinates().x - drag.start_x).clamp(drag.minimum, drag.maximum);
                        let _ = drag.wrapper.style().set_property("--sidenav-width", &format!("{width}px"));
                    }
                }
            },
            onpointerup: { let drag = drag.clone(); move |event: PointerEvent| finish(&drag, event.pointer_id(), true) },
            onpointercancel: { let drag = drag.clone(); move |event: PointerEvent| finish(&drag, event.pointer_id(), false) },
            onlostpointercapture: move |event: PointerEvent| finish(&drag, event.pointer_id(), false),
        }
    }
}
