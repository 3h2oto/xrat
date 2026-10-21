use dioxus::prelude::*;
use wasm_bindgen::JsCast;

use crate::ui::{
    button::{Button, ButtonVariant},
    icon_button::IconButton,
};

#[derive(Clone, Copy)]
pub struct CopyFeedback {
    message: Signal<Option<&'static str>>,
    attempt: Signal<u64>,
}

pub fn use_copy_feedback() -> CopyFeedback {
    CopyFeedback {
        message: use_signal(|| None),
        attempt: use_signal(|| 0),
    }
}

impl CopyFeedback {
    pub fn label(self, original: &str) -> String {
        self.message
            .read()
            .map_or_else(|| original.to_string(), str::to_string)
    }

    pub fn copy_target(mut self, target_id: &str, event: MouseEvent) {
        let target = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id(target_id));
        let Some(target) = target else { return };
        let button = event
            .data()
            .downcast::<web_sys::MouseEvent>()
            .and_then(|event| event.target())
            .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
            .and_then(|target| target.closest("button").ok().flatten())
            .and_then(|target| target.dyn_into::<web_sys::HtmlElement>().ok());
        let attempt = *self.attempt.peek() + 1;
        self.attempt.set(attempt);
        spawn(async move {
            let result = copy_element(&target, button.as_ref()).await;
            if *self.attempt.peek() != attempt {
                return;
            }
            self.message.set(Some(if result.is_ok() {
                "Copied"
            } else {
                "Copy manually"
            }));
            gloo_timers::future::TimeoutFuture::new(2500).await;
            if *self.attempt.peek() == attempt {
                self.message.set(None);
            }
        });
    }
}

#[component]
pub fn CopyButton(
    target_id: String,
    label: String,
    #[props(into, optional)] class: Option<String>,
    #[props(default)] variant: ButtonVariant,
    #[props(default)] icon_only: bool,
    #[props(default)] disabled: bool,
) -> Element {
    let feedback = use_copy_feedback();
    let label = feedback.label(&label);
    rsx! {
        if icon_only {
            IconButton {
                class,
                label,
                variant,
                disabled,
                onclick: move |event| feedback.copy_target(&target_id, event),
                icons::Copy {}
            }
        } else {
            Button {
                class,
                variant,
                disabled,
                onclick: move |event| feedback.copy_target(&target_id, event),
                icons::Copy {}
                span { "{label}" }
            }
        }
    }
}

async fn copy_element(
    target: &web_sys::Element,
    button: Option<&web_sys::HtmlElement>,
) -> Result<(), String> {
    let text = target
        .dyn_ref::<web_sys::HtmlTextAreaElement>()
        .map(|textarea| textarea.value())
        .unwrap_or_else(|| target.text_content().unwrap_or_default());
    let window = web_sys::window().ok_or("Clipboard unavailable")?;
    if clipboard_available(&window) {
        return copy(&text).await;
    }
    let document = window.document().ok_or("Clipboard unavailable")?;
    if let Some(element) = target.dyn_ref::<web_sys::HtmlElement>() {
        let _ = element.focus();
    }
    let selection = window.get_selection().ok().flatten();
    if let Some(textarea) = target.dyn_ref::<web_sys::HtmlTextAreaElement>() {
        textarea.select();
    } else {
        let range = document.create_range().map_err(|_| "Copy failed")?;
        range
            .select_node_contents(target)
            .map_err(|_| "Copy failed")?;
        if let Some(selection) = &selection {
            let _ = selection.remove_all_ranges();
            let _ = selection.add_range(&range);
        }
    }
    let copied = document
        .dyn_ref::<web_sys::HtmlDocument>()
        .is_some_and(|document| document.exec_command("copy").unwrap_or(false));
    if !copied {
        return Err("Copy failed".into());
    }
    if let Some(selection) = selection {
        let _ = selection.remove_all_ranges();
    }
    if let Some(button) = button {
        let options = web_sys::FocusOptions::new();
        options.set_prevent_scroll(true);
        let _ = button.focus_with_options(&options);
    }
    Ok(())
}

fn clipboard_available(window: &web_sys::Window) -> bool {
    window.is_secure_context()
        && js_sys::Reflect::get(
            window.navigator().as_ref(),
            &wasm_bindgen::JsValue::from_str("clipboard"),
        )
        .is_ok_and(|value| !value.is_undefined() && !value.is_null())
}

pub async fn copy(text: &str) -> Result<(), String> {
    let window = web_sys::window().ok_or("Clipboard unavailable")?;
    let navigator = window.navigator();
    if !clipboard_available(&window) {
        return Err("Clipboard unavailable. Copy the text below.".into());
    }
    wasm_bindgen_futures::JsFuture::from(navigator.clipboard().write_text(text))
        .await
        .map_err(|_| "Clipboard unavailable. Copy the text below.".to_string())?;
    Ok(())
}
