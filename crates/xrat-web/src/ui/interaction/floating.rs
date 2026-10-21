use dioxus::prelude::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::JsCast;

use super::{Interactions, Placement, dom, placement};

#[derive(Clone)]
pub(crate) struct Floating {
    pub open: Signal<bool>,
    pub(super) nodes: Rc<RefCell<Nodes>>,
    placement: Rc<Cell<Placement>>,
    focus_on_open: Rc<Cell<bool>>,
}

#[derive(Default)]
pub(super) struct Nodes {
    pub content: Option<web_sys::HtmlElement>,
    pub trigger: Option<web_sys::HtmlElement>,
}

pub(crate) fn use_floating(placement: Placement) -> Floating {
    let interactions = use_context::<Interactions>();
    let control = Floating {
        open: use_signal(|| false),
        nodes: use_hook(|| Rc::new(RefCell::new(Nodes::default()))),
        placement: use_hook(|| Rc::new(Cell::new(placement))),
        focus_on_open: use_hook(|| Rc::new(Cell::new(false))),
    };
    use_drop({
        let control = control.clone();
        move || interactions.detach_floating(&control)
    });
    control
}

impl Floating {
    pub(super) fn same(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.nodes, &other.nodes)
    }

    pub fn mount_trigger(&self, event: MountedEvent, as_child: bool) {
        let trigger = dom::mounted(event).and_then(|node| {
            if as_child {
                dom::controls(&node, "button,a").into_iter().next()
            } else {
                Some(node)
            }
        });
        self.nodes.borrow_mut().trigger = trigger;
        self.sync();
    }

    pub fn mount_content(&self, event: MountedEvent) {
        self.nodes.borrow_mut().content = dom::mounted(event);
        self.sync();
    }

    pub fn set_placement(&self, placement: Placement) {
        self.placement.set(placement);
    }

    pub fn disabled(&self) -> bool {
        self.nodes
            .borrow()
            .trigger
            .as_ref()
            .is_none_or(|node| node.matches(":disabled").unwrap_or(false))
    }

    pub(super) fn set_open(&self, open: bool, focus: bool) {
        self.focus_on_open.set(focus);
        let mut signal = self.open;
        if *signal.peek() != open {
            signal.set(open);
        }
        if !open {
            self.sync();
        }
    }

    pub fn sync(&self) {
        let (content, trigger) = {
            let nodes = self.nodes.borrow();
            (nodes.content.clone(), nodes.trigger.clone())
        };
        let open = *self.open.peek();
        let tooltip = matches!(self.placement.get(), Placement::Tooltip(_));
        if !tooltip && let Some(trigger) = &trigger {
            let _ = trigger.set_attribute(
                "aria-haspopup",
                if self.placement.get() == Placement::Select {
                    "listbox"
                } else {
                    "menu"
                },
            );
            let _ = trigger.set_attribute("aria-expanded", if open { "true" } else { "false" });
            if let Some(content) = &content {
                let _ = trigger.set_attribute("aria-controls", &content.id());
            }
        }
        let Some(content) = &content else { return };
        if !open {
            dom::hide_popover(content);
            if tooltip {
                let _ = content.style().set_property("opacity", "0");
                let _ = content.style().set_property("visibility", "hidden");
            }
            return;
        }
        let Some(trigger) = &trigger else { return };
        dom::show_popover(content);
        placement::position(content, trigger, self.placement.get());
        if self.focus_on_open.replace(false) {
            let items = options(content);
            let selected = items
                .iter()
                .find(|node| node.get_attribute("aria-selected").as_deref() == Some("true"));
            if let Some(item) = selected.or_else(|| items.first()) {
                dom::focus(item);
            }
        }
    }

    pub(super) fn restore_focus(&self) {
        let trigger = self.nodes.borrow().trigger.clone();
        if let Some(trigger) = trigger {
            dom::focus(&trigger);
        }
    }

    pub fn focus_first(&self) {
        let content = self.nodes.borrow().content.clone();
        if let Some(content) = content
            && let Some(first) = options(&content).first()
        {
            dom::focus(first);
        }
    }

    pub fn keydown(&self, event: KeyboardEvent, interactions: &Interactions) {
        let key = event.key();
        if !*self.open.peek() {
            if matches!(key, Key::ArrowDown | Key::ArrowUp) && !self.disabled() {
                event.prevent_default();
                interactions.open_popup(self.clone(), true);
            }
            return;
        }
        let native = event.data().downcast::<web_sys::KeyboardEvent>().cloned();
        let editable = native
            .as_ref()
            .and_then(|event| event.target())
            .and_then(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
            .is_some_and(|node| {
                node.matches("input,textarea").unwrap_or(false) || node.is_content_editable()
            });
        if editable && key != Key::ArrowDown {
            return;
        }
        let content = self.nodes.borrow().content.clone();
        let Some(content) = content else { return };
        let items = options(&content);
        let active = dom::active_element();
        let index = items.iter().position(|item| {
            active
                .as_ref()
                .is_some_and(|active| item.is_same_node(Some(active)))
        });
        let next = match key {
            Key::Home => items.first(),
            Key::End => items.last(),
            Key::ArrowDown if !items.is_empty() => {
                items.get(index.map_or(0, |index| (index + 1) % items.len()))
            }
            Key::ArrowUp if !items.is_empty() => {
                items.get(index.map_or(items.len() - 1, |index| {
                    (index + items.len() - 1) % items.len()
                }))
            }
            Key::Enter => {
                if let Some(index) = index {
                    event.prevent_default();
                    items[index].click();
                }
                return;
            }
            Key::Character(ref character) if character == " " => {
                if let Some(index) = index {
                    event.prevent_default();
                    items[index].click();
                }
                return;
            }
            Key::Character(ref character)
                if character.chars().count() == 1
                    && !event
                        .modifiers()
                        .intersects(Modifiers::CONTROL | Modifiers::META | Modifiers::ALT) =>
            {
                let start = index.map_or(0, |index| index + 1);
                (0..items.len())
                    .map(|offset| &items[(start + offset) % items.len()])
                    .find(|node| {
                        node.text_content()
                            .unwrap_or_default()
                            .trim()
                            .to_lowercase()
                            .starts_with(&character.to_lowercase())
                    })
            }
            _ => None,
        };
        if let Some(next) = next {
            event.prevent_default();
            dom::focus(next);
            let options = web_sys::ScrollIntoViewOptions::new();
            options.set_block(web_sys::ScrollLogicalPosition::Nearest);
            next.scroll_into_view_with_scroll_into_view_options(&options);
        }
    }

    pub fn leave_hover(&self, interactions: Interactions) {
        let control = self.clone();
        spawn(async move {
            gloo_timers::future::TimeoutFuture::new(120).await;
            let (content, trigger) = {
                let nodes = control.nodes.borrow();
                (nodes.content.clone(), nodes.trigger.clone())
            };
            let hovered = trigger
                .and_then(|trigger| trigger.closest("[data-name=DropdownMenu]").ok().flatten())
                .is_some_and(|root| root.matches(":hover").unwrap_or(false));
            let focused = dom::contains(
                content.as_ref(),
                dom::active_element()
                    .as_ref()
                    .map(|element| element.as_ref()),
            );
            if !hovered && !focused {
                interactions.close_floating(&control, false);
            }
        });
    }

    pub fn focus_within(&self) -> bool {
        dom::contains(
            self.nodes.borrow().trigger.as_ref(),
            dom::active_element()
                .as_ref()
                .map(|element| element.as_ref()),
        )
    }
}

fn options(content: &web_sys::HtmlElement) -> Vec<web_sys::HtmlElement> {
    dom::controls(
        content,
        "[data-select-option],[data-dropdown-close],[role=menuitem],[role=menuitemcheckbox]",
    )
    .into_iter()
    .filter(|node| !node.matches(":disabled").unwrap_or(false))
    .collect()
}
