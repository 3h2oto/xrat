use super::*;
use dioxus::prelude::*;

fn tooltip_lifecycle() -> Element {
    let interactions = use_context_provider(Interactions::default);
    let first = use_floating(Placement::Tooltip(Side::Top));
    let second = use_floating(Placement::Tooltip(Side::Right));
    use_hook(move || {
        interactions.open_tooltip(first.clone());
        assert!(*first.open.peek());
        interactions.open_tooltip(second.clone());
        assert!(!*first.open.peek());
        assert!(*second.open.peek());
        interactions.detach_floating(&first);
        assert!(*second.open.peek());
        interactions.close_floating(&second, false);
        assert!(!*second.open.peek());
    });
    rsx! {}
}

fn popup_precedence() -> Element {
    let interactions = use_context_provider(Interactions::default);
    let popup = use_floating(Placement::Select);
    let tooltip = use_floating(Placement::Tooltip(Side::Top));
    use_hook(move || {
        let mut open = popup.open;
        open.set(true);
        interactions.0.borrow_mut().popup = Some(popup.clone());
        interactions.open_tooltip(tooltip.clone());
        assert!(!*tooltip.open.peek());
        interactions.close_popup(true);
        assert!(!*popup.open.peek());
        assert!(interactions.0.borrow().popup.is_none());
        interactions.open_tooltip(tooltip.clone());
        assert!(*tooltip.open.peek());
        interactions.close_floating(&popup, true);
        assert!(*tooltip.open.peek());
        interactions.detach_floating(&tooltip);
        assert!(!*tooltip.open.peek());
    });
    rsx! {}
}

#[test]
fn replacing_and_detaching_tooltips_preserves_the_active_control() {
    let mut dom = VirtualDom::new(tooltip_lifecycle);
    dom.rebuild_in_place();
}

#[test]
fn popups_suppress_tooltips_and_stale_controls_cannot_close_them() {
    let mut dom = VirtualDom::new(popup_precedence);
    dom.rebuild_in_place();
}
