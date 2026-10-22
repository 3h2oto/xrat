mod activity;
mod activity_progress;
pub(crate) use activity_progress::TestActivityProgress;
mod bulk;
mod bulk_progress;
mod execution;
#[cfg(any(windows, test))]
mod machine;
mod output_types;
mod progress;
mod request;
mod settings;
mod stages;

use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::Serialize;
use tokio::task::JoinSet;

use crate::app::AppError;
use crate::app::config::defaults;
use crate::app::config::{AppConfig, ConnectionTestStage, TestFailurePolicy};
use crate::app::context::{AppContext, RuntimePaths};
use xrat_db::{ConfigRecord, ConnectionTestInsert, ConnectionTestRunInsert, Database};
use xrat_model::{ConfigId, Node};
use xrat_prober::{
    AcceptedHttpStatuses, FailureKind, TestResult, download_speed_check, icmp_ping,
    real_delay_check, tcp_check, upload_speed_check,
};
use xrat_support::geoip;

pub(crate) use activity::{TestActivity, TestActivitySender, TestMetric};
pub(crate) use bulk::{run_bulk_for_configs, run_bulk_for_configs_cancellable};
pub(crate) use execution::test_and_record_config;
pub(crate) use output_types::*;
pub use progress::{TestProgressUpdate, run_bulk_for_config_ids_with_progress};
pub(crate) use progress::{run_bulk_for_config_ids_with_activity, run_machine_probe};
pub use request::TestRunRequest;
pub(crate) use settings::*;
use stages::*;
#[cfg(test)]
pub(crate) use stages::{classify_endpoint_location, resolve_endpoint_meta};

#[cfg(any(windows, test))]
pub(crate) use machine::run_for_client as run_machine_for_config_ids;
