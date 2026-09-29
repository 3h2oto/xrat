use super::configs::*;
use super::prelude::*;
use super::subscriptions::*;
use crate::app::commands::output;

pub async fn run(context: &AppContext, command: &ListArgs) -> crate::app::Result<()> {
    match &command.target {
        ListTarget::Configs(filters) => print_configs(context, filters).await?,
        ListTarget::Subscriptions(filters) => print_subscriptions(context, filters).await?,
    }

    Ok(())
}

async fn print_configs(context: &AppContext, filters: &ListConfigsArgs) -> crate::app::Result<()> {
    let services = context.services();
    let request = build_config_list_request(context, filters).await?;
    let mut configs = services.configs.list(&request).await?.items;
    crate::app::services::enrich_endpoint_locations(
        &context.app_config,
        &context.runtime_paths,
        &mut configs,
    )
    .await;
    let subscriptions = services.configs.subscriptions().await?;
    let subscription_refs = subscriptions
        .iter()
        .map(|subscription| (subscription.id, subscription.r#ref.as_str()))
        .collect::<HashMap<_, _>>();

    if configs.is_empty() {
        println!("{}", output::empty_message("No configs matched."));
        return Ok(());
    }

    println!(
        "{}",
        format_configs(
            &configs,
            &subscription_refs,
            filters.format,
            Some(&context.app_config.testing),
        )?
    );

    Ok(())
}

async fn print_subscriptions(
    context: &AppContext,
    filters: &ListSubscriptionsArgs,
) -> crate::app::Result<()> {
    let mut subscriptions = context.services().configs.subscriptions().await?;
    if let Some(kind) = &filters.kind {
        subscriptions.retain(|subscription| subscription.source_kind == kind.as_str());
    }

    if subscriptions.is_empty() {
        println!("{}", output::empty_message("No subscriptions matched."));
        return Ok(());
    }

    println!("{}", format_subscriptions(&subscriptions, filters.format)?);

    Ok(())
}

async fn build_config_list_request(
    context: &AppContext,
    args: &ListConfigsArgs,
) -> crate::app::Result<crate::app::services::ConfigListRequest> {
    let subscription_id = match &args.subscription {
        Some(raw) => Some(resolve_subscription_id(context, raw).await?),
        None => None,
    };

    Ok(crate::app::services::ConfigListRequest {
        only_enabled: args.enabled_only,
        only_active: args.active_only,
        only_deleted: args.deleted_only,
        include_deleted: args.include_deleted,
        subscription_id,
        protocol: None,
        ..crate::app::services::ConfigListRequest::default()
    })
}
