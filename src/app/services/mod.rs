mod configs;

pub use configs::{
    ConfigListRequest, ConfigListResult, ConfigService, DatabaseConfigRepository,
    enrich_endpoint_locations,
};

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
    pub clock: Arc<dyn Clock>,
    pub filesystem: Arc<dyn Filesystem>,
}

impl AppServices {
    /// Wire production services from an application context.
    pub fn from_context(context: &AppContext) -> Self {
        Self {
            configs: ConfigService::new(DatabaseConfigRepository::new(context.db.clone())),
            clock: Arc::new(SystemClock),
            filesystem: Arc::new(RealFilesystem),
        }
    }

    /// Wire services with explicit ports, for tests and alternative hosts.
    pub fn with_ports(
        configs: ConfigService,
        clock: Arc<dyn Clock>,
        filesystem: Arc<dyn Filesystem>,
    ) -> Self {
        Self {
            configs,
            clock,
            filesystem,
        }
    }
}
