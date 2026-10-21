use dioxus::prelude::*;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsCast;

use super::{Interactions, dom};

const FOCUSABLE: &str = "button:not([disabled]):not([hidden]),input:not([disabled]),textarea:not([disabled]),select:not([disabled]),a[href],[tabindex=\"0\"]";

#[derive(Clone)]
pub(crate) struct Overlay {
    pub open: Signal<bool>,
    nodes: Rc<RefCell<Nodes>>,
    on_close: Rc<RefCell<Option<EventHandler>>>,
}

#[derive(Default)]
struct Nodes {
    content: Option<web_sys::HtmlElement>,
    backdrop: Option<web_sys::HtmlElement>,
    origin: Option<web_sys::HtmlElement>,
}

pub(crate) fn use_overlay() -> Overlay {
    let interactions = use_context::<Interactions>();
    let overlay = Overlay {
        open: use_signal(|| false),
        nodes: use_hook(|| Rc::new(RefCell::new(Nodes::default()))),
        on_close: use_hook(|| Rc::new(RefCell::new(None))),
    };
    use_drop({
        let overlay = overlay.clone();
        move || interactions.close_overlay(&overlay, false)
    });
    overlay
}

impl Overlay {
    pub(super) fn same(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.nodes, &other.nodes)
    }

    pub fn configure(&self, on_close: Option<EventHandler>) {
        *self.on_close.borrow_mut() = on_close;
    }

    pub fn mount_content(&self, event: MountedEvent) {
        self.nodes.borrow_mut().content = dom::mounted(event);
    }

    pub fn mount_backdrop(&self, event: MountedEvent) {
        self.nodes.borrow_mut().backdrop = dom::mounted(event);
    }

    pub fn sync(&self, interactions: &Interactions) {
        if !*self.open.peek() {
            return;
        }
        interactions.sync_isolation();
        let content = self.nodes.borrow().content.clone();
        if let Some(content) = content {
            let active = dom::active_element();
            if !content.contains(active.as_ref().map(|element| element.as_ref())) {
                let first = focusable(&content).into_iter().next().unwrap_or(content);
                dom::focus(&first);
            }
        }
    }

    pub(super) fn set_open(&self, open: bool) {
        let mut signal = self.open;
        signal.set(open);
    }

    pub(super) fn capture_origin(&self) {
        self.nodes.borrow_mut().origin =
            dom::active_element().and_then(|element| element.dyn_into().ok());
    }

    pub(super) fn restore_focus(&self) {
        let origin = self.nodes.borrow_mut().origin.take();
        if let Some(origin) = origin {
            dom::focus(&origin);
        }
    }

    pub(super) fn notify_close(&self) {
        let handler = *self.on_close.borrow();
        if let Some(handler) = handler {
            handler.call(());
        }
    }

    pub(super) fn trap_focus(&self, event: &web_sys::KeyboardEvent) {
        let content = self.nodes.borrow().content.clone();
        let Some(content) = content else { return };
        let nodes = focusable(&content);
        let active = dom::active_element();
        let same = |node: &web_sys::HtmlElement| {
            active
                .as_ref()
                .is_some_and(|active| node.is_same_node(Some(active)))
        };
        let target = if nodes.is_empty() {
            Some(&content)
        } else if event.shift_key() && (nodes.first().is_some_and(same) || same(&content)) {
            nodes.last()
        } else if !event.shift_key() && nodes.last().is_some_and(same) {
            nodes.first()
        } else {
            None
        };
        if let Some(target) = target {
            event.prevent_default();
            dom::focus(target);
        }
    }
}

fn focusable(content: &web_sys::HtmlElement) -> Vec<web_sys::HtmlElement> {
    dom::controls(content, FOCUSABLE)
        .into_iter()
        .filter(|node| {
            node.closest("[inert]").ok().flatten().is_none() && node.get_client_rects().length() > 0
        })
        .collect()
}

#[derive(Default)]
pub(super) struct Isolation {
    inert: Vec<(web_sys::HtmlElement, bool)>,
    overflow: Option<String>,
}

impl Isolation {
    pub fn sync(&mut self, overlays: &[Overlay]) {
        for (element, inert) in self.inert.drain(..) {
            if element.is_connected() {
                element.set_inert(inert);
            }
        }
        let Some(body) = dom::document().and_then(|document| document.body()) else {
            return;
        };
        if overlays.is_empty() {
            if let Some(overflow) = self.overflow.take() {
                let _ = body.style().set_property("overflow", &overflow);
            }
            return;
        }
        if self.overflow.is_none() {
            self.overflow = body.style().get_property_value("overflow").ok();
        }
        let _ = body.style().set_property("overflow", "hidden");
        for (index, overlay) in overlays.iter().enumerate() {
            let nodes = overlay.nodes.borrow();
            if let Some(content) = &nodes.content {
                content.set_inert(false);
                let _ = content
                    .style()
                    .set_property("z-index", &(90 + index * 20).to_string());
            }
            if let Some(backdrop) = &nodes.backdrop {
                let _ = backdrop
                    .style()
                    .set_property("z-index", &(80 + index * 20).to_string());
            }
        }
        let Some(top) = overlays.last() else { return };
        let nodes = top.nodes.borrow();
        let Some(mut branch) = nodes.content.clone().map(web_sys::Element::from) else {
            return;
        };
        let backdrop = nodes.backdrop.as_ref();
        while let Some(parent) = branch.parent_element() {
            let children = parent.children();
            for index in 0..children.length() {
                let Some(sibling) = children
                    .item(index)
                    .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
                else {
                    continue;
                };
                if sibling.is_same_node(Some(&branch))
                    || backdrop.is_some_and(|backdrop| sibling.contains(Some(backdrop)))
                    || matches!(sibling.tag_name().as_str(), "SCRIPT" | "STYLE")
                {
                    continue;
                }
                self.inert.push((sibling.clone(), sibling.inert()));
                sibling.set_inert(true);
            }
            branch = parent;
        }
    }
}
