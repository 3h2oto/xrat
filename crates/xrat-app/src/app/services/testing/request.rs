use xrat_db::ConfigListFilter;
use xrat_model::SubscriptionId;

/// Stage and filter selection for a connection-test run.
///
/// This is the application-level input shared by CLI and TUI. CLI-only options
/// such as output format, sort order, ping mode, and latest-run summary stay in
/// the CLI argument type and never reach the test core.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TestRunRequest {
    /// Filter: only enabled configs.
    pub enabled_only: bool,
    /// Filter: only the active config.
    pub active_only: bool,

    /// Skip flags, mirroring the CLI defaults so callers can opt in per stage.
    pub skip_icmp: bool,
    pub skip_tcp: bool,
    pub skip_real_delay: bool,
    pub skip_download: bool,
    pub skip_upload: bool,

    pub test_url: Option<String>,
    pub download_url: Option<String>,
    pub upload_url: Option<String>,

    pub icmp_timeout_ms: Option<u64>,
    pub tcp_timeout_ms: Option<u64>,
    pub real_delay_timeout_ms: Option<u64>,
    pub download_timeout_ms: Option<u64>,
    pub upload_timeout_ms: Option<u64>,

    pub concurrency: Option<i32>,
}

impl TestRunRequest {
    /// Build the list filter for bulk selection.
    pub fn config_filter(&self, subscription_id: Option<SubscriptionId>) -> ConfigListFilter {
        ConfigListFilter {
            only_enabled: self.enabled_only,
            only_active: self.active_only,
            only_deleted: false,
            include_deleted: false,
            subscription_id,
            protocol: None,
        }
    }
}
