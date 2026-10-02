mod configs;
pub mod dashboard;
pub mod engine_probe;
mod lifecycle;
pub mod proxy_pac;
pub mod releases;
pub mod rotation;
pub mod runtime_control;
pub mod runtime_transitions;
pub mod runtime_tuning;
pub mod testing;

pub use configs::{
    ConfigExportRequest, ConfigListRequest, ConfigListResult, ConfigService,
    DatabaseConfigRepository, MAX_TOP, enrich_endpoint_locations, validate_top,
};
pub use lifecycle::{ConfigLifecycleService, DeleteOutcome, RestoreOutcome, ToggleOutcome};

#[cfg(test)]
pub mod test_support;

use std::sync::Arc;

use crate::app::context::AppContext;
use crate::app::ports::{Clock, Filesystem, RealFilesystem, SystemClock};

/// Application services shared by every interface (CLI, TUI, HTTP, daemon).
///
/// Construct once per process and pass to handlers. Services depend on ports,
/// not on concrete infrastructure, so tests can build an `AppServices` with
/// fakes.
#[derive(Clone)]
pub struct AppServices {
    pub configs: ConfigService,
    pub lifecycle: ConfigLifecycleService,
    pub clock: Arc<dyn Clock>,
    pub filesystem: Arc<dyn Filesystem>,
}

impl AppServices {
    /// Wire production services from an application context.
    pub fn from_context(context: &AppContext) -> Self {
        Self::from_database(context.db.clone())
    }

    /// Wire production services from a database handle alone.
    ///
    /// Used by hosts that only need config services, such as the HTTP API
    /// server, without building a full [`AppContext`].
    pub fn from_database(db: xrat_db::Database) -> Self {
        let repository = Arc::new(DatabaseConfigRepository::new(db));
        Self {
            configs: ConfigService::new(Arc::clone(&repository)),
            lifecycle: ConfigLifecycleService::new(repository),
            clock: Arc::new(SystemClock),
            filesystem: Arc::new(RealFilesystem),
        }
    }

    /// Wire services with explicit ports, for tests and alternative hosts.
    pub fn with_ports(
        configs: ConfigService,
        lifecycle: ConfigLifecycleService,
        clock: Arc<dyn Clock>,
        filesystem: Arc<dyn Filesystem>,
    ) -> Self {
        Self {
            configs,
            lifecycle,
            clock,
            filesystem,
        }
    }
}
