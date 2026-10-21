use super::entry::{profile_labels, subscription, unexpected};
use crate::app::commands::{add, import as import_command, progress::CliProgress};
use crate::app::services::machine_service::{CatalogPayload, CatalogRequest, MachineClient};
use crate::app::{AppError, Result, import as import_service};
use crate::cli::ImportArgs;
use std::collections::BTreeMap;

pub(super) async fn import(client: &MachineClient, args: &ImportArgs) -> Result<()> {
    let progress = CliProgress::spinner(true, "importing subscription");
    let result = async {
        let headers: BTreeMap<String, String> = args
            .headers
            .iter()
            .map(|(name, value)| {
                (
                    name.to_string(),
                    String::from_utf8_lossy(value.as_bytes()).into_owned(),
                )
            })
            .collect();
        if args.url_list {
            client
                .import_url_list(&args.input, args.name.as_deref())
                .await
        } else {
            client
                .import_subscription(&args.input, args.name.as_deref(), &headers)
                .await
        }
    }
    .await;
    progress.finish_and_clear();
    let summary = result?;
    let subscription = subscription(client, &summary.subscription_id.to_string()).await?;
    let (database_label, config_path) = profile_labels(client).await?;
    import_command::print_summary(&summary, &subscription.r#ref, &database_label, &config_path);
    Ok(())
}

pub(super) async fn add(client: &MachineClient, input: &str) -> Result<()> {
    let progress = CliProgress::spinner(true, "adding config");
    let result = async {
        let (source, _node) = import_service::load_single_node(input)?;
        let source_kind = source.kind.as_str().to_string();
        let result = client
            .catalog(CatalogRequest::Import {
                contents: vec![input.into()],
                source,
                name: None,
                headers: BTreeMap::new(),
            })
            .await?;
        Ok::<_, AppError>((source_kind, result))
    }
    .await;
    progress.finish_and_clear();
    let (source_kind, CatalogPayload::Import(summary)) = result? else {
        return Err(unexpected());
    };
    let (database_label, config_path) = profile_labels(client).await?;
    add::print_summary(
        &source_kind,
        summary.total_configs,
        &database_label,
        &config_path,
    );
    Ok(())
}

pub(super) async fn update(client: &MachineClient, args: &crate::cli::UpdateArgs) -> Result<()> {
    let summary = client.refresh_subscriptions(&args.subs_ref).await?;
    crate::app::commands::update::print_summary(&summary);
    Ok(())
}
