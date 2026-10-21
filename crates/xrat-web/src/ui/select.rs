const SELECT_CONTENT_STYLES: &str = concat!(
    "[&&]:invisible [&&]:transition-none [&&]:animate-none [&&]:[transform:none] [&&]:[scale:1] ",
    "[&&]:[padding:5px] [&&]:[border-radius:8px] ",
    "[&&]:[border-color:color-mix(in_srgb,_var(--border)_65%,_var(--panel))] ",
    "[&&]:[box-shadow:0_6px_18px_#0003,_0_1px_3px_#0002] [&&]:[z-index:var(--layer-popup)] [&&]:fixed ",
    "[&&]:[max-width:calc(100vw_-_16px)] [&&[data-state=open]]:visible [&&[popover]]:m-0 ",
    "[&&[popover]]:[inset:auto] [&&::backdrop]:[background:transparent] [&&::backdrop]:pointer-events-none ",
);

const SELECT_OPTION_STYLES: &str = concat!(
    "[&&:focus-visible]:[outline:2px_solid_var(--ring)] [&&:focus-visible]:[outline-offset:-2px] ",
    "[&&:focus-visible]:bg-accent [&&[aria-selected=true]]:bg-accent [&&]:[min-height:30px] ",
    "[&&]:[padding:6px_8px] [&&]:[border-radius:4px] [&&]:[font-size:12px] [&&]:[line-height:18px] ",
    "[&&]:[gap:9px] ",
);

use super::interaction::{Floating, Interactions, Placement, use_floating};
use dioxus::prelude::*;
use icons::{Check, ChevronDown, ChevronUp};
use tw_merge::tw_merge;

#[component]
pub fn SelectLabel(#[props(into, optional)] class: Option<String>, children: Element) -> Element {
    let merged = tw_merge!(
        "px-2 py-1.5 text-sm font-medium data-inset:pl-8 mb-1",
        class.as_deref().unwrap_or("")
    );
    rsx! {
        span { "data-name": "SelectLabel", class: "{merged}", {children} }
    }
}

use crate::hooks::use_can_scroll_vertical::use_can_scroll_vertical;
use crate::hooks::use_random::use_random_id_for;

/* ========================================================== */
/*                     ✨ CONTEXT ✨                          */
/* ========================================================== */

#[derive(Clone)]
struct SelectContext {
    target_id: String,
    value: Signal<Option<String>>,
    on_change: EventHandler<Option<String>>,
    floating: Floating,
}

/* ========================================================== */
/*                     ✨ SELECT ✨                           */
/* ========================================================== */

#[component]
pub fn Select(
    children: Element,
    #[props(into, optional)] class: Option<String>,
    #[props(into, optional)] default_value: Option<String>,
    #[props(into, optional)] selected_value: Option<String>,
    #[props(optional)] on_change: EventHandler<Option<String>>,
) -> Element {
    let target_id = use_hook(|| use_random_id_for("select"));
    let floating = use_floating(Placement::Select);
    let interactions = use_context::<Interactions>();
    let mut value = use_signal(|| selected_value.clone().or(default_value));
    use_effect(use_reactive!(|(selected_value,)| {
        if let Some(next) = selected_value.clone()
            && value.peek().as_ref() != Some(&next)
        {
            value.set(Some(next));
        }
    }));
    provide_context(SelectContext {
        target_id,
        value,
        on_change,
        floating: floating.clone(),
    });

    let merged = tw_merge!("relative w-fit", class.as_deref().unwrap_or(""));
    rsx! {
        div { "data-name": "Select", class: "{merged}",
            onkeydown: move |event| floating.keydown(event, &interactions),
            {children}
        }
    }
}

/* ========================================================== */
/*                   ✨ SELECT TRIGGER ✨                     */
/* ========================================================== */

#[component]
pub fn SelectTrigger(
    children: Element,
    #[props(into, optional)] class: Option<String>,
    #[props(into, optional)] id: Option<String>,
    #[props(optional)] disabled: bool,
) -> Element {
    let ctx = use_context::<SelectContext>();
    let interactions = use_context::<Interactions>();
    let mounted = ctx.floating.clone();
    let floating = ctx.floating.clone();
    let merged = tw_merge!(
        "w-full p-2 h-9 inline-flex items-center justify-between text-sm font-medium whitespace-nowrap rounded-md transition-colors focus:outline-none focus:ring-1 focus:ring-ring focus-visible:outline-hidden focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 [&_svg:not(:last-child)]:mr-2 [&_svg:not(:first-child)]:ml-2 [&_svg:not([class*='size-'])]:size-4 border bg-background border-input hover:bg-accent hover:text-accent-foreground",
        class.as_deref().unwrap_or("")
    );
    rsx! {
        button {
            r#type: "button",
            id,
            disabled,
            role: "combobox",
            aria_haspopup: "listbox",
            aria_expanded: (ctx.floating.open)().to_string(),
            aria_controls: ctx.target_id.clone(),
            class: "{merged}",
            "data-name": "SelectTrigger",
            onmounted: move |event| mounted.mount_trigger(event, false),
            onclick: move |_| {
                if *floating.open.peek() { interactions.close_floating(&floating, false); }
                else { interactions.open_popup(floating.clone(), true); }
            },
            tabindex: "0",
            {children}
            ChevronDown { class: "text-muted-foreground" }
        }
    }
}

