use crate::app::commands::output;
use crate::app::commands::resolve::resolve_subscription_id;
use crate::app::context::AppContext;
use crate::cli::UpdateArgs;
use xrat_db::SubscriptionRecord;
use xrat_model::SubscriptionId;

#[cfg(test)]
use crate::app::services::subscriptions::SubscriptionUpdateResult;
pub use crate::app::services::subscriptions::SubscriptionUpdateSummary;

pub async fn run(context: &AppContext, args: &UpdateArgs) -> crate::app::Result<()> {
    let summary = if args.subs_ref.is_empty() {
        update_all(context).await?
    } else {
        update_by_refs(context, &args.subs_ref).await?
    };

    print_summary(&summary);

    Ok(())
}

pub async fn update_by_ids(
    context: &AppContext,
    ids: &[SubscriptionId],
) -> crate::app::Result<SubscriptionUpdateSummary> {
    let subscriptions = context.services().configs.subscriptions().await?;
    let targets = ids
        .iter()
        .filter_map(|id| subscriptions.iter().find(|row| row.id == *id).cloned())
        .collect();
    update_targets(context, targets).await
}

pub async fn update_all(context: &AppContext) -> crate::app::Result<SubscriptionUpdateSummary> {
    let subscriptions = context.services().configs.subscriptions().await?;
    let targets = subscriptions
        .into_iter()
        .filter(|subscription| !subscription.source_url.as_deref().unwrap_or("").is_empty())
        .collect::<Vec<_>>();
    update_targets(context, targets).await
}

pub async fn update_by_refs(
    context: &AppContext,
    refs: &[String],
) -> crate::app::Result<SubscriptionUpdateSummary> {
    let subscriptions = context.services().configs.subscriptions().await?;
    let mut targets = Vec::with_capacity(refs.len());
    for raw in refs {
        let id = resolve_subscription_id(context, raw).await?;
        let subscription = subscriptions
            .iter()
            .find(|row| row.id == id)
            .cloned()
            .ok_or_else(|| {
                crate::app::AppError::InvalidArgument(format!("no subscription found for '{raw}'"))
            })?;
        targets.push(subscription);
    }
    update_targets(context, targets).await
}

async fn update_targets(
    context: &AppContext,
    targets: Vec<SubscriptionRecord>,
) -> crate::app::Result<SubscriptionUpdateSummary> {
    Ok(
        crate::app::services::subscriptions::refresh_targets(targets, |url, headers| async move {
            crate::app::services::catalog_import::subscription(context, &url, None, &headers).await
        })
        .await,
    )
}

pub(super) fn print_summary(summary: &SubscriptionUpdateSummary) {
    println!(
        "{}",
        output::notice(summary.status_message(), output::color_enabled())
    );
    println!(
        "{}",
        output::format_kv(
            Some("Subscription update"),
            &[
                ("attempted", summary.attempted.to_string()),
                ("succeeded", summary.succeeded.to_string()),
                ("failed", summary.failed.to_string()),
                ("imported configs", summary.imported_configs.to_string()),
                ("removed configs", summary.removed_configs.to_string()),
            ],
            output::color_enabled(),
        )
    );

    if summary.failed > 0 {
        let failed = summary
            .failures
            .iter()
            .map(|entry| {
                format!(
                    "{} ({}) — {}",
                    entry.subscription_name, entry.subscription_ref, entry.error
                )
            })
            .collect::<Vec<_>>();
        println!(
            "{}",
            output::format_list("Failed subscriptions", &failed, output::color_enabled())
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_message_reports_all_updated_for_multi_success() {
        let summary = SubscriptionUpdateSummary {
            attempted: 2,
            succeeded: 2,
            failed: 0,
            imported_configs: 3,
            removed_configs: 1,
            successes: vec![
                SubscriptionUpdateResult {
                    subscription_id: 1.into(),
                    subscription_ref: "aabbccdd".to_string(),
                    subscription_name: "main".to_string(),
                    imported_configs: 2,
                    removed_configs: 1,
                },
                SubscriptionUpdateResult {
                    subscription_id: 2.into(),
                    subscription_ref: "eeff0011".to_string(),
                    subscription_name: "backup".to_string(),
                    imported_configs: 1,
                    removed_configs: 0,
                },
            ],
            failures: Vec::new(),
        };

        assert_eq!(summary.status_message(), "All subscriptions updated!");
    }

    #[test]
    fn status_message_reports_named_single_success() {
        let summary = SubscriptionUpdateSummary {
            attempted: 1,
            succeeded: 1,
            failed: 0,
            imported_configs: 1,
            removed_configs: 0,
            successes: vec![SubscriptionUpdateResult {
                subscription_id: 1.into(),
                subscription_ref: "aabbccdd".to_string(),
                subscription_name: "main".to_string(),
                imported_configs: 1,
                removed_configs: 0,
            }],
            failures: Vec::new(),
        };

        assert_eq!(summary.status_message(), "main updated");
    }

    #[test]
    fn status_message_reports_failures() {
        let summary = SubscriptionUpdateSummary {
            attempted: 4,
            succeeded: 2,
            failed: 2,
            imported_configs: 3,
            removed_configs: 0,
            successes: Vec::new(),
            failures: Vec::new(),
        };
        assert_eq!(summary.status_message(), "2 of 4 subscriptions failed");
    }
}
