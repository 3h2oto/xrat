mod catalog;
#[cfg(windows)]
mod entry;
#[cfg(windows)]
mod installation;

#[cfg(windows)]
pub use entry::run;
#[cfg(windows)]
pub use installation::{install, uninstall};

#[cfg(windows)]
mod cores;

#[cfg(windows)]
mod tun_setup;

#[cfg(windows)]
use crate::app::services::machine_service::api_task;
