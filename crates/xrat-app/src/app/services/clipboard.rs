use crate::app::ports::{Clipboard, ClipboardError};
use std::cell::RefCell;

thread_local! {
    static CLIPBOARD: RefCell<Option<arboard::Clipboard>> = const { RefCell::new(None) };
}

pub struct ArboardClipboard;
impl Clipboard for ArboardClipboard {
    fn copy_text(&self, text: &str) -> Result<(), ClipboardError> {
        CLIPBOARD.with(|clipboard| {
            let mut clipboard = clipboard.borrow_mut();
            if clipboard.is_none() {
                *clipboard = Some(
                    arboard::Clipboard::new()
                        .map_err(|error| ClipboardError::Unavailable(error.to_string()))?,
                );
            }
            if let Some(clipboard) = clipboard.as_mut() {
                clipboard
                    .set_text(text)
                    .map_err(|error| ClipboardError::Write(error.to_string()))?;
            }
            Ok(())
        })
    }
}
