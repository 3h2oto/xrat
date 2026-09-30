mod bulk;
mod handlers;
mod output;

use std::cmp::Ordering;
#[cfg(test)]
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;

use crate::app::AppError;
use crate::app::config::ConnectionTestStage;
#[cfg(test)]
use crate::app::config::{AppConfig, TestFailurePolicy};
use crate::app::context::AppContext;
use crate::app::services::testing::*;
pub(crate) use crate::app::services::testing::{
    TestProgressUpdate, run_bulk_for_config_ids_with_progress,
};
use crate::cli::{TestArgs, TestFormat, TestSortBy};
#[cfg(test)]
use xrat_db::DatabaseConnectionConfig;
use xrat_db::{ConfigRecord, ConnectionTestRunInsert, Database};
#[cfg(test)]
use xrat_support::geoip;

pub(crate) use bulk::run_rotation_bulk_tests;
use bulk::*;
pub use handlers::run;
use output::*;

#[cfg(test)]
mod tests;
