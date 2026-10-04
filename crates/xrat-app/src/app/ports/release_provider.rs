#[allow(
    clippy::double_must_use,
    reason = "async_trait adds must_use to methods returning already must-use boxed futures"
)]
#[async_trait::async_trait]
pub trait ReleaseProvider: Send + Sync {
    async fn latest_tag(&self, timeout_secs: u64) -> crate::app::Result<String>;
}
