use super::*;

pub(super) async fn run_bulk(
    args: &TestArgs,
    context: &AppContext,
    settings: ResolvedTestSettings,
    subscription_id: Option<SubscriptionId>,
) -> crate::app::Result<()> {
    let configs = context
        .db
        .list_configs(&args.config_filter(subscription_id))
        .await?;
    if configs.is_empty() {
        tracing::info!("no configs found for requested test filters");
        write_results(args, &[])?;
        return Ok(());
    }

    let mut outputs = crate::app::services::testing::run_bulk_for_configs(
        context,
        settings,
        configs,
        "bulk",
        !args.no_progress,
    )
    .await?;
    sort_results(&mut outputs, args.sort_by);
    write_results(args, &outputs)?;
    Ok(())
}
