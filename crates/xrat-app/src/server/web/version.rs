use crate::app::services::releases::ReleaseService;
use axum::Json;
use std::{
    sync::OnceLock,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use xrat_web::types::VersionInfo;

pub async fn get() -> Json<VersionInfo> {
    static CACHE: OnceLock<Mutex<Option<(Instant, VersionInfo)>>> = OnceLock::new();
    let mut cache = CACHE.get_or_init(|| Mutex::new(None)).lock().await;
    if let Some((checked, info)) = cache.as_ref()
        && checked.elapsed() < Duration::from_secs(900)
    {
        return Json(info.clone());
    }
    let version = env!("CARGO_PKG_VERSION").to_string();
    let latest = ReleaseService::default().latest_tag(5).await.ok();
    let info = version_info(&version, latest.as_deref());
    *cache = Some((Instant::now(), info.clone()));
    Json(info)
}

fn version_info(current: &str, latest: Option<&str>) -> VersionInfo {
    let current_version = semver::Version::parse(current).ok();
    let latest_version =
        latest.and_then(|value| semver::Version::parse(value.trim().trim_start_matches('v')).ok());
    VersionInfo {
        version: current.into(),
        update_available: current_version
            .zip(latest_version.clone())
            .map(|(current, latest)| latest > current),
        latest_version: latest_version.map(|version| version.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compares_versions_without_reporting_failed_checks_as_current() {
        assert_eq!(
            version_info("0.30.1", Some("v0.30.2")).update_available,
            Some(true)
        );
        assert_eq!(
            version_info("0.30.2", Some("v0.30.2")).update_available,
            Some(false)
        );
        assert_eq!(
            version_info("0.31.0", Some("v0.30.2")).update_available,
            Some(false)
        );
        assert_eq!(version_info("0.30.2", None).update_available, None);
        assert_eq!(
            version_info("0.30.2", Some("invalid")).update_available,
            None
        );
    }
}
