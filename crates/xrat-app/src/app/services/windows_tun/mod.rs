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
