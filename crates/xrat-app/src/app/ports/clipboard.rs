#[derive(Debug, thiserror::Error)]
pub enum ClipboardError {
    #[error("clipboard unavailable: {0}")]
    Unavailable(String),
    #[error("clipboard write failed: {0}")]
    Write(String),
}

pub trait Clipboard {
    fn copy_text(&self, text: &str) -> Result<(), ClipboardError>;
}
