use xrat::app::{commands, context::AppContext};
use xrat::cli;

#[tokio::main]
async fn main() {
    let args = cli::parse();
    init_tracing(&args);

    if let Err(err) = run(&args).await {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

async fn run(args: &cli::Cli) -> xrat::app::Result<()> {
    if args.profile == cli::ExecutionProfile::Machine {
        if args.database.is_some()
            || args.config.is_some()
            || args.xray.is_some()
            || args.v2ray.is_some()
            || args.sing_box.is_some()
        {
            return Err(xrat::app::AppError::InvalidArgument(
                "machine profile does not accept database, config, or engine path overrides".into(),
            ));
        }
        #[cfg(windows)]
        return commands::machine::run(&args.command).await;
        #[cfg(not(windows))]
        return Err(xrat::app::AppError::UnsupportedPlatform(
            "machine profile requires Windows".into(),
        ));
    }
    #[cfg(windows)]
    if let cli::Command::Daemon(daemon) = &args.command {
        use xrat::app::commands::machine;
        match &daemon.action {
            cli::DaemonAction::Install(install) => return machine::install(install, false),
            cli::DaemonAction::Uninstall(uninstall) => return machine::uninstall(uninstall),
            _ => {}
        }
    }
    #[cfg(windows)]
    if matches!(
        &args.command,
        cli::Command::Daemon(cli::DaemonArgs {
            action: cli::DaemonAction::RunService
        })
    ) {
        return xrat::app::services::machine_service::run_service();
    }
    if let cli::Command::HashPassword(_) = &args.command {
        return xrat::app::commands::password_hash();
    }
    if let cli::Command::Validate(validate_args) = &args.command {
        return xrat::app::commands::validate::run(validate_args);
    }
    if let cli::Command::Upgrade(upgrade_args) = &args.command {
        let config_path = xrat::app::context::resolve_config_path(args)?;
        return xrat::app::commands::upgrade::run(&config_path, upgrade_args).await;
    }

    let context = AppContext::build(args).await?;
    commands::run(&context, &args.command).await?;

    Ok(())
}

fn init_tracing(args: &cli::Cli) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(args.default_log_filter()));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .compact()
        .init();
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    #[tokio::test]
    async fn machine_profile_rejects_path_overrides_before_bootstrap() {
        let root = tempfile::tempdir().unwrap();
        let database = root.path().join("must-not-exist.sqlite");
        let args = xrat::cli::Cli::try_parse_from([
            "xrat",
            "--profile",
            "machine",
            "--database",
            database.to_str().unwrap(),
            "daemon",
            "status",
        ])
        .unwrap();
        let error = super::run(&args).await.unwrap_err();
        assert!(error.to_string().contains("does not accept"));
        assert!(!database.exists());
    }

    #[test]
    fn user_profile_remains_default_and_machine_selection_is_explicit() {
        let user = xrat::cli::Cli::try_parse_from(["xrat", "daemon", "status"]).unwrap();
        assert_eq!(user.profile, xrat::cli::ExecutionProfile::User);
        let machine =
            xrat::cli::Cli::try_parse_from(["xrat", "daemon", "status", "--profile", "machine"])
                .unwrap();
        assert_eq!(machine.profile, xrat::cli::ExecutionProfile::Machine);
    }

    #[tokio::test]
    async fn upgrade_bypasses_config_and_database_initialization() {
        let root = tempfile::tempdir().expect("temporary directory should exist");
        let config_path = root.path().join("invalid.toml");
        std::fs::write(&config_path, "this is not valid TOML = [")
            .expect("invalid config fixture should write");
        let args = xrat::cli::Cli::try_parse_from([
            "xrat",
            "--config",
            config_path.to_str().expect("config path should be UTF-8"),
            "upgrade",
            "--version",
            concat!("v", env!("CARGO_PKG_VERSION")),
        ])
        .expect("upgrade arguments should parse");

        super::run(&args)
            .await
            .expect("current-version upgrade should not initialize the database");
    }
}