/* ========================================================== */
/*                   ✨ SELECT VALUE ✨                       */
/* ========================================================== */

#[component]
pub fn SelectValue(
    #[props(into, optional)] placeholder: Option<String>,
    #[props(into, optional)] display_value: Option<String>,
) -> Element {
    let ctx = use_context::<SelectContext>();
    let text = display_value
        .or_else(|| ctx.value.read().clone())
        .unwrap_or_else(|| placeholder.unwrap_or_default());

    rsx! {
        span {
            "data-name": "SelectValue",
            class: "text-sm text-muted-foreground truncate",
            {text}
        }
    }
}

/* ========================================================== */
/*                   ✨ SELECT CONTENT ✨                     */
/* ========================================================== */

#[derive(Clone, Copy, PartialEq, Eq, Default, strum::IntoStaticStr)]
pub enum SelectPosition {
    #[default]
    Below,
    Above,
}

impl SelectPosition {
    fn as_str(self) -> &'static str {
        self.into()
    }
}

#[component]
pub fn SelectContent(
    children: Element,
    #[props(into, optional)] class: Option<String>,
    #[props(default = SelectPosition::Below)] position: SelectPosition,
) -> Element {
    let ctx = use_context::<SelectContext>();
    let mounted = ctx.floating.clone();
    let open = ctx.floating.open;
    use_effect({
        let floating = ctx.floating.clone();
        move || {
            let _ = open();
            floating.sync();
        }
    });
    let (on_scroll, can_scroll_up, can_scroll_down) = use_can_scroll_vertical();

    let merged = tw_merge!(
        "w-[150px] overflow-auto z-50 p-1 rounded-md border bg-card shadow-md h-fit max-h-[300px] absolute top-[calc(100%+4px)] left-0 data-[position=Above]:top-auto data-[position=Above]:bottom-[calc(100%+4px)] transition-all duration-200 data-[state=closed]:opacity-0 data-[state=closed]:scale-95 data-[state=open]:opacity-100 data-[state=open]:scale-100 data-[state=closed]:data-[position=Below]:origin-top data-[state=open]:data-[position=Below]:origin-top data-[state=closed]:data-[position=Above]:origin-bottom data-[state=open]:data-[position=Above]:origin-bottom [scrollbar-width:none] [&::-webkit-scrollbar]:hidden",
        class.as_deref().unwrap_or("")
    );

    rsx! {
        div {
            "data-name": "SelectContent",
            "popover": "manual",
            id: "{ctx.target_id}",
            class: "{merged} {SELECT_CONTENT_STYLES}",
            "data-target": "target__select",
            "data-state": if open() { "open" } else { "closed" },
            "data-position": position.as_str(),
            inert: !open(),
            style: if open() { "pointer-events:auto" } else { "pointer-events:none" },
            onmounted: move |event| mounted.mount_content(event),
            onscroll: on_scroll,
            div { class: if can_scroll_up() { "sticky -top-1 z-10 flex items-center justify-center py-1 bg-card" } else { "hidden" },
                ChevronUp { class: "size-4 text-muted-foreground" }
            }
            {children}
            div { class: if can_scroll_down() { "sticky -bottom-1 z-10 flex items-center justify-center py-1 bg-card" } else { "hidden" },
                ChevronDown { class: "size-4 text-muted-foreground" }
            }
        }
    }
}

/* ========================================================== */
/*                    ✨ SELECT GROUP ✨                      */
/* ========================================================== */

#[component]
pub fn SelectGroup(
    children: Element,
    #[props(into, optional)] class: Option<String>,
    #[props(into, optional)] aria_label: Option<String>,
) -> Element {
    let merged = tw_merge!("group", class.as_deref().unwrap_or(""));
    rsx! {
        ul {
            "data-name": "SelectGroup",
            role: "listbox",
            "aria-label": aria_label.as_deref().unwrap_or("Select options"),
            class: "{merged}",
            {children}
        }
    }
}

/* ========================================================== */
/*                   ✨ SELECT OPTION ✨                      */
/* ========================================================== */

#[component]
pub fn SelectOption(
    children: Element,
    #[props(into, optional)] class: Option<String>,
    #[props(into, optional)] value: Option<String>,
) -> Element {
    let ctx = use_context::<SelectContext>();
    let interactions = use_context::<Interactions>();
    let mut value_signal = ctx.value;
    let on_change = ctx.on_change;
    let val = value.clone();
    let val_check = value;

    let is_selected = move || *value_signal.read() == val_check;

    let merged = tw_merge!(
        "group inline-flex gap-2 items-center w-full rounded-sm px-2 py-1.5 text-sm cursor-pointer no-underline transition-colors duration-200 text-popover-foreground hover:bg-accent hover:text-accent-foreground [&_svg:not([class*='size-'])]:size-4",
        class.as_deref().unwrap_or("")
    );

    rsx! {
        li {
            "data-name": "SelectOption",
            class: "{merged} {SELECT_OPTION_STYLES}",
            role: "option",
            tabindex: "0",
            "aria-selected": if is_selected() { "true" } else { "false" },
            "data-select-option": "true",
            onclick: move |_| {
                value_signal.set(val.clone());
                on_change.call(val.clone());
                interactions.close_floating(&ctx.floating, true);
            },
            {children}
            Check { class: "ml-auto opacity-0 size-4 text-muted-foreground group-aria-selected:opacity-100" }
        }
    }
}
