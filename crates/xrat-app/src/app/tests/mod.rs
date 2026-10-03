//! Shared test support for application tests.
//!
//! Provides a single [`TestAppBuilder`] so command, runtime-service, supervisor,
//! and TUI tests build an [`AppContext`] the same way instead of hand-rolling a
//! temp root, SQLite config, and `RuntimePaths` in each module.

pub mod fixtures;
pub(crate) mod signals;

use std::sync::atomic::{AtomicU16, Ordering};

use crate::app::config::AppConfig;
use crate::app::context::{AppContext, RuntimePaths};
use xrat_db::{Database, DatabaseConnectionConfig};

#[cfg(test)]
mod builder_tests;

mod context_fixture;
pub use context_fixture::TestAppBuilder;
use context_fixture::*;
