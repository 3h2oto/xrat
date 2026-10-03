mod release;
mod source;

use std::path::{Path, PathBuf};
use xrat_support::process::Command;

use crate::app::AppError;
use crate::cli::UpgradeArgs;

pub(crate) use crate::app::services::releases::REPO;

#[cfg(test)]
mod tests;

mod updater;
pub(crate) use updater::current_version;
pub(crate) use updater::install_binary;
pub use updater::run;
pub(crate) use updater::run_post_upgrade_migrations;
pub(crate) use updater::run_post_upgrade_migrations_with_spawner;
pub(crate) use updater::same_version;
