use std::sync::atomic::{AtomicUsize, Ordering};

use dioxus::prelude::*;
use icons::X;
use tw_merge::tw_merge;

static DIALOG_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, PartialEq)]
struct DialogContext {
    target_id: String,
}

#[must_use]
pub fn use_dialog_trigger_id() -> Option<String> {
    try_consume_context::<DialogContext>().map(|ctx| ctx.target_id)
}

// ---------------------------------------------------------------------------
// Dialog (root)
// ---------------------------------------------------------------------------

#[component]
pub fn Dialog(#[props(into, optional)] class: Option<String>, children: Element) -> Element {
    let dialog_target_id = use_hook(|| {
        let id = DIALOG_COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("dialog_{id}")
    });

    provide_context(DialogContext {
        target_id: dialog_target_id,
    });

    let merged = tw_merge!("w-fit", class.as_deref().unwrap_or(""));

    rsx! {
        div { class: "{merged}", "data-name": "__Dialog", {children} }
    }
}

// ---------------------------------------------------------------------------
// DialogTrigger
// ---------------------------------------------------------------------------

#[component]
pub fn DialogTrigger(#[props(into, optional)] class: Option<String>, children: Element) -> Element {
    let ctx = use_context::<DialogContext>();
    let trigger_id = format!("trigger_{}", ctx.target_id);

    rsx! {
        button {
            class: tw_merge!(
                "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium border bg-background shadow-xs hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2 cursor-pointer",
                class.as_deref().unwrap_or("")
            ),
            r#type: "button",
            id: "{trigger_id}",
            tabindex: "0",
            "data-dialog-trigger": "{ctx.target_id}",
            {children}
        }
    }
}

// ---------------------------------------------------------------------------
// DialogContent
// ---------------------------------------------------------------------------

#[component]
pub fn DialogContent(
    #[props(into, optional)] class: Option<String>,
    #[props(default = true)] show_close_button: bool,
    #[props(default = true)] close_on_backdrop_click: bool,
    #[props(default)] initial_open: bool,
    #[props(default)] alert: bool,
    #[props(optional)] on_close: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let ctx = use_context::<DialogContext>();
    let target_id = ctx.target_id;
    let backdrop_id = format!("{target_id}_backdrop");
    let backdrop_behavior = if close_on_backdrop_click {
        "auto"
    } else {
        "manual"
    };

    let merged = tw_merge!(
        "relative bg-background border rounded-2xl shadow-lg p-6 w-full max-w-[calc(100%-2rem)] max-h-[85vh] fixed top-[50%] left-[50%] translate-x-[-50%] translate-y-[-50%] z-100 transition-all duration-200 data-[state=closed]:opacity-0 data-[state=closed]:scale-95 data-[state=open]:opacity-100 data-[state=open]:scale-100",
        class.as_deref().unwrap_or("")
    );

    let close_button_class = format!(
        "absolute top-4 right-4 p-1 rounded-sm focus:ring-2 focus:ring-offset-2 focus:outline-none [&_svg:not([class*='size-'])]:size-4 focus:ring-ring{}",
        if show_close_button { "" } else { " hidden" }
    );

    rsx! {
        // Backdrop
        div {
            "data-name": "DialogBackdrop",
            id: "{backdrop_id}",
            class: "fixed inset-0 transition-opacity duration-200 pointer-events-none z-60 bg-black/50 data-[state=closed]:opacity-0 data-[state=open]:opacity-100",
            "data-state": "closed",
        }
        // Panel
        div {
            role: if alert { "alertdialog" } else { "dialog" },
            aria_modal: "true",
            tabindex: "-1",
            inert: true,
            aria_labelledby: format!("{target_id}-title"),
            "data-auto-open": initial_open.to_string(),
            "data-name": "DialogContent",
            class: "{merged}",
            id: "{target_id}",
            "data-target": "target__dialog",
            "data-state": "closed",
            "data-backdrop": "{backdrop_behavior}",
            style: "pointer-events: none;",
            button {
                r#type: "button",
                class: "{close_button_class}",
                hidden: !show_close_button,
                "data-dialog-close": "{target_id}",
                aria_label: "Close dialog",
                span { class: "hidden", "Close Dialog" }
                X {}
            }
            button {
                hidden: true,
                r#type: "button",
                "data-overlay-notify-close": "true",
                onclick: move |event| {
                    if let Some(handler) = &on_close {
                        handler.call(event);
                    }
                },
            }
            {children}
        }
        // Setup script
    }
}

// ---------------------------------------------------------------------------
// DialogClose / DialogAction
// ---------------------------------------------------------------------------

#[component]
pub fn DialogClose(#[props(into, optional)] class: Option<String>, children: Element) -> Element {
    let ctx = use_context::<DialogContext>();
    let merged = tw_merge!(
        "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium border bg-background shadow-xs hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2 cursor-pointer",
        class.as_deref().unwrap_or("")
    );
    rsx! {
        button {
            class: "{merged}",
            "data-dialog-close": "{ctx.target_id}",
            aria_label: "Close dialog",
            {children}
        }
    }
}

#[component]
pub fn DialogAction(#[props(into, optional)] class: Option<String>, children: Element) -> Element {
    let ctx = use_context::<DialogContext>();
    let merged = tw_merge!(
        "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium bg-primary text-primary-foreground shadow-xs hover:bg-primary/90 h-9 px-4 py-2 cursor-pointer",
        class.as_deref().unwrap_or("")
    );
    rsx! {
        button {
            class: "{merged}",
            "data-dialog-close": "{ctx.target_id}",
            aria_label: "Close dialog",
            {children}
        }
    }
}

// ---------------------------------------------------------------------------
// Layout helpers
// ---------------------------------------------------------------------------

#[component]
pub fn DialogHeader(#[props(into, optional)] class: Option<String>, children: Element) -> Element {
    let merged = tw_merge!(
        "flex flex-col gap-2 text-center sm:text-left",
        class.as_deref().unwrap_or("")
    );
    rsx! {
        div { class: "{merged}", {children} }
    }
}

#[component]
pub fn DialogTitle(#[props(into, optional)] class: Option<String>, children: Element) -> Element {
    let ctx = use_context::<DialogContext>();
    let merged = tw_merge!(
        "text-lg leading-none font-semibold",
        class.as_deref().unwrap_or("")
    );
    rsx! {
        h3 { id: format!("{}-title", ctx.target_id), class: "{merged}", {children} }
    }
}

#[component]
pub fn DialogDescription(
    #[props(into, optional)] class: Option<String>,
    children: Element,
) -> Element {
    let merged = tw_merge!(
        "text-muted-foreground text-sm",
        class.as_deref().unwrap_or("")
    );
    rsx! {
        p { class: "{merged}", {children} }
    }
}

#[component]
pub fn DialogBody(#[props(into, optional)] class: Option<String>, children: Element) -> Element {
    let merged = tw_merge!("flex flex-col gap-4", class.as_deref().unwrap_or(""));
    rsx! {
        div { class: "{merged}", {children} }
    }
}

#[component]
pub fn DialogFooter(#[props(into, optional)] class: Option<String>, children: Element) -> Element {
    let merged = tw_merge!(
        "flex flex-col-reverse gap-2 sm:flex-row sm:justify-end",
        class.as_deref().unwrap_or("")
    );
    rsx! {
        footer { class: "{merged}", {children} }
    }
}
