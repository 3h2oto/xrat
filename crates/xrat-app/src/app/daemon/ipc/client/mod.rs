#[cfg(any(unix, windows))]
mod requests;
#[cfg(not(any(unix, windows)))]
mod unsupported_impl;
#[cfg(any(unix, windows))]
mod wire;

#[cfg(any(unix, windows))]
pub use requests::*;
#[cfg(not(any(unix, windows)))]
pub use unsupported_impl::*;
