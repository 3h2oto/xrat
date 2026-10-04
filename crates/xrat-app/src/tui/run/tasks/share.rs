use crate::app::context::AppContext;
use crate::app::ports::{Clipboard, ClipboardError};
use crate::app::services::clipboard::ArboardClipboard;
use crate::tui::app::{QrKind, QrModalState, TuiApp};

pub async fn open_qr_for_config(
    context: &AppContext,
    app: &mut TuiApp,
    config_id: xrat_model::ConfigId,
    config_name: String,
) {
    match context.db.get_config_by_id(config_id).await {
        Ok(Some(record)) => {
            app.qr_modal = Some(QrModalState::new(
                QrKind::Config,
                config_name,
                record.raw_config,
            ));
            app.needs_full_clear = true;
        }
        Ok(None) => app.push_log(format!("ERR config {config_id} not found")),
        Err(err) => app.push_log(format!("ERR failed to load config URI: {err}")),
    }
}

pub async fn copy_config_uri(
    context: &AppContext,
    app: &mut TuiApp,
    config_id: xrat_model::ConfigId,
) {
    match context.db.get_config_by_id(config_id).await {
        Ok(Some(record)) => {
            let config_ref = record.r#ref;
            set_clipboard(
                app,
                record.raw_config,
                format!(
                    "copied config#{} to clipboard",
                    xrat_support::refs::short_ref(&config_ref)
                ),
            );
        }
        Ok(None) => app.push_log(format!("ERR config {config_id} not found")),
        Err(err) => app.push_log(format!("ERR failed to load config URI: {err}")),
    }
}

pub fn open_qr_for_source(app: &mut TuiApp, source_name: String, source_url: String) {
    app.qr_modal = Some(QrModalState::new(QrKind::Source, source_name, source_url));
    app.needs_full_clear = true;
}

pub fn copy_source_uri(app: &mut TuiApp, source_name: String, source_url: String) {
    set_clipboard(
        app,
        source_url,
        format!("copied subscription \"{source_name}\" to clipboard"),
    );
}

pub fn open_qr_for_api_url(app: &mut TuiApp) {
    let url = app.data.api_b64_url.clone();
    if url.is_empty() {
        app.push_log("ERR API subscription URL not available");
        return;
    }
    app.qr_modal = Some(QrModalState::new(QrKind::Api, "API /b64 subscription", url));
    app.needs_full_clear = true;
}

pub fn copy_api_url(app: &mut TuiApp) {
    let url = app.data.api_b64_url.clone();
    if url.is_empty() {
        app.push_log("ERR API subscription URL not available");
        return;
    }
    set_clipboard(
        app,
        url,
        "copied API subscription URL to clipboard".to_string(),
    );
}

fn set_clipboard(app: &mut TuiApp, text: String, success_message: String) {
    set_clipboard_with(app, text, success_message, &ArboardClipboard);
}

fn set_clipboard_with(
    app: &mut TuiApp,
    text: String,
    success_message: String,
    clipboard: &dyn Clipboard,
) {
    match clipboard.copy_text(&text) {
        Ok(()) => handle_clipboard_write_result(app, text, success_message, Ok(())),
        Err(ClipboardError::Write(error)) => {
            handle_clipboard_write_result(app, text, success_message, Err(error))
        }
        Err(error) => {
            app.push_log(format!("ERR {error}"));
            app.set_chrome_message(error.to_string(), true);
        }
    }
}

fn handle_clipboard_write_result(
    app: &mut TuiApp,
    text: String,
    success_message: String,
    result: Result<(), String>,
) {
    match result {
        Ok(()) => {
            let preview = if text.chars().count() > 40 {
                format!("{}…", text.chars().take(40).collect::<String>())
            } else {
                text
            };
            app.push_log(format!("OK  copied: {preview}"));
            app.set_chrome_message(success_message, false);
        }
        Err(error) => {
            let error = format!("clipboard write failed: {error}");
            app.push_log(format!("ERR {error}"));
            app.set_chrome_message(error, true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::handle_clipboard_write_result;
    use crate::tui::app::TuiApp;

    #[test]
    fn clipboard_success_sets_ok_log_and_chrome_message() {
        let mut app = TuiApp::default();
        handle_clipboard_write_result(
            &mut app,
            "vless://example".to_string(),
            "copied config#abc123 to clipboard".to_string(),
            Ok(()),
        );
        assert_eq!(
            app.event_log.last().map(String::as_str),
            Some("OK  copied: vless://example")
        );
        assert_eq!(
            app.chrome_message
                .as_ref()
                .map(|message| message.text.as_str()),
            Some("copied config#abc123 to clipboard")
        );
        assert_eq!(
            app.chrome_message.as_ref().map(|message| message.is_error),
            Some(false)
        );
    }

    #[test]
    fn clipboard_failure_sets_error_log_and_chrome_message() {
        let mut app = TuiApp::default();
        handle_clipboard_write_result(
            &mut app,
            "vless://example".to_string(),
            "copied config#abc123 to clipboard".to_string(),
            Err("backend unavailable".to_string()),
        );
        assert_eq!(
            app.event_log.last().map(String::as_str),
            Some("ERR clipboard write failed: backend unavailable")
        );
        assert_eq!(
            app.chrome_message
                .as_ref()
                .map(|message| message.text.as_str()),
            Some("clipboard write failed: backend unavailable")
        );
        assert_eq!(
            app.chrome_message.as_ref().map(|message| message.is_error),
            Some(true)
        );
    }
}

#[cfg(test)]
mod clipboard_port_tests {
    use super::*;
    use std::cell::RefCell;
    struct FakeClipboard {
        text: RefCell<String>,
        unavailable: bool,
    }
    impl Clipboard for FakeClipboard {
        fn copy_text(&self, text: &str) -> Result<(), ClipboardError> {
            if self.unavailable {
                return Err(ClipboardError::Unavailable("headless".into()));
            }
            *self.text.borrow_mut() = text.into();
            Ok(())
        }
    }
    #[test]
    fn copies_unicode_with_fake_backend_and_safe_preview() {
        let clipboard = FakeClipboard {
            text: RefCell::new(String::new()),
            unavailable: false,
        };
        let text = "节点".repeat(30);
        let mut app = TuiApp::default();
        set_clipboard_with(&mut app, text.clone(), "copied".into(), &clipboard);
        assert_eq!(*clipboard.text.borrow(), text);
        assert_eq!(
            app.event_log.last().unwrap(),
            &format!("OK  copied: {}…", "节点".repeat(20))
        );
    }
    #[test]
    fn unavailable_backend_preserves_error_feedback_without_copying() {
        let clipboard = FakeClipboard {
            text: RefCell::new(String::new()),
            unavailable: true,
        };
        let mut app = TuiApp::default();
        set_clipboard_with(&mut app, "secret".into(), "copied".into(), &clipboard);
        assert!(clipboard.text.borrow().is_empty());
        assert_eq!(
            app.event_log.last().unwrap(),
            "ERR clipboard unavailable: headless"
        );
    }
}
