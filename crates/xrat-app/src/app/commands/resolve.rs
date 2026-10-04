//! Resolve user-facing identifiers through the shared lifecycle service.

use crate::app::context::AppContext;

pub async fn resolve_config_id(
    context: &AppContext,
    raw: &xrat_model::ConfigRef,
) -> crate::app::Result<xrat_model::ConfigId> {
    context.services().lifecycle.resolve_config_id(raw).await
}

pub async fn resolve_subscription_id(
    context: &AppContext,
    raw: &str,
) -> crate::app::Result<xrat_model::SubscriptionId> {
    context
        .services()
        .lifecycle
        .resolve_subscription_id(raw)
        .await
}

#[cfg(test)]
mod tests;
