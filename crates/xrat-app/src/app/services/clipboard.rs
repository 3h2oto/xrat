use crate::app::ports::{Clipboard, ClipboardError};
use std::cell::RefCell;

thread_local! {
    static CLIPBOARD: RefCell<Option<arboard::Clipboard>> = const { RefCell::new(None) };
}

pub struct ArboardClipboard;

impl Clipboard for ArboardClipboard {
    fn copy_text(&self, text: &str) -> Result<(), ClipboardError> {
        let arboard_result = CLIPBOARD.with(|clipboard| {
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
        });

        let osc_success = emit_osc52(text);

        match arboard_result {
            Ok(()) => Ok(()),
            Err(_) if osc_success => Ok(()),
            Err(err) => Err(err),
        }
    }
}

pub(crate) fn format_osc52(text: &str, in_tmux: bool) -> String {
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
    if in_tmux {
        format!("\x1bPtmux;\x1b\x1b]52;c;{encoded}\x07\x1b\\")
    } else {
        format!("\x1b]52;c;{encoded}\x07")
    }
}

pub(crate) fn write_osc52<W: std::io::Write>(
    writer: &mut W,
    text: &str,
    in_tmux: bool,
) -> std::io::Result<()> {
    let sequence = format_osc52(text, in_tmux);
    writer.write_all(sequence.as_bytes())?;
    writer.flush()
}

fn emit_osc52(text: &str) -> bool {
    use std::io::IsTerminal;

    if !std::io::stdout().is_terminal() {
        return false;
    }

    let in_tmux = std::env::var_os("TMUX").is_some();
    let mut stdout = std::io::stdout();
    write_osc52(&mut stdout, text, in_tmux).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_standard_osc52_sequence() {
        let seq = format_osc52("hello", false);
        assert_eq!(seq, "\x1b]52;c;aGVsbG8=\x07");
    }

    #[test]
    fn formats_tmux_osc52_sequence() {
        let seq = format_osc52("hello", true);
        assert_eq!(seq, "\x1bPtmux;\x1b\x1b]52;c;aGVsbG8=\x07\x1b\\");
    }

    #[test]
    fn writes_osc52_to_writer() {
        let mut buffer = Vec::new();
        write_osc52(&mut buffer, "hello", false).unwrap();
        assert_eq!(buffer, b"\x1b]52;c;aGVsbG8=\x07");

        let mut tmux_buffer = Vec::new();
        write_osc52(&mut tmux_buffer, "hello", true).unwrap();
        assert_eq!(tmux_buffer, b"\x1bPtmux;\x1b\x1b]52;c;aGVsbG8=\x07\x1b\\");
    }

    #[test]
    fn arboard_copy_tolerates_headless_environment() {
        let res = ArboardClipboard.copy_text("HELLO_XRAT_TEST_123");
        match res {
            Ok(()) | Err(ClipboardError::Unavailable(_)) => {}
            Err(err) => panic!("unexpected clipboard error: {err}"),
        }
    }
}
