mod detect;
mod error;
mod parsers;
#[allow(dead_code)]
mod subscription;

use xrat_model::Node;

pub use error::ImportParseError;

#[cfg(test)]
mod tests;

mod loader;
pub use loader::ImportMode;
pub use loader::ImportResult;
pub use loader::SubscriptionMetadata;
pub use loader::parse_import;
