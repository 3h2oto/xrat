//! Engine-neutral traffic stats sampling for the TUI stats tab. A
//! [`StatsSource`] yields cumulative uplink/downlink byte counters that the
//! poller turns into totals and throughput. Sampling is best-effort: callers
//! treat any [`StatsError`] as "no sample this tick" and keep the previous view.

mod singbox;
mod xray;

pub use singbox::SingboxStatsSource;
pub use xray::XrayStatsSource;

mod parser;
pub use parser::StatsError;
pub use parser::StatsSample;
pub use parser::StatsSource;
