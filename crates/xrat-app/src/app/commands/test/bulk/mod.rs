use super::*;

mod bulk_executor;
mod distribution;
mod rotation;

pub(crate) use rotation::run_rotation_bulk_tests;

pub(super) fn print_geo_distribution<'a>(label: &str, values: impl Iterator<Item = &'a str>) {
    distribution::print_geo_distribution(label, values);
}

pub(super) async fn run_single(
    args: &TestArgs,
    context: &AppContext,
    settings: ResolvedTestSettings,
    config_id: ConfigId,
) -> crate::app::Result<()> {
    bulk_executor::run_single(args, context, settings, config_id).await
}

pub(super) async fn run_bulk(
    args: &TestArgs,
    context: &AppContext,
    settings: ResolvedTestSettings,
    subscription_id: Option<SubscriptionId>,
) -> crate::app::Result<()> {
    bulk_executor::run_bulk(args, context, settings, subscription_id).await
}
