use crate::Notice;
use crate::api::{self, actions::run};
use crate::components::{clipboard, qr_code::QrCodeImage, states::ErrorState};
use crate::ui::{
    button::ButtonVariant, dialog::*, icon_button::IconButton, spinner::Spinner, textarea::Textarea,
};
use dioxus::prelude::*;
use icons::{Copy, ListChecks, Pencil, QrCode, RefreshCw, Trash2};
use xrat_web::types::{Action, SubscriptionRow, SubscriptionSource, SubscriptionSourcePreview};

#[derive(Clone, Copy, PartialEq)]
enum SourceMode {
    Copy,
    Qr,
}

#[component]
pub(super) fn SubscriptionActions(
    row: SubscriptionRow,
    on_rename: EventHandler<SubscriptionRow>,
    mut deleting: Signal<Option<i64>>,
) -> Element {
    let mut mode = use_signal(|| None::<SourceMode>);
    let id = row.id;
    let is_url = row.source == "url";
    let revision = row.updated_at.clone();
    let preview = use_resource(use_reactive!(|(revision,)| async move {
        let _ = revision;
        if is_url {
            api::get::<SubscriptionSourcePreview>(&format!("subscriptions/{id}/source/preview"))
                .await
        } else {
            Ok(SubscriptionSourcePreview {
                qr_supported: false,
            })
        }
    }));
    let qr_supported = preview
        .read()
        .as_ref()
        .is_some_and(|result| result.as_ref().is_ok_and(|value| value.qr_supported));
    rsx! {
        IconButton { label: format!("View configurations for {}", row.name), variant: ButtonVariant::Ghost, href: format!("/ui/configs?subscription={}", row.reference), ListChecks {} }
        IconButton { label: format!("Refresh subscription {}", row.name), variant: ButtonVariant::Ghost, disabled: !is_url, onclick: move |_| run(Action::Refresh { id: Some(id) }), RefreshCw {} }
        IconButton { label: if is_url { format!("Copy source URL for {}", row.name) } else { "This subscription has no source URL".into() }, variant: ButtonVariant::Ghost, disabled: !is_url, onclick: move |_| mode.set(Some(SourceMode::Copy)), Copy {} }
        IconButton { label: if qr_supported { format!("Show source URL QR for {}", row.name) } else if preview.pending() { "Checking QR capacity".into() } else { "QR unavailable: no source URL, exceeds capacity, or check failed".into() }, variant: ButtonVariant::Ghost, disabled: !qr_supported, onclick: move |_| mode.set(Some(SourceMode::Qr)), QrCode {} }
        IconButton { label: format!("Rename subscription {}", row.name), variant: ButtonVariant::Ghost, onclick: move |_| on_rename.call(row.clone()), Pencil {} }
        IconButton { label: "Delete subscription", variant: ButtonVariant::Ghost, onclick: move |_| deleting.set(Some(id)), Trash2 {} }
        if let Some(current_mode) = mode() {
            SourceModal { id, mode: current_mode, on_close: move |_| mode.set(None) }
        }
    }
}

#[component]
fn SourceModal(id: i64, mode: SourceMode, on_close: EventHandler) -> Element {
    let Notice(mut notice) = use_context();
    let mut needs_manual_copy = use_signal(|| false);
    let text_id = use_hook(|| crate::hooks::use_random::use_random_id_for("subscription-source"));
    let source = use_resource(move || async move {
        let source = api::get::<SubscriptionSource>(&format!("subscriptions/{id}/source")).await?;
        if mode == SourceMode::Copy {
            match clipboard::copy(&source.url).await {
                Ok(()) => {
                    notice.set(Some(crate::Feedback::success("Subscription URL copied.")));
                    on_close.call(());
                }
                Err(message) => {
                    notice.set(Some(crate::Feedback::error(message)));
                    needs_manual_copy.set(true);
                }
            }
        }
        Ok::<_, String>(source)
    });
    if mode == SourceMode::Copy
        && !needs_manual_copy()
        && !source.read().as_ref().is_some_and(Result::is_err)
    {
        return rsx! {};
    }
    rsx! {
        Dialog {
            DialogContent { initial_open: true, class: "w-[min(420px,calc(100%-2rem))] rounded-xl p-5", on_close: move |_| on_close.call(()),
                DialogHeader { class: "text-left pr-6 mb-4",
                    DialogTitle { if mode == SourceMode::Qr { "Subscription QR" } else { "Copy subscription URL" } }
                }
                match source.read().as_ref() {
                    Some(Ok(source)) => rsx! {
                        if mode == SourceMode::Qr {
                            QrCodeImage { payload: source.url.clone(), label: "QR code for the subscription source URL", filename: "xrat-subscription-source.svg" }
                        } else {
                            label { class: "sr-only", r#for: text_id.clone(), "Subscription source URL" }
                            Textarea { id: text_id.clone(), value: source.url.clone(), readonly: true, rows: 3, class: "font-mono text-xs break-all" }
                            clipboard::CopyButton { class: "mt-3", target_id: text_id.clone(), label: "Copy subscription URL" }
                        }
                    },
                    Some(Err(message)) => rsx! { ErrorState { message: message.clone() } },
                    None => rsx! { div { role: "status", class: "flex justify-center py-10", Spinner {} } },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn copy_view() -> Element {
        source_view(SourceMode::Copy)
    }

    fn qr_view() -> Element {
        source_view(SourceMode::Qr)
    }

    fn source_view(mode: SourceMode) -> Element {
        let notice = use_signal(|| None);
        use_context_provider(|| Notice(notice));
        rsx! { SourceModal { id: 1, mode, on_close: |_| {} } }
    }

    #[test]
    fn copy_waits_without_rendering_a_modal_while_qr_opens_immediately() {
        for (view, visible) in [(copy_view as fn() -> Element, false), (qr_view, true)] {
            let mut dom = VirtualDom::new(view);
            dom.provide_root_context(crate::ui::interaction::Interactions::default());
            let mutations = dom.rebuild_to_vec();
            assert_eq!(
                mutations
                    .edits
                    .iter()
                    .any(|edit| matches!(edit, dioxus::core::Mutation::LoadTemplate { .. })),
                visible
            );
        }
    }
}
