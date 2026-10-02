use super::*;
use crate::app::tests::{
    TestAppBuilder,
    fixtures::{test_node, test_source},
};

#[tokio::test]
async fn exports_share_filters_and_keep_subscription_order() {
    let (context, _root) = TestAppBuilder::new("exports").build_with_root().await;
    let nodes = [
        test_node("first.example"),
        test_node("disabled.example"),
        test_node("deleted.example"),
    ];
    context
        .db
        .import_nodes(&test_source(), &nodes)
        .await
        .unwrap();
    let rows = context.db.list_configs(&Default::default()).await.unwrap();
    context
        .db
        .set_config_enabled(rows[1].id, false)
        .await
        .unwrap();
    context.db.delete_config(rows[2].id).await.unwrap();
    let service = context.services().configs;
    for (enabled, expected) in [
        (None, vec![0]),
        (Some(true), vec![0]),
        (Some(false), vec![0, 1]),
    ] {
        let request = ConfigExportRequest {
            enabled,
            ..Default::default()
        };
        let summaries = service.export_summaries(&request).await.unwrap();
        assert_eq!(
            summaries.iter().map(|row| row.id).collect::<Vec<_>>(),
            expected
                .iter()
                .map(|&index| rows[index].id)
                .collect::<Vec<_>>()
        );
        assert_eq!(
            service.export_subscription(&request).await.unwrap(),
            expected
                .iter()
                .map(|&index| nodes[index].raw_config.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
    let absent = ConfigExportRequest {
        protocol: Some("trojan".into()),
        ..Default::default()
    };
    assert!(service.export_summaries(&absent).await.unwrap().is_empty());
    assert_eq!(service.export_subscription(&absent).await.unwrap(), "");
}

#[tokio::test]
async fn both_export_formats_validate_top_without_an_http_adapter() {
    let (context, _root) = TestAppBuilder::new("export-top").build_with_root().await;
    let service = context.services().configs;
    for top in [0, 201, u32::MAX] {
        let request = ConfigExportRequest {
            top: Some(top),
            ..Default::default()
        };
        assert!(matches!(
            service.export_summaries(&request).await,
            Err(crate::app::AppError::InvalidArgument(_))
        ));
        assert!(matches!(
            service.export_subscription(&request).await,
            Err(crate::app::AppError::InvalidArgument(_))
        ));
    }
    for top in [1, 200] {
        let request = ConfigExportRequest {
            top: Some(top),
            ..Default::default()
        };
        assert!(service.export_summaries(&request).await.unwrap().is_empty());
        assert_eq!(service.export_subscription(&request).await.unwrap(), "");
    }
}
