use clap::{Args, Subcommand};

#[derive(Debug, Args)]
#[command(about = "Enable, disable, and inspect managed TUN capture.")]
pub struct TunArgs {
    #[command(subcommand)]
    pub action: TunAction,
}

#[derive(Debug, Subcommand)]
pub enum TunAction {
    #[command(
        about = "Enable TUN on the current connection, or on the next connect when disconnected."
    )]
    Enable(TunModeArgs),
    #[command(about = "Disable TUN and keep the current config connected in proxy mode.")]
    Disable(TunModeArgs),
    #[command(about = "Report configured mode, active capture, engine support, and privileges.")]
    Status(TunStatusArgs),
    #[command(about = "Grant CAP_NET_ADMIN/CAP_NET_RAW to the files TUN needs via setcap.")]
    Setup(TunSetupArgs),
}

#[derive(Debug, Args, Default)]
pub struct TunStatusArgs {
    #[arg(long = "json", help = "Print the result as JSON.")]
    pub json: bool,
}

#[derive(Debug, Args, Default)]
pub struct TunSetupArgs {
    #[arg(long, help = "Print the setcap command without running it.")]
    pub dry_run: bool,
}

#[derive(Debug, Args, Default)]
pub struct TunModeArgs {
    #[arg(long, help = "Print the applied TUN state as JSON.")]
    pub json: bool,
}
