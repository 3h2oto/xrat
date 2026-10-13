mod diagnostic;
mod entry;
mod prelude;
mod semantics;
mod toml_structure;
mod tui;

#[cfg(test)]
mod tests;

pub use entry::run;
pub(crate) use semantics::validate_app_config;
