const TOOLTIP_CONTENT_STYLES: &str = concat!(
    "[&&]:transition-none [&&]:animate-none [&&]:[z-index:var(--layer-tooltip)] [&&]:pointer-events-none ",
    "[&&]:fixed [&&]:[left:0] [&&]:[top:0] [&&]:[right:auto] [&&]:[bottom:auto] [&&]:m-0 [&&]:invisible ",
    "[&&]:[max-width:calc(100vw_-_16px)] [&&]:whitespace-normal [&&]:[overflow-wrap:anywhere] [&&[popover]]:m-0 ",
    "[&&[popover]]:[inset:auto] [&&[popover]]:[width:max-content] [&&[popover]]:[max-width:calc(100vw_-_16px)] ",
    "[&&[popover]]:[border:0] [&&::backdrop]:[background:transparent] [&&::backdrop]:pointer-events-none ",
);

use super::interaction::{Floating, Interactions, Placement, Side, use_floating};
use dioxus::prelude::*;
use tw_merge::tw_merge;

#[derive(Clone)]
struct TooltipContext(Floating);

#[derive(Default, Clone, PartialEq, Eq)]
pub enum TooltipPosition {
    #[default]
    Top,
    Left,
    Right,
    Bottom,
}

#[component]
pub fn Tooltip(
    #[props(into, optional)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let floating = use_floating(Placement::Tooltip(Side::Top));
    let interactions = use_context::<Interactions>();
    provide_context(TooltipContext(floating.clone()));
    let mounted = floating.clone();
    let merged = tw_merge!(
        "inline-block relative mx-0 whitespace-nowrap transition-all duration-300 ease-in-out group/tooltip my-[5px]",
        class.as_deref().unwrap_or("")
    );
    rsx! {
        div { "data-name": "Tooltip", class: "{merged} [&&]:transition-none [&&]:animate-none",
            onmounted: move |event| mounted.mount_trigger(event, false),
            onmouseenter: {
                let floating = floating.clone(); let interactions = interactions.clone();
                move |_| interactions.open_tooltip(floating.clone())
            },
            onfocusin: {
                let floating = floating.clone(); let interactions = interactions.clone();
                move |_| interactions.open_tooltip(floating.clone())
            },
            onmouseleave: move |_| {
                if !floating.focus_within() { interactions.close_floating(&floating, false); }
            },
            ..attributes, {children}
        }
    }
}

#[component]
pub fn TooltipContent(
    #[props(into, optional)] class: Option<String>,
    #[props(default = TooltipPosition::default())] position: TooltipPosition,
    #[props(into, optional)] id: Option<String>,
    children: Element,
) -> Element {
    let TooltipContext(floating) = use_context();
    let mounted = floating.clone();
    let open = floating.open;
    floating.set_placement(Placement::Tooltip(match position {
        TooltipPosition::Top => Side::Top,
        TooltipPosition::Bottom => Side::Bottom,
        TooltipPosition::Left => Side::Left,
        TooltipPosition::Right => Side::Right,
    }));
    use_effect(move || {
        let _ = open();
        floating.sync();
    });
    const SHARED: &str = "absolute opacity-0 transition-all duration-300 ease-in-out pointer-events-none group-hover/tooltip:opacity-100 group-hover/tooltip:pointer-events-auto group-focus-within/tooltip:opacity-100 group-focus-within/tooltip:pointer-events-auto z-[1000000]";

    let (pos_class, arrow_pos_class, pos_str) = match position {
        TooltipPosition::Top => (
            "left-1/2 bottom-full mb-1 -ml-2.5",
            "left-1/2 bottom-full -mb-2 border-t-foreground/90",
            "Top",
        ),
        TooltipPosition::Right => (
            "bottom-1/2 left-full ml-2.5 -mb-3.5",
            "bottom-1/2 left-full -mr-0.5 -mb-1 border-r-foreground/90",
            "Right",
        ),
        TooltipPosition::Bottom => (
            "left-1/2 top-full mt-1 -ml-2.5",
            "left-1/2 top-full -mt-2 border-b-foreground/90",
            "Bottom",
        ),
        TooltipPosition::Left => (
            "bottom-1/2 right-full mr-2.5 -mb-3.5",
            "bottom-1/2 right-full -mb-1 -ml-0.5 border-l-foreground/90",
            "Left",
        ),
    };

    let arrow_class = tw_merge!(
        SHARED,
        "bg-transparent border-transparent border-6",
        arrow_pos_class
    );
    let content_class = tw_merge!(
        SHARED,
        "py-2 px-2.5 text-xs whitespace-nowrap shadow-lg text-background bg-foreground/90",
        pos_class,
        class.as_deref().unwrap_or("")
    );

    rsx! {
        div {
            "data-name": "TooltipArrow",
            "data-position": "{pos_str}",
            class: "{arrow_class} [&&]:hidden",
        }
        div {
            id,
            role: "tooltip",
            "popover": "manual",
            "data-name": "TooltipContent",
            onmounted: move |event| mounted.mount_content(event),
            "data-position": "{pos_str}",
            class: "{content_class} {TOOLTIP_CONTENT_STYLES}",
            {children}
        }
    }
}

/// Kept for compatibility; interaction state is provided at the app root.
#[component]
pub fn TooltipProvider() -> Element {
    rsx! {}
}
