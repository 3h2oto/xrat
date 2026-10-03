mod layout;

#[cfg(test)]
mod tests;

pub use layout::{AppPaths, ensure_config_file, ensure_layout, resolve, resolve_with_env};
