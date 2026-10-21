use crate::app::context::AppContext;
use crate::app::daemon::ipc;
use crate::cli::StatusArgs;

mod display;

#[cfg(all(test, unix))]
mod tests;

mod handler;
pub use handler::run;
