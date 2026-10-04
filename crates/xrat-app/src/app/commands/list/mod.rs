mod configs;
mod entry;
mod prelude;
mod subscriptions;

#[cfg(test)]
mod tests;

pub(crate) use configs::subscription_json;
pub use entry::run;
