pub mod clipboard;
mod configs;
pub mod dashboard;
#[cfg(any(windows, test))]
pub(crate) mod desktop_proxy;
pub mod engine_probe;
mod lifecycle;
pub mod machine_service;
pub mod proxy_pac;
pub(crate) mod release_notes;
pub mod releases;
pub mod rotation;
pub mod runtime_control;
pub mod runtime_transitions;
pub mod runtime_tuning;
pub mod split_tunnel;
pub mod testing;
pub(crate) mod tun;
pub(crate) mod tun_control;

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

mod contracts;
pub use contracts::AppServices;

pub(crate) mod cores;
pub(crate) mod web_management;

pub(crate) mod upgrades;

pub(crate) mod geoip_assets;
