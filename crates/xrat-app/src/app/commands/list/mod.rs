mod configs;
mod entry;
mod prelude;
mod subscriptions;

#[cfg(test)]
mod tests;

#[cfg(any(windows, test))]
pub(super) use configs::format_configs;
pub(crate) use configs::subscription_json;
pub use entry::run;
#[cfg(any(windows, test))]
pub(super) use subscriptions::format_subscriptions;
