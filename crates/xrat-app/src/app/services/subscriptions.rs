use std::collections::{BTreeMap, HashMap};
use xrat_db::{ImportSummary, SubscriptionRecord};
use xrat_model::SubscriptionId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionUpdateResult {
    pub subscription_id: SubscriptionId,
    pub subscription_ref: String,
    pub subscription_name: String,
    pub imported_configs: usize,
    pub removed_configs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionUpdateError {
    pub subscription_id: SubscriptionId,
    pub subscription_ref: String,
    pub subscription_name: String,
    pub error: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SubscriptionUpdateSummary {
    pub attempted: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub imported_configs: usize,
    pub removed_configs: u64,
    pub successes: Vec<SubscriptionUpdateResult>,
    pub failures: Vec<SubscriptionUpdateError>,
}

impl SubscriptionUpdateSummary {
    pub fn status_message(&self) -> String {
        if self.attempted == 0 {
            return "No subscriptions matched.".to_string();
        }
        if self.failed == 0 {
            if self.succeeded == 1 {
                let item = &self.successes[0];
                return format!("{} updated", item.subscription_name);
            }
            return "All subscriptions updated!".to_string();
        }
        format!("{} of {} subscriptions failed", self.failed, self.attempted)
    }
}

pub(crate) async fn refresh_targets<F, Fut>(
    targets: Vec<SubscriptionRecord>,
    mut refresh: F,
) -> SubscriptionUpdateSummary
where
    F: FnMut(String, BTreeMap<String, String>) -> Fut,
    Fut: std::future::Future<Output = crate::app::Result<ImportSummary>>,
{
    let mut summary = SubscriptionUpdateSummary::default();
    let mut dedup = HashMap::<SubscriptionId, SubscriptionRecord>::new();
    for subscription in targets {
        dedup.insert(subscription.id, subscription);
    }
    let mut ordered = dedup.into_values().collect::<Vec<_>>();
    ordered.sort_by_key(|subscription| subscription.id);

    for subscription in ordered {
        let Some(source_url) = subscription.source_url.as_deref() else {
            continue;
        };
        if source_url.is_empty() {
            continue;
        }
        summary.attempted += 1;
        let label = subscription
            .name
            .as_deref()
            .filter(|value| !value.is_empty())
            .unwrap_or(subscription.r#ref.as_str())
            .to_string();
        match refresh(source_url.to_owned(), subscription.headers.clone()).await {
            Ok(import_summary) => {
                summary.succeeded += 1;
                summary.imported_configs += import_summary.imported_configs;
                summary.removed_configs += import_summary.removed_configs;
                summary.successes.push(SubscriptionUpdateResult {
                    subscription_id: subscription.id,
                    subscription_ref: subscription.r#ref.clone(),
                    subscription_name: label,
                    imported_configs: import_summary.imported_configs,
                    removed_configs: import_summary.removed_configs,
                });
            }
            Err(error) => {
                summary.failed += 1;
                summary.failures.push(SubscriptionUpdateError {
                    subscription_id: subscription.id,
                    subscription_ref: subscription.r#ref.clone(),
                    subscription_name: label,
                    error: error.to_string(),
                });
            }
        }
    }

    summary
}

#[cfg(test)]
mod tests;
