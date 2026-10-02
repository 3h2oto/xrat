mod daemon;
mod enrichment;
mod geo;
mod logs;

pub use enrichment::enrich_locations;
pub use geo::build_geo_lookup;
pub use logs::{DashboardEvent, DashboardLogs};

use crate::app::context::AppContext;
use crate::app::ports::{Clock, SystemClock};
use crate::app::read_models::ConfigDetail;
use crate::app::runtime_service::{RuntimeService, RuntimeStatusSnapshot};
use std::sync::Arc;
use xrat_db::{ConnectionTestRecord, ConnectionTestRunRecord, SubscriptionRecord};
use xrat_model::ConfigId;

#[derive(Debug, Clone, Default)]
pub struct DaemonOverview {
    pub running: bool,
    pub rotation_enabled: bool,
    pub interval_secs: u64,
}

#[derive(Debug)]
pub struct DashboardSnapshot {
    pub configs: Vec<ConfigDetail>,
    pub sources: Vec<SubscriptionRecord>,
    pub runtime: RuntimeStatusSnapshot,
    pub local_address: Option<String>,
    pub latest_run: Option<ConnectionTestRunRecord>,
    pub test_results: Vec<ConnectionTestRecord>,
    pub logs: DashboardLogs,
    pub probe_history: Vec<ConnectionTestRecord>,
    pub daemon: DaemonOverview,
    pub db_label: String,
    pub config_path: String,
    pub api_b64_url: String,
    pub server_enabled: bool,
    pub test_stages: Vec<String>,
    pub pending_enrichment: Vec<(ConfigId, String)>,
}

pub struct DashboardService<'a> {
    context: &'a AppContext,
    clock: Arc<dyn Clock>,
}

impl<'a> DashboardService<'a> {
    pub fn new(context: &'a AppContext) -> Self {
        Self::with_clock(context, Arc::new(SystemClock))
    }

    pub fn with_clock(context: &'a AppContext, clock: Arc<dyn Clock>) -> Self {
        Self { context, clock }
    }

    pub async fn clear_events(&self) -> crate::app::Result<u64> {
        Ok(self.context.db.clear_events().await?)
    }

    pub async fn load(&self, include_deleted: bool) -> crate::app::Result<DashboardSnapshot> {
        let context = self.context;
        let services = context.services();
        let request = super::ConfigListRequest {
            include_deleted,
            ..Default::default()
        };
        let mut configs = services.configs.list(&request).await?.items;
        configs.sort_by_key(|row| {
            (
                row.summary.test().real_delay_ms.unwrap_or(i64::MAX),
                row.summary.id,
            )
        });
        let pending_enrichment =
            geo::apply_geo_cache(context, &mut configs, self.clock.as_ref()).await;
        let sources = services.configs.subscriptions().await?;
        let runtime = RuntimeService::new(context).status().await?;
        let server = &context.app_config.server;
        let needs_local_address = matches!(server.host.as_str(), "0.0.0.0" | "::")
            || [
                &runtime.inbound_health.socks,
                &runtime.inbound_health.http,
                &runtime.inbound_health.shadowsocks,
            ]
            .into_iter()
            .flatten()
            .any(|health| matches!(health.endpoint.host.as_str(), "0.0.0.0" | "::"));
        let local_address = needs_local_address
            .then(xrat_support::net::primary_local_ip)
            .flatten();
        let latest_run = context.db.get_latest_connection_test_run().await?;
        let test_results = match &latest_run {
            Some(run) => context.db.list_connection_tests_by_run(run.id).await?,
            None => Vec::new(),
        };
        let logs = DashboardLogs::load(context).await?;
        let probe_history = match runtime.active_config.as_ref() {
            Some(config) => context.db.list_connection_tests(config.id).await?,
            None => Vec::new(),
        };
        Ok(DashboardSnapshot {
            configs,
            sources,
            runtime,
            latest_run,
            test_results,
            logs,
            probe_history,
            daemon: daemon::load_daemon_info(context).await,
            db_label: context.runtime_paths.database_label.clone(),
            config_path: context.runtime_paths.config_path.display().to_string(),
            api_b64_url: format!(
                "http://{}:{}/b64",
                display_host(&server.host, local_address.as_deref()),
                server.port
            ),
            local_address,
            server_enabled: server.enabled,
            test_stages: context.app_config.runtime.rotation.test_stages.clone(),
            pending_enrichment,
        })
    }
}

fn display_host(host: &str, local_address: Option<&str>) -> String {
    match host {
        "0.0.0.0" | "::" => local_address
            .map(str::to_string)
            .unwrap_or_else(|| xrat_support::net::connect_host_for_bind_host(host)),
        _ => host.to_string(),
    }
}

#[cfg(test)]
mod tests;
