pub async fn run(
    config_path: &std::path::Path,
    args: &crate::cli::UpgradeArgs,
) -> crate::app::Result<()> {
    crate::app::services::upgrades::run(config_path, &args.into()).await
}
