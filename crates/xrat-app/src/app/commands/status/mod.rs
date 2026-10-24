use crate::app::context::AppContext;
use crate::app::daemon::ipc;
use crate::cli::StatusArgs;

mod display;
#[cfg(any(windows, test))]
pub(super) use display::print_payload;

#[cfg(all(test, unix))]
mod tests;

mod handler;
pub use handler::run;
