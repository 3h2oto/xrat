use super::entry::unexpected;
use crate::app::Result;
use crate::app::commands::{list, output};
use crate::app::services::{
    ConfigListRequest,
    machine_service::{CatalogPayload, CatalogRequest, MachineClient},
};
use crate::cli::{ListArgs, ListTarget};

pub(super) async fn run(client: &MachineClient, args: &ListArgs) -> Result<()> {
    match &args.target {
        ListTarget::Configs(args) => {
            let subscription_id = match &args.subscription {
                Some(reference) => match client
                    .catalog(CatalogRequest::ResolveSubscription {
                        reference: reference.clone(),
                    })
                    .await?
                {
                    CatalogPayload::SubscriptionId(id) => Some(id),
                    _ => return Err(unexpected()),
                },
                None => None,
            };
            let request = ConfigListRequest {
                only_enabled: args.enabled_only,
                only_active: args.active_only,
                only_deleted: args.deleted_only,
                include_deleted: args.include_deleted,
                subscription_id,
                ..Default::default()
            };
            let configs = client.list(&request).await?.items;
            let subscriptions = client.subscriptions().await?;
            let references = subscriptions
                .iter()
                .map(|record| (record.id, record.r#ref.as_str()))
                .collect();
            if configs.is_empty() {
                println!("{}", output::empty_message("No configs matched."));
                return Ok(());
            }
            println!(
                "{}",
                list::format_configs(&configs, &references, args.format, None)?
            );
        }
        ListTarget::Subscriptions(args) => {
            let mut subscriptions = client.subscriptions().await?;
            if let Some(kind) = &args.kind {
                subscriptions.retain(|record| record.source_kind == kind.as_str());
            }
            if subscriptions.is_empty() {
                println!("{}", output::empty_message("No subscriptions matched."));
                return Ok(());
            }
            println!(
                "{}",
                list::format_subscriptions(&subscriptions, args.format)?
            );
        }
    }
    Ok(())
}
