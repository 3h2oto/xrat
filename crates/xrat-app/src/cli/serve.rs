use clap::Args;

#[derive(Debug, Args)]
pub struct ServeArgs {
    #[arg(long, help = "Override HTTP API bind host.")]
    pub host: Option<String>,
    #[arg(long, help = "Override HTTP API bind port.")]
    pub port: Option<u16>,
}

#[derive(Debug, Args)]
pub struct WebUiArgs {
    #[command(flatten)]
    pub serve: ServeArgs,
    #[arg(long, help = "Do not open a browser.")]
    pub no_open: bool,
}

#[derive(Debug, Args)]
pub struct HashPasswordArgs {}
