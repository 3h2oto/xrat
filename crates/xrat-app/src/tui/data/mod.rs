mod configs;
pub(crate) mod logs;
mod probe_stats;
mod runtime;
mod sources;
mod stats;
mod tests_view;

pub use configs::TuiConfigRow;
pub use logs::TuiLogs;
pub use probe_stats::{MetricSummary, TuiProbeHistory};
pub use runtime::TuiRuntimeStatus;
pub use sources::TuiSourceRow;
pub use stats::StatsHistory;
pub use tests_view::TuiTestStatus;
pub use xrat_support::engine_log::{EngineLogRow as TuiProxyLogRow, ProxyStream};

use crate::app::services::dashboard::{DashboardService, DashboardSnapshot};

pub use crate::app::ports::EngineInfo;

pub use crate::app::services::dashboard::DaemonOverview as TuiDaemonInfo;

#[cfg(test)]
mod stage_tests;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod overview_tests;

mod snapshot;
pub use snapshot::TuiData;
pub use snapshot::TuiMetricColumns;
#[cfg(test)]
use snapshot::{format_test_stage_label, normalize_test_stage_names};
