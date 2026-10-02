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
mod tests {
    use super::*;
    use crate::app::AppError;
    use xrat_db::{ImportSource, SourceKind};
    use xrat_model::{Node, Protocol};

    #[tokio::test]
    async fn resolves_numeric_config_id_and_ref_prefix() {
        let context = test_context("config").await;
        let config = seed_config(&context).await;

        let by_id = resolve_config_id(
            &context,
            &xrat_model::ConfigRef::from(config.id.to_string()),
        )
        .await
        .expect("numeric id should resolve");
        let by_ref = resolve_config_id(&context, &xrat_model::ConfigRef::from(&config.r#ref[..8]))
            .await
            .expect("ref prefix should resolve");

        assert_eq!(by_id, config.id);
        assert_eq!(by_ref, config.id);
    }

    #[tokio::test]
    async fn digit_only_config_ref_resolves_through_read_and_lifecycle_services() {
        let context = test_context("digit-ref").await;
        let config = seed_config(&context).await;
        let pool = sqlx::SqlitePool::connect(&format!(
            "sqlite://{}",
            context.runtime_paths.database_path.display()
        ))
        .await
        .unwrap();
        sqlx::query("UPDATE configs SET ref = '12345678abcd' WHERE id = ?")
            .bind(config.id)
            .execute(&pool)
            .await
            .unwrap();
        let services = context.services();
        assert_eq!(
            services
                .configs
                .resolve_id(&xrat_model::ConfigRef::from("12345678"))
                .await
                .unwrap(),
            Some(config.id)
        );
        assert_eq!(
            resolve_config_id(&context, &xrat_model::ConfigRef::from("12345678"))
                .await
                .unwrap(),
            config.id
        );
        assert_eq!(
            services
                .configs
                .resolve_id(&xrat_model::ConfigRef::from("87654321"))
                .await
                .unwrap(),
            None
        );
        assert_eq!(
            services
                .configs
                .resolve_id(&xrat_model::ConfigRef::from(config.id.to_string()))
                .await
                .unwrap(),
            Some(config.id)
        );
    }

    #[tokio::test]
    async fn resolves_numeric_subscription_id_and_ref_prefix() {
        let context = test_context("subscription").await;
        let subscription = seed_subscription(&context).await;

        let by_id = resolve_subscription_id(&context, &subscription.id.to_string())
            .await
            .expect("numeric id should resolve");
        let by_ref = resolve_subscription_id(&context, &subscription.r#ref[..8])
            .await
            .expect("ref prefix should resolve");

        assert_eq!(by_id, subscription.id);
        assert_eq!(by_ref, subscription.id);
    }

    #[tokio::test]
    async fn missing_identifier_returns_invalid_argument() {
        let context = test_context("missing").await;
        let err = resolve_config_id(&context, &xrat_model::ConfigRef::from("missing"))
            .await
            .expect_err("missing config should error");

        assert!(matches!(err, AppError::InvalidArgument(_)));
    }

    async fn seed_config(context: &AppContext) -> xrat_db::ConfigRecord {
        seed_subscription(context).await;
        context
            .db
            .list_configs(&Default::default())
            .await
            .expect("configs should load")
            .into_iter()
            .next()
            .expect("config should exist")
    }

    async fn seed_subscription(context: &AppContext) -> xrat_db::SubscriptionRecord {
        context
            .db
            .import_nodes(
                &ImportSource {
                    kind: SourceKind::RawText,
                    value: "seed".to_string(),
                    name: Some("seed".to_string()),
                },
                &[Node {
                    protocol: Protocol::Vless,
                    address: "example.com".to_string(),
                    port: 443,
                    username: None,
                    uuid: Some("00000000-0000-0000-0000-000000000001".to_string()),
                    password: None,
                    method: None,
                    network: "tcp".to_string(),
                    tls: Some("tls".to_string()),
                    sni: Some("example.com".to_string()),
                    host: None,
                    path: None,
                    name: Some("seed".to_string()),
                    extensions: None,
                    raw_config: "vless://00000000-0000-0000-0000-000000000001@example.com:443#seed"
                        .to_string(),
                }],
            )
            .await
            .expect("import should succeed");

        context
            .db
            .list_subscriptions()
            .await
            .expect("subscriptions should load")
            .into_iter()
            .next()
            .expect("subscription should exist")
    }

    async fn test_context(prefix: &str) -> AppContext {
        crate::app::tests::TestAppBuilder::new(prefix).build().await
    }
}
