use super::*;

pub(super) const DEFAULT_PORT_BASE: u16 = 20000;

/// Builds isolated [`AppContext`] values for tests.
///
/// Each builder owns a temp root and, by default, assigns a unique inbound port
/// block so tests that launch a runtime can run in parallel without colliding on
/// the fixed default ports.
pub struct TestAppBuilder {
    pub(super) prefix: String,
    pub(super) unique_ports: bool,
}

impl TestAppBuilder {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            unique_ports: true,
        }
    }

    /// Keep the default runtime ports instead of a unique per-test block.
    pub fn with_default_ports(mut self) -> Self {
        self.unique_ports = false;
        self
    }

    /// Build an [`AppContext`] whose temp root stays alive for the test.
    ///
    /// The root is intentionally leaked: callers hold only the context, so the
    /// directory must outlive the surrounding test rather than being dropped
    /// here. Tests that need to own the directory use
    /// [`build_with_root`](Self::build_with_root).
    pub async fn build(self) -> AppContext {
        let (context, root) = self.build_with_root().await;
        std::mem::forget(root);
        context
    }

    /// Build an [`AppContext`] plus the [`tempfile::TempDir`] that owns its root.
    ///
    /// Tests that need to hold the directory (or drop it deterministically) use
    /// this instead of [`build`](Self::build).
    pub async fn build_with_root(self) -> (AppContext, tempfile::TempDir) {
        let root = tempfile::tempdir().expect("temp directory should be created");
        let root_dir = root.path().to_path_buf();
        let database_path = root_dir.join("db.sqlite");
        let database_config = DatabaseConnectionConfig::Sqlite {
            path: database_path.clone(),
        };
        let db = Database::connect(&database_config)
            .await
            .expect("database should connect");

        let mut app_config = AppConfig::default();
        if self.unique_ports {
            let base = next_port_base();
            app_config.runtime.socks.port = base;
            app_config.runtime.http.port = base + 1;
            app_config.runtime.shadowsocks.port = base + 2;
        }

        let runtime_paths = RuntimePaths {
            root_dir,
            database_config,
            database_path: database_path.clone(),
            database_label: database_path.display().to_string(),
            config_path: root.path().join("config.toml"),
            runtime_dir: root.path().join("runtime"),
            xray_path: "xray".into(),
            v2ray_path: "v2ray".into(),
            sing_box_path: "sing-box".into(),
        };

        let context = AppContext {
            db,
            app_config,
            runtime_paths,
        };
        let _ = self.prefix;
        (context, root)
    }
}

pub(super) fn next_port_base() -> u16 {
    static COUNTER: AtomicU16 = AtomicU16::new(0);
    let slot = COUNTER.fetch_add(1, Ordering::Relaxed) % 4000;
    DEFAULT_PORT_BASE + slot * 4
}
