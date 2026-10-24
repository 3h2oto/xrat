use clap::{Args, Subcommand};

#[derive(Debug, Args)]
#[command(about = "Run or control the XRAT daemon supervisor process.")]
pub struct DaemonArgs {
    #[command(subcommand)]
    pub action: DaemonAction,
}

#[derive(Debug, Subcommand)]
pub enum DaemonAction {
    #[command(about = "Start the long-lived XRAT daemon process.")]
    Start(DaemonStartArgs),
    #[command(hide = true, about = "Internal: run the daemon IPC server loop.")]
    RunServer(DaemonServeArgs),
    #[cfg(windows)]
    #[command(
        hide = true,
        about = "Internal: run the Windows SCM service entrypoint."
    )]
    RunService,
    #[command(about = "Show daemon IPC reachability and protocol information.")]
    Status(DaemonStatusArgs),
    #[command(about = "Request daemon shutdown via local IPC.")]
    Stop(DaemonStopArgs),
    #[command(about = "Restart the daemon, reloading config.toml and the runtime session.")]
    Restart(DaemonRestartArgs),
    #[command(about = "Install the platform's XRAT daemon service.")]
    Install(DaemonInstallArgs),
    #[command(about = "Remove the installed XRAT daemon service, preserving its data.")]
    Uninstall(DaemonUninstallArgs),
}

#[derive(Debug, Args, Default)]
pub struct DaemonStartArgs {}

#[derive(Debug, Args, Default)]
pub struct DaemonServeArgs {}

#[derive(Debug, Args, Default)]
pub struct DaemonStatusArgs {}

#[derive(Debug, Args, Default)]
pub struct DaemonStopArgs {}

#[derive(Debug, Args, Default)]
pub struct DaemonRestartArgs {}

#[derive(Debug, Args)]
pub struct DaemonInstallArgs {
    /// Windows operator SID when installing using a different administrator account.
    #[cfg(windows)]
    #[arg(long)]
    pub operator_sid: Option<String>,
    /// Start the daemon immediately after enabling.
    #[arg(long)]
    pub start: bool,

    /// Print the generated service unit without writing files.
    #[arg(long)]
    pub dry_run: bool,

    /// Install the API server alongside the daemon (Windows: operator login task).
    #[arg(long)]
    pub with_api: bool,

    /// Configure the systemd service override for TUN capture (disables NoNewPrivileges).
    #[arg(long)]
    pub tun: bool,
}

#[derive(Debug, Args)]
pub struct DaemonUninstallArgs {
    /// Print actions without executing them.
    #[arg(long)]
    pub dry_run: bool,
}
