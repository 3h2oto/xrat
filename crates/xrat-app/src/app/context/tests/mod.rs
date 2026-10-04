use std::path::PathBuf;

use clap::Parser;

use super::paths::resolve_runtime;
use crate::cli::Cli;

mod binary_cases;
mod database_cases;

mod fixtures;
use fixtures::*;
