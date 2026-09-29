mod bulk;
mod execution;
mod handlers;
mod output;
mod output_types;
mod progress;
mod settings;
mod stages;

use std::cmp::Ordering;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde::Serialize;
use tokio::task::JoinSet;

use crate::app::AppError;
use crate::app::config::defaults;
use crate::app::config::{AppConfig, ConnectionTestStage, TestFailurePolicy};
use crate::app::context::{AppContext, RuntimePaths};
use crate::app::services::testing::TestRunRequest;
use crate::cli::{TestArgs, TestFormat, TestSortBy};
#[cfg(test)]
use xrat_db::DatabaseConnectionConfig;
use xrat_db::{ConfigRecord, ConnectionTestInsert, ConnectionTestRunInsert, Database};
use xrat_model::Node;
use xrat_prober::{
    AcceptedHttpStatuses, FailureKind, TestResult, download_speed_check, icmp_ping,
    real_delay_check, tcp_check, upload_speed_check,
};
use xrat_support::geoip;

pub(crate) use bulk::run_rotation_bulk_tests;
use bulk::*;
use execution::*;
pub use handlers::run;
use output::*;
use output_types::*;
pub(crate) use progress::{TestProgressUpdate, run_bulk_for_config_ids_with_progress};
use settings::*;
use stages::*;

#[cfg(test)]
mod tests;
