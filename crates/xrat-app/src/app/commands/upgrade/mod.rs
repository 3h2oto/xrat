mod release;
mod source;

use std::path::{Path, PathBuf};
use xrat_support::process::Command;

use crate::app::AppError;
use crate::app::commands::output;
use crate::cli::UpgradeArgs;

pub(crate) use crate::app::services::releases::REPO;

#[cfg(test)]
mod tests;

mod updater;
pub(crate) use updater::current_version;
pub(crate) use updater::install_binary;
pub(crate) use updater::restart_daemon_if_active;
pub use updater::run;
pub(crate) use updater::run_post_upgrade_migrations;
pub(crate) use updater::run_post_upgrade_migrations_with_spawner;
pub(crate) use updater::run_post_upgrade_tun_setup_with_spawner;
pub(crate) use updater::same_version;
pub(crate) use updater::should_restart_daemon;
pub(crate) use updater::should_restore_tun;
