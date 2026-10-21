pub async fn run(
    config_path: &std::path::Path,
    args: &crate::cli::UpgradeArgs,
) -> crate::app::Result<()> {
    #[cfg(windows)]
    {
        if let Some(path) = &args.apply_transaction {
            return crate::app::services::upgrades::apply_transaction(path).await;
        }
        if args.status {
            println!("{}", crate::app::services::upgrades::status()?);
            return Ok(());
        }
    }
    let outcome = crate::app::services::upgrades::run(config_path, &args.into()).await?;
    println!("{}", outcome.message());
    Ok(())
}
