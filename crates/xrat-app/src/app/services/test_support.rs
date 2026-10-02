use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use crate::app::ports::{Clock, Filesystem};
use crate::app::read_models::{ConfigSummary, LatestTestSummary};
use xrat_db::record::{ConfigListFilter, ConfigRecord, ConfigWithLatestTest};

/// Clock stub whose time can be advanced or pinned in tests.
#[derive(Debug, Default)]
pub struct FakeClock {
    now: AtomicI64,
}

impl FakeClock {
    pub fn at(now: i64) -> Arc<Self> {
        Arc::new(Self {
            now: AtomicI64::new(now),
        })
    }

    pub fn advance(&self, seconds: i64) {
        self.now.fetch_add(seconds, Ordering::SeqCst);
    }
}

impl Clock for FakeClock {
    fn now_unix_secs(&self) -> i64 {
        self.now.load(Ordering::SeqCst)
    }
}

/// In-memory filesystem for tests that should not touch disk.
#[derive(Debug, Default)]
pub struct InMemoryFilesystem {
    files: std::sync::Mutex<std::collections::HashMap<std::path::PathBuf, String>>,
}

impl InMemoryFilesystem {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn contents(&self, path: &std::path::Path) -> Option<String> {
        self.files.lock().unwrap().get(path).cloned()
    }
}

impl Filesystem for InMemoryFilesystem {
    fn read_to_string(&self, path: &std::path::Path) -> std::io::Result<String> {
        self.files
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "not found"))
    }

    fn write_string(&self, path: &std::path::Path, contents: &str) -> std::io::Result<()> {
        self.files
            .lock()
            .unwrap()
            .insert(path.to_path_buf(), contents.to_string());
        Ok(())
    }

    fn exists(&self, path: &std::path::Path) -> bool {
        self.files.lock().unwrap().contains_key(path)
    }

    fn create_dir_all(&self, _path: &std::path::Path) -> std::io::Result<()> {
        Ok(())
    }
}

/// Build a joined config row for read-model tests.
pub fn sample_joined_row(id: i64, enabled: bool) -> ConfigWithLatestTest {
    ConfigWithLatestTest {
        config: ConfigRecord {
            id: id.into(),
            r#ref: format!("ref{id}"),
            subscription_id: None,
            dedup_key: format!("key{id}"),
            protocol: "vless".to_string(),
            address: "example.com".to_string(),
            port: 443,
            username: None,
            uuid: None,
            password: None,
            method: None,
            network: "tcp".to_string(),
            tls: Some("tls".to_string()),
            sni: None,
            host: None,
            path: None,
            name: Some(format!("node-{id}")),
            raw_config: String::new(),
            extensions_json: None,
            is_active: false,
            is_enabled: enabled,
            is_deleted: false,
            deleted_at: None,
            imported_at: "2026-01-01T00:00:00Z".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        },
        test_id: None,
        icmp_ok: None,
        icmp_ms: None,
        tcp_ok: None,
        tcp_ms: None,
        real_delay_ok: None,
        real_delay_ms: None,
        download_mbps: None,
        upload_mbps: None,
        connect_ms: None,
        ttfb_ms: None,
        http_status: None,
        dial_endpoint_location: None,
        dial_endpoint_country: None,
        dial_endpoint_asn: None,
        dial_endpoint_geoip_source: None,
        dial_endpoint_fronting: None,
        failure_kind: None,
        failure_reason: None,
        tested_at: None,
    }
}

#[test]
fn fake_clock_advances() {
    let clock = FakeClock::at(1000);
    assert_eq!(clock.now_unix_secs(), 1000);
    clock.advance(60);
    assert_eq!(clock.now_unix_secs(), 1060);
}

#[test]
fn in_memory_filesystem_round_trips() {
    let fs = InMemoryFilesystem::new();
    let path = std::path::PathBuf::from("/tmp/xrat-test/config.toml");
    fs.write_string(&path, "hello").unwrap();
    assert!(fs.exists(&path));
    assert_eq!(fs.read_to_string(&path).unwrap(), "hello");
}

#[test]
fn summary_maps_config_fields() {
    let row = sample_joined_row(7, true);
    let summary = ConfigSummary::from_joined(&row);
    assert_eq!(summary.id, xrat_model::ConfigId(7));
    assert_eq!(summary.r#ref, "ref7");
    assert_eq!(summary.name.as_deref(), Some("node-7"));
    assert!(summary.is_enabled);
    assert!(summary.latest_test.is_none());
}

#[test]
fn latest_test_is_none_without_test_id() {
    let row = sample_joined_row(1, true);
    assert_eq!(LatestTestSummary::from_joined(&row), None);
}

#[test]
fn request_defaults_disable_all_filters() {
    let filter: ConfigListFilter = crate::app::services::ConfigListRequest::default().filter();
    assert!(!filter.only_enabled);
    assert!(!filter.only_active);
    assert!(!filter.only_deleted);
    assert!(!filter.include_deleted);
    assert!(filter.subscription_id.is_none());
    assert!(filter.protocol.is_none());
}
