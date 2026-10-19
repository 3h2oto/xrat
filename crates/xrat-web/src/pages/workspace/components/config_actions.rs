use super::data_tables::sharing::{Mode, ShareModal, use_qr_preview};
use super::*;
use crate::ui::dropdown_menu::*;
use icons::{Copy, EllipsisVertical, FlaskConical, QrCode, Trash2};
use xrat_web::configs::sharing::ConfigShareRequest;

#[component]
pub(crate) fn ConfigActions(
    id: i64,
    deleted: bool,
    #[props(default)] revision: String,
    mut pending_delete: Signal<Option<i64>>,
) -> Element {
    let mut sharing = use_signal(|| None::<Mode>);
    let preview = use_qr_preview(ConfigShareRequest::Selected { ids: vec![id] }, revision);
    let qr_supported = preview
        .read()
        .as_ref()
        .is_some_and(|result| result.as_ref().is_ok_and(|preview| preview.qr_supported));
    let qr_label = if preview.pending() {
        "Checking QR capacity".into()
    } else if qr_supported {
        format!("Show QR for configuration #{id}")
    } else {
        "QR unavailable: config exceeds QR capacity or could not be checked".into()
    };
    rsx! {
        div { class: "flex items-center gap-1.5",
            if deleted {
                IconButton { label: "Restore", variant: ButtonVariant::Outline, onclick: move |_| run(Action::Restore { id }), RotateCcw {} }
            } else {
                IconButton { label: "Connect", onclick: move |_| run(Action::Connect { id }), Play {} }
            }
            div { class: "flex items-center gap-1.5 mobile:hidden",
                IconButton { label: "Copy configuration #{id}", variant: ButtonVariant::Ghost, onclick: move |_| sharing.set(Some(Mode::Copy)), Copy {} }
                IconButton { label: qr_label, variant: ButtonVariant::Ghost, disabled: !qr_supported, onclick: move |_| sharing.set(Some(Mode::Qr)), QrCode {} }
                if !deleted {
                    IconButton { label: "Test configuration #{id}", variant: ButtonVariant::Ghost, onclick: move |_| run(Action::Test { id: Some(id) }), FlaskConical {} }
                    IconButton { label: "Delete configuration #{id}", variant: ButtonVariant::Ghost, onclick: move |_| pending_delete.set(Some(id)), Trash2 {} }
                }
            }
            div { class: "hidden mobile:block",
                DropdownMenu {
                    DropdownMenuTrigger { as_child: true,
                        IconButton { label: "Actions for configuration #{id}", variant: ButtonVariant::Ghost, EllipsisVertical {} }
                    }
                    DropdownMenuContent {
                        DropdownMenuLabel { "Configuration #{id}" }
                        DropdownMenuAction { onclick: move |_| sharing.set(Some(Mode::Copy)), Copy {} "Copy config" }
                        DropdownMenuAction { disabled: !qr_supported, onclick: move |_| sharing.set(Some(Mode::Qr)), QrCode {} "Show QR" }
                        if !deleted {
                            DropdownMenuAction { onclick: move |_| run(Action::Test { id: Some(id) }), FlaskConical {} "Test connection" }
                            DropdownMenuSeparator {}
                            DropdownMenuAction { variant: DropdownMenuActionVariant::Destructive, onclick: move |_| pending_delete.set(Some(id)), Trash2 {} "Delete" }
                        }
                    }
                }
            }
        }
        if let Some(mode) = sharing() {
            ShareModal { request: ConfigShareRequest::Selected { ids: vec![id] }, count: 1, mode, on_close: move |_| sharing.set(None) }
        }
    }
}
