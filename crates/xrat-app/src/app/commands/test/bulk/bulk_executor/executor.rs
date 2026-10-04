use super::*;

pub(in super::super) async fn run_single(
    args: &TestArgs,
    context: &AppContext,
    settings: ResolvedTestSettings,
    config_id: ConfigId,
) -> crate::app::Result<()> {
    single::run_single(args, context, settings, config_id).await
}

pub(in super::super) async fn run_bulk(
    args: &TestArgs,
    context: &AppContext,
    settings: ResolvedTestSettings,
    subscription_id: Option<SubscriptionId>,
) -> crate::app::Result<()> {
    bulk::run_bulk(args, context, settings, subscription_id).await
}
