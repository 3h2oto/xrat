mod archive;
mod release;
mod source;
#[cfg(windows)]
mod windows;

use std::path::{Path, PathBuf};
use xrat_support::process::Command;

use crate::app::AppError;
#[cfg(not(windows))]
use crate::app::commands::output;
use crate::app::services::upgrades::UpgradeOptions as UpgradeArgs;

pub(crate) use crate::app::services::releases::REPO;

#[cfg(test)]
mod tests;

mod updater;
pub(crate) use updater::current_version;
pub(crate) use updater::install_binary;
#[cfg(not(windows))]
pub(crate) use updater::restart_daemon_if_active;
pub(crate) use updater::run;
pub(crate) use updater::run_post_upgrade_migrations;
#[cfg(not(windows))]
pub(crate) use updater::run_post_upgrade_migrations_with_spawner;
#[cfg(not(windows))]
pub(crate) use updater::run_post_upgrade_tun_setup_with_spawner;
pub(crate) use updater::same_version;
#[cfg(any(not(windows), test))]
pub(crate) use updater::should_restart_daemon;
#[cfg(any(not(windows), test))]
pub(crate) use updater::should_restore_tun;

#[derive(Debug)]
pub(crate) enum UpgradeOutcome {
    Current,
    #[cfg(not(windows))]
    Completed,
    #[cfg(windows)]
    Staged(std::path::PathBuf),
}

impl UpgradeOutcome {
    pub(crate) fn message(&self) -> String {
        match self {
            Self::Current => "Already using the requested XRAT version".into(),
            #[cfg(not(windows))]
            Self::Completed => "XRAT executable updated; restart existing clients to use it".into(),
            #[cfg(windows)]
            Self::Staged(path) => format!(
                "Upgrade staged; exit XRAT to apply it. Check `xrat upgrade --status` after exit. Transaction: {}",
                path.display()
            ),
        }
    }
}

#[cfg(windows)]
pub(crate) use windows::{apply_transaction, status};

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
