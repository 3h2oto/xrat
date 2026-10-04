use clap::{Args, Subcommand};

#[derive(Debug, Args)]
#[command(about = "Prepare and inspect system privileges for managed TUN capture.")]
pub struct TunArgs {
    #[command(subcommand)]
    pub action: TunAction,
}

#[derive(Debug, Subcommand)]
pub enum TunAction {
    #[command(about = "Report TUN readiness: engine, interface, and file capabilities.")]
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
