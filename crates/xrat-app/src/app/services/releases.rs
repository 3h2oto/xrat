mod github;
pub use github::{GithubReleaseProvider, HttpReleaseProvider};

use crate::app::ports::ReleaseProvider;
use std::sync::Arc;

pub const REPO: &str = "mhyrzt/xrat";

pub struct ReleaseService {
    provider: Arc<dyn ReleaseProvider>,
}

impl Default for ReleaseService {
    fn default() -> Self {
        Self::with_provider(Arc::new(GithubReleaseProvider))
    }
}

impl ReleaseService {
    pub fn with_provider(provider: Arc<dyn ReleaseProvider>) -> Self {
        Self { provider }
    }

    pub async fn latest_tag(&self, timeout_secs: u64) -> crate::app::Result<String> {
        self.provider.latest_tag(timeout_secs).await
    }

    pub async fn newer_tag(
        &self,
        current: &str,
        timeout_secs: u64,
    ) -> crate::app::Result<Option<String>> {
        let tag = self.latest_tag(timeout_secs).await?;
        Ok(is_newer(&tag, current).then_some(tag))
    }
}

fn is_newer(latest: &str, current: &str) -> bool {
    match (parse_semver(latest), parse_semver(current)) {
        (Some(latest), Some(current)) => latest > current,
        _ => normalize(latest) != normalize(current),
    }
}

fn normalize(tag: &str) -> &str {
    tag.trim().trim_start_matches('v')
}

fn parse_semver(tag: &str) -> Option<(u32, u32, u32)> {
    let mut parts = normalize(tag).split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next().unwrap_or("0").parse().ok()?;
    Some((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use super::is_newer;

    #[test]
    fn detects_newer_release() {
        assert!(is_newer("v0.3.0", "0.2.1"));
        assert!(is_newer("0.2.2", "0.2.1"));
        assert!(is_newer("1.0.0", "0.9.9"));
    }

    #[test]
    fn ignores_same_or_older() {
        assert!(!is_newer("v0.2.1", "0.2.1"));
        assert!(!is_newer("0.2.0", "0.2.1"));
        assert!(!is_newer("v0.1.0", "0.2.1"));
    }
}

#[cfg(test)]
mod provider_tests {
    use super::*;
    struct FakeProvider {
        tag: Option<&'static str>,
    }
    #[async_trait::async_trait]
    impl ReleaseProvider for FakeProvider {
        async fn latest_tag(&self, timeout_secs: u64) -> crate::app::Result<String> {
            assert_eq!(timeout_secs, 8);
            self.tag
                .map(str::to_string)
                .ok_or_else(|| crate::app::AppError::InvalidArgument("offline".into()))
        }
    }
    #[tokio::test]
    async fn newer_release_checks_preserve_current_versions_and_propagate_provider_errors() {
        for (tag, expected) in [
            ("v0.21.0", Some("v0.21.0")),
            ("v0.20.0", None),
            ("v0.19.0", None),
        ] {
            let service = ReleaseService::with_provider(Arc::new(FakeProvider { tag: Some(tag) }));
            assert_eq!(service.latest_tag(8).await.unwrap(), tag);
            assert_eq!(
                service.newer_tag("0.20.0", 8).await.unwrap().as_deref(),
                expected
            );
        }
        let service = ReleaseService::with_provider(Arc::new(FakeProvider { tag: None }));
        assert!(matches!(
            service.newer_tag("0.20.0", 8).await,
            Err(crate::app::AppError::InvalidArgument(_))
        ));
    }
}
