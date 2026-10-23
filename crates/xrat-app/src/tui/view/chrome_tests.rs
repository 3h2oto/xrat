use ratatui::{Terminal, backend::TestBackend};

use super::render_key_bar;
use crate::tui::app::{BulkOp, ConfirmKind, ConfirmState, TuiApp};

fn render(app: &TuiApp, width: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, 1)).unwrap();
    terminal
        .draw(|frame| render_key_bar(frame, frame.area(), app))
        .unwrap();
    let buffer = terminal.backend().buffer();
    (0..width)
        .map(|column| buffer[(column, 0)].symbol())
        .collect()
}

#[test]
fn confirmations_remain_visible_with_idle_or_long_connection_labels() {
    for connected in [false, true] {
        let mut app = TuiApp::default();
        app.data.runtime.pid_running = connected;
        app.data.runtime.status = if connected { "running" } else { "stopped" }.into();
        app.data.runtime.active_config = Some("a very long active configuration name".into());
        app.confirm = Some(ConfirmState {
            kind: ConfirmKind::ClearEvents,
            prompt: "clear all persisted events from db (cannot undo)?".into(),
        });
        for width in [80, 120] {
            let text = render(&app, width);
            assert!(
                text.contains("clear all persisted events from db (cannot undo)?"),
                "{text}"
            );
            assert!(text.contains("y / n"), "{text}");
        }
    }
}

#[test]
fn bulk_confirmations_take_priority_over_connection_details() {
    let mut app = TuiApp::default();
    app.data.runtime.pid_running = true;
    app.data.runtime.status = "running".into();
    app.data.runtime.active_config = Some("a very long active configuration name".into());
    app.pending_bulk = Some(BulkOp::PurgeAllDeleted);
    let text = render(&app, 80);
    assert!(text.contains("purge 0"), "{text}");
    assert!(text.contains("y / n"), "{text}");
}

#[test]
fn long_unicode_confirmation_preserves_choices_in_narrow_rows() {
    let app = TuiApp {
        confirm: Some(ConfirmState {
            kind: ConfirmKind::ClearEvents,
            prompt: format!("purge {} (cannot undo)?", "界".repeat(60)),
        }),
        ..TuiApp::default()
    };
    let text = render(&app, 32);
    assert!(text.starts_with("purge "), "{text}");
    assert!(text.contains("…"), "{text}");
    assert!(text.contains("y / n"), "{text}");
}

#[test]
fn operation_errors_remain_visible_instead_of_optional_hints() {
    let mut app = TuiApp::default();
    app.set_chrome_message(
        "failed to copy subscription: clipboard unavailable".into(),
        true,
    );
    let text = render(&app, 80);
    assert!(
        text.contains("failed to copy subscription: clipboard unavailable"),
        "{text}"
    );
    assert!(!text.contains("Keybindings:"), "{text}");
}
