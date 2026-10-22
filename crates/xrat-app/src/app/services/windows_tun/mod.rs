#[cfg(any(windows, test))]
mod package;

#[cfg(windows)]
mod signature;
#[cfg(windows)]
pub(crate) use signature::verify_signature;

#[cfg(windows)]
pub(crate) use package::verify_installed;
#[cfg(windows)]
pub(crate) use package::{ARCHIVE_LIMIT, ARCHIVE_URL, prepare};

#[cfg(test)]
mod tests;

#[cfg(all(windows, test))]
mod native_adapter;
#[cfg(all(windows, test))]
pub(crate) use native_adapter::verify_native_adapter;

pub(crate) mod policy;
#[cfg(windows)]
mod runtime;
#[cfg(windows)]
pub(crate) use runtime::{check_engine, preflight, ready, verify_release};

#[cfg(any(windows, test))]
mod xray;
#[cfg(windows)]
pub(crate) use xray::configure_xray_dns;
