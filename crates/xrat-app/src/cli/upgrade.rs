use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args, Default)]
#[command(about = "Self-upgrade xrat from the latest release or by building from source.")]
pub struct UpgradeArgs {
    #[arg(
        long = "source",
        help = "Build and install from source instead of downloading a release."
    )]
    pub source: bool,
    #[arg(
        long = "path",
        default_value = ".",
        help = "Source directory to build from when --source is set."
    )]
    pub path: PathBuf,
    #[arg(
        long = "version",
        help = "Download a specific release tag instead of the latest."
    )]
    pub version: Option<String>,
    #[arg(
        long = "force",
        help = "Reinstall even when already on the requested version."
    )]
    pub force: bool,
    #[arg(
        long = "timeout",
        help = "HTTP request timeout in seconds for release downloads.",
        default_value_t = 120
    )]
    pub timeout_secs: u64,
    #[arg(
        long = "tun-setup",
        help = "Force running `tun setup` after upgrade to restore capabilities.",
        conflicts_with = "no_tun_setup"
    )]
    pub tun_setup: bool,
    #[arg(
        long = "no-tun-setup",
        help = "Skip restoring TUN capabilities after upgrade."
    )]
    pub no_tun_setup: bool,
    #[arg(
        long = "no-restart-daemon",
        help = "Do not restart running xrat-daemon after upgrade."
    )]
    pub no_restart_daemon: bool,
}
