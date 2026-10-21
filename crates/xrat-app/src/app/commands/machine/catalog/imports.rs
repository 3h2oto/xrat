use super::entry::{profile_labels, subscription, unexpected};
use crate::app::commands::{add, import as import_command, progress::CliProgress};
use crate::app::services::machine_service::{CatalogPayload, CatalogRequest, MachineClient};
use crate::app::{AppError, Result, import as import_service};
use crate::cli::ImportArgs;
use std::collections::BTreeMap;

fn body(bytes: &[u8]) -> Result<String> {
    import_service::parse_nodes(bytes)?;
    String::from_utf8(bytes.to_vec()).map_err(|_| {
        AppError::InvalidArgument("subscription contents must be UTF-8 text or base64".into())
    })
}

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
        let (source, contents) = if args.url_list {
            import_service::load_url_list_parts(&args.input, body).await?
        } else {
            let (source, bytes) =
                import_service::load_subscription_input(&args.input, &headers).await?;
            (source, vec![body(&bytes)?])
        };
        client
            .catalog(CatalogRequest::Import {
                contents,
                source,
                name: args.name.clone(),
                headers,
            })
            .await
    }
    .await;
    progress.finish_and_clear();
    let CatalogPayload::Import(summary) = result? else {
        return Err(unexpected());
    };
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
