#[async_trait::async_trait]
pub trait ReleaseProvider: Send + Sync {
    async fn latest_tag(&self, timeout_secs: u64) -> crate::app::Result<String>;
}
