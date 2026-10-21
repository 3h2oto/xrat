use dioxus::prelude::*;
use std::{cell::RefCell, rc::Rc};

use super::{Floating, Overlay, dom, listeners::GlobalListeners};

#[derive(Clone, Default)]
pub(crate) struct Interactions(pub(super) Rc<RefCell<State>>);

#[derive(Default)]
pub(super) struct State {
    pub popup: Option<Floating>,
    pub tooltip: Option<Floating>,
    pub overlays: Vec<Overlay>,
    pub isolation: super::overlay::Isolation,
    restoring_focus: bool,
}

pub(crate) fn use_interactions() {
    let interactions = use_context_provider(Interactions::default);
    use_hook(move || Rc::new(GlobalListeners::new(interactions)));
}

impl Interactions {
    pub fn close_floating(&self, control: &Floating, restore: bool) {
        let popup = self
            .0
            .borrow()
            .popup
            .as_ref()
            .is_some_and(|active| active.same(control));
        let tooltip = self
            .0
            .borrow()
            .tooltip
            .as_ref()
            .is_some_and(|active| active.same(control));
        if popup {
            self.close_popup(restore);
        }
        if tooltip {
            self.close_tooltip();
        }
    }
    pub fn open_popup(&self, control: Floating, focus: bool) {
        if self.0.borrow().restoring_focus {
            return;
        }
        let same = self
            .0
            .borrow()
            .popup
            .as_ref()
            .is_some_and(|active| active.same(&control));
        if same {
            return;
        }
        self.close_popup(false);
        self.close_tooltip();
        if control.disabled() {
            return;
        }
        self.0.borrow_mut().popup = Some(control.clone());
        control.set_open(true, focus);
    }

    pub fn close_popup(&self, restore: bool) {
        let control = self.0.borrow_mut().popup.take();
        if let Some(control) = control {
            let restoring = self.0.borrow().restoring_focus;
            self.0.borrow_mut().restoring_focus = true;
            control.set_open(false, false);
            if restore {
                control.restore_focus();
            }
            self.0.borrow_mut().restoring_focus = restoring;
        }
    }

    pub fn open_tooltip(&self, control: Floating) {
        if self.0.borrow().popup.is_some() || self.0.borrow().restoring_focus {
            return;
        }
        let same = self
            .0
            .borrow()
            .tooltip
            .as_ref()
            .is_some_and(|active| active.same(&control));
        if same {
            return;
        }
        self.close_tooltip();
        self.0.borrow_mut().tooltip = Some(control.clone());
        control.set_open(true, false);
    }

    pub fn close_tooltip(&self) {
        let control = self.0.borrow_mut().tooltip.take();
        if let Some(control) = control {
            control.set_open(false, false);
        }
    }

    pub fn detach_floating(&self, control: &Floating) {
        let popup = self
            .0
            .borrow()
            .popup
            .as_ref()
            .is_some_and(|active| active.same(control));
        let tooltip = self
            .0
            .borrow()
            .tooltip
            .as_ref()
            .is_some_and(|active| active.same(control));
        if popup {
            self.close_popup(false);
        }
        if tooltip {
            self.close_tooltip();
        }
    }

    pub fn open_overlay(&self, overlay: Overlay) {
        if *overlay.open.peek() {
            return;
        }
        self.close_popup(false);
        self.close_tooltip();
        overlay.capture_origin();
        self.0.borrow_mut().overlays.push(overlay.clone());
        overlay.set_open(true);
    }

    pub fn close_overlay(&self, overlay: &Overlay, notify: bool) {
        let was_open = *overlay.open.peek();
        if !was_open {
            return;
        }
        self.close_popup(false);
        self.close_tooltip();
        self.0
            .borrow_mut()
            .overlays
            .retain(|active| !active.same(overlay));
        overlay.set_open(false);
        self.sync_isolation();
        overlay.restore_focus();
        if notify {
            overlay.notify_close();
        }
    }

    pub fn overlay_open(&self) -> bool {
        !self.0.borrow().overlays.is_empty()
    }

    pub fn sync_isolation(&self) {
        let (mut isolation, overlays) = {
            let mut state = self.0.borrow_mut();
            (std::mem::take(&mut state.isolation), state.overlays.clone())
        };
        isolation.sync(&overlays);
        self.0.borrow_mut().isolation = isolation;
    }

    pub(super) fn outside(&self, event: &web_sys::Event) {
        let target = dom::target(event);
        let popup = self.0.borrow().popup.clone();
        if let Some(popup) = popup {
            let outside = {
                let nodes = popup.nodes.borrow();
                !dom::contains(nodes.content.as_ref(), target.as_ref())
                    && !dom::contains(nodes.trigger.as_ref(), target.as_ref())
            };
            if outside {
                self.close_popup(false);
            }
        }
        let tooltip = self.0.borrow().tooltip.clone();
        if let Some(tooltip) = tooltip {
            let outside = !dom::contains(tooltip.nodes.borrow().trigger.as_ref(), target.as_ref());
            if outside {
                self.close_tooltip();
            }
        }
    }

    pub(super) fn keydown(&self, event: &web_sys::KeyboardEvent) {
        if event.default_prevented() {
            return;
        }
        if event.key() == "Escape" {
            let popup = self.0.borrow().popup.is_some();
            let tooltip = self.0.borrow().tooltip.is_some();
            let overlay = self.0.borrow().overlays.last().cloned();
            if popup {
                self.close_popup(true);
            } else if tooltip {
                self.close_tooltip();
            } else if let Some(overlay) = overlay {
                self.close_overlay(&overlay, true);
            } else {
                return;
            }
            event.prevent_default();
            event.stop_propagation();
        } else if event.key() == "Tab" {
            self.close_popup(true);
            let overlay = self.0.borrow().overlays.last().cloned();
            if let Some(overlay) = overlay {
                overlay.trap_focus(event);
            }
        }
    }

    pub(super) fn scroll(&self, event: &web_sys::Event) {
        self.close_tooltip();
        let popup = self.0.borrow().popup.clone();
        if let Some(popup) = popup {
            let inside = dom::contains(
                popup.nodes.borrow().content.as_ref(),
                dom::target(event).as_ref(),
            );
            if !inside {
                self.close_popup(false);
            }
        }
    }
}
