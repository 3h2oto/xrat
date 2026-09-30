use super::super::*;

mod bulk;
mod single;

pub(super) async fn run_single(
    args: &TestArgs,
    context: &AppContext,
    settings: ResolvedTestSettings,
    config_id: i64,
) -> crate::app::Result<()> {
    single::run_single(args, context, settings, config_id).await
}

pub(super) async fn run_bulk(
    args: &TestArgs,
    context: &AppContext,
    settings: ResolvedTestSettings,
    subscription_id: Option<i64>,
) -> crate::app::Result<()> {
    bulk::run_bulk(args, context, settings, subscription_id).await
}
