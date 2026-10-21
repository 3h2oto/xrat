use dioxus::prelude::*;
use wasm_bindgen::JsCast;

pub(super) fn document() -> Option<web_sys::Document> {
    web_sys::window()?.document()
}

pub(super) fn mounted(event: MountedEvent) -> Option<web_sys::HtmlElement> {
    event
        .data()
        .downcast::<web_sys::Element>()?
        .clone()
        .dyn_into()
        .ok()
}

pub(super) fn controls(root: &web_sys::Element, selector: &str) -> Vec<web_sys::HtmlElement> {
    let Ok(nodes) = root.query_selector_all(selector) else {
        return Vec::new();
    };
    (0..nodes.length())
        .filter_map(|index| nodes.item(index)?.dyn_into().ok())
        .collect()
}

pub(super) fn active_element() -> Option<web_sys::Element> {
    document()?.active_element()
}

pub(super) fn focus(element: &web_sys::HtmlElement) {
    if element.is_connected() {
        let _ = element.focus();
    }
}

pub(super) fn target(event: &web_sys::Event) -> Option<web_sys::Node> {
    event.target()?.dyn_into().ok()
}

pub(super) fn contains(
    element: Option<&web_sys::HtmlElement>,
    target: Option<&web_sys::Node>,
) -> bool {
    element.is_some_and(|element| element.contains(target))
}

pub(super) fn show_popover(content: &web_sys::HtmlElement) {
    if !content.matches(":popover-open").unwrap_or(false) {
        let _ = content.show_popover();
    }
}

pub(super) fn hide_popover(content: &web_sys::HtmlElement) {
    if content.matches(":popover-open").unwrap_or(false) {
        let _ = content.hide_popover();
    }
}
