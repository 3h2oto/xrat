use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineInfo {
    pub name: &'static str,
    pub available: bool,
    pub version: Option<String>,
}

#[allow(
    clippy::double_must_use,
    reason = "async_trait adds must_use to methods returning already must-use boxed futures"
)]
#[async_trait::async_trait]
pub trait RuntimeEngineProbe: Send + Sync {
    async fn probe(&self, name: &'static str, path: &Path) -> crate::app::Result<EngineInfo>;
}
