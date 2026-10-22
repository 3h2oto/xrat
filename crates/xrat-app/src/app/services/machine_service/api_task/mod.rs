mod definition;
#[cfg(windows)]
mod runtime;
#[cfg(windows)]
pub(crate) use runtime::{install, uninstall};
#[cfg(test)]
mod tests;

#[cfg(windows)]
pub(crate) use runtime::{inspect, pause, resume};
