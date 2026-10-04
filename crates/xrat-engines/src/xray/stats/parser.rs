/// A single cumulative traffic reading. `uplink_total` and `downlink_total` are
/// monotonically increasing byte counters for the lifetime of the engine
/// session; throughput is derived by differencing successive samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatsSample {
    pub uplink_total: u64,
    pub downlink_total: u64,
}

#[derive(Debug, thiserror::Error)]
#[error("stats sampling failed: {0}")]
pub struct StatsError(pub String);

#[allow(
    clippy::double_must_use,
    reason = "async_trait adds must_use to methods returning already must-use boxed futures"
)]
#[async_trait::async_trait]
pub trait StatsSource: Send + Sync {
    async fn sample(&self) -> Result<StatsSample, StatsError>;
}
