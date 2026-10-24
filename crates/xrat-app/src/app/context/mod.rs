mod app_context;
#[cfg(any(windows, test))]
mod machine_client;
mod paths;

#[cfg(test)]
mod tests;

pub use app_context::{AppContext, build_app_context};
pub use paths::{RuntimePaths, resolve_config_path};

#[cfg(windows)]
pub(crate) use machine_client::require_ordinary_user;
