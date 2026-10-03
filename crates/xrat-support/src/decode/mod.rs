mod b64;
mod text;

#[cfg(test)]
mod tests;

pub use b64::b64_decode_text;
pub use text::{decode_or_json_text, decode_or_raw_text};

use thiserror::Error;

mod decoder;
pub use decoder::DecodeError;
