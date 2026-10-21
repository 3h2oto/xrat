mod catalog;
#[cfg(windows)]
mod entry;
#[cfg(windows)]
mod installation;

#[cfg(windows)]
pub use entry::run;
#[cfg(windows)]
pub use installation::{install, uninstall};
