mod release;
mod source;

use std::path::{Path, PathBuf};
use xrat_support::process::Command;

use crate::app::AppError;
use crate::app::commands::output;
use crate::app::services::upgrades::UpgradeOptions as UpgradeArgs;

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

#[derive(Clone, Debug, Default)]
pub struct UpgradeOptions {
    pub source: bool,
    pub path: std::path::PathBuf,
    pub version: Option<String>,
    pub force: bool,
    pub timeout_secs: u64,
    pub tun_setup: bool,
    pub no_tun_setup: bool,
    pub no_restart_daemon: bool,
}
impl From<&crate::cli::UpgradeArgs> for UpgradeOptions {
    fn from(args: &crate::cli::UpgradeArgs) -> Self {
        Self {
            source: args.source,
            path: args.path.clone(),
            version: args.version.clone(),
            force: args.force,
            timeout_secs: args.timeout_secs,
            tun_setup: args.tun_setup,
            no_tun_setup: args.no_tun_setup,
            no_restart_daemon: args.no_restart_daemon,
        }
    }
}
