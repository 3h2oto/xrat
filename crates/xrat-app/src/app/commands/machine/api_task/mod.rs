mod definition;
#[cfg(windows)]
mod runtime;
#[cfg(windows)]
pub(super) use runtime::{install, uninstall};
#[cfg(test)]
mod tests;
