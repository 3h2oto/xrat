mod layout;
#[cfg(any(windows, test))]
pub(crate) use layout::ensure_layout_at;

#[cfg(test)]
mod tests;

pub use layout::{AppPaths, ensure_config_file, ensure_layout, resolve, resolve_with_env};
