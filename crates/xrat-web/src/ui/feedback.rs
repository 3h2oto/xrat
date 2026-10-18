use super::{
    alert::{Alert, AlertDescription, AlertTitle, AlertVariant},
    button::{Button, ButtonSize, ButtonVariant},
    toast::{ToastContainer, ToastList},
};
use crate::Notice;
use dioxus::prelude::*;
use icons::X;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Copy, PartialEq)]
pub enum FeedbackKind {
    Info,
    Success,
    Error,
}
#[derive(Clone, PartialEq)]
pub struct Feedback {
    id: usize,
    pub kind: FeedbackKind,
    pub message: String,
}
static NEXT: AtomicUsize = AtomicUsize::new(1);
impl Feedback {
    fn new(kind: FeedbackKind, message: impl Into<String>) -> Self {
        Self {
            id: NEXT.fetch_add(1, Ordering::Relaxed),
            kind,
            message: message.into(),
        }
    }
    pub fn job_started(kind: &str) -> Self {
        Self::info(format!(
            "{} started. Follow progress in Jobs.",
            super::job_labels::job_label(kind)
        ))
    }
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(FeedbackKind::Info, message)
    }
    pub fn success(message: impl Into<String>) -> Self {
        Self::new(FeedbackKind::Success, message)
    }
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(FeedbackKind::Error, message)
    }
}
#[component]
pub fn FeedbackView() -> Element {
    let Notice(mut notice) = use_context();
    let mut shown = use_signal(|| None::<Feedback>);
    let mut closing = use_signal(|| false);
    use_effect(move || {
        if let Some(message) = notice() {
            shown.set(Some(message.clone()));
            closing.set(false);
            if message.kind != FeedbackKind::Error {
                spawn(async move {
                    gloo_timers::future::TimeoutFuture::new(6000).await;
                    if notice
                        .peek()
                        .as_ref()
                        .is_some_and(|current| current.id == message.id)
                    {
                        notice.set(None);
                    }
                });
            }
        } else if let Some(message) = shown.peek().clone() {
            closing.set(true);
            spawn(async move {
                gloo_timers::future::TimeoutFuture::new(140).await;
                if notice.peek().is_none()
                    && shown
                        .peek()
                        .as_ref()
                        .is_some_and(|current| current.id == message.id)
                {
                    shown.set(None);
                    closing.set(false);
                }
            });
        }
    });
    rsx! {
        if let Some(message) = shown() {
            ToastContainer { class: "right-4 bottom-4 max-w-[calc(100vw-2rem)] z-[150]",
                ToastList { class: "w-[min(400px,calc(100vw-2rem))] h-auto list-none",
                    li {
                        key: "{message.id}",
                        class: concat!(
                            "toast-notification [&&]:[animation:toast-enter_180ms_cubic-bezier(0.2,_0.8,_0.2,_1)_both] ",
                            "[&&[data-state=closing]]:[animation:toast-exit_140ms_ease-in_both] ",
                            "[&&[data-state=closing]]:pointer-events-none ",
                        ),
                        "data-state": if closing() { "closing" } else { "open" },
                        Alert {
                            class: "bg-card shadow-lg pr-12",
                            variant: if message.kind == FeedbackKind::Error { AlertVariant::Destructive } else { AlertVariant::Default },
                            AlertTitle {
                                match message.kind {
                                    FeedbackKind::Error => "Action failed",
                                    FeedbackKind::Success => "Done",
                                    FeedbackKind::Info => "In progress",
                                }
                            }
                            AlertDescription { "{message.message}" }
                            Button {
                                variant: ButtonVariant::Ghost,
                                size: ButtonSize::IconXs,
                                class: "absolute right-2 top-2",
                                aria_label: "Dismiss notification",
                                onclick: move |_: MouseEvent| notice.set(None),
                                X {}
                            }
                        }
                    }
                }
            }
        }
    }
}
