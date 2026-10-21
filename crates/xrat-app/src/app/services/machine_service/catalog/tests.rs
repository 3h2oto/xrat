use super::*;
use crate::app::context::AppContext;
use crate::app::services::{ConfigListRequest, ToggleOutcome};
use crate::app::tests::TestAppBuilder;
use std::collections::BTreeMap;
use xrat_db::{ImportSource, SourceKind};
use xrat_model::ConfigRef;

const FIRST: &str =
    "vless://00000000-0000-0000-0000-000000000001@one.example:443?security=tls#first";
const SECOND: &str =
    "vless://00000000-0000-0000-0000-000000000002@two.example:443?security=tls#second";

async fn fixture() -> (AppContext, MachineCatalog, tempfile::TempDir) {
    let (context, root) = TestAppBuilder::new("machine-catalog")
        .build_with_root()
        .await;
    (context.clone(), MachineCatalog::new(context), root)
}

fn import_request(content: &str) -> CatalogRequest {
    CatalogRequest::Import {
        contents: vec![content.into()],
        source: ImportSource {
            kind: SourceKind::Url,
            value: "https://never-fetch.invalid/subscription".into(),
            name: None,
        },
        name: Some("catalog".into()),
        headers: BTreeMap::new(),
    }
}

fn page(offset: i64, limit: i64) -> ConfigListRequest {
    ConfigListRequest {
        offset: Some(offset),
        limit: Some(limit),
        ..Default::default()
    }
}

#[tokio::test]
async fn imported_catalog_uses_shared_reconciliation_without_fetching_source_metadata() {
    let (context, catalog, _root) = fixture().await;
    let content = format!("{FIRST}\n{SECOND}");
    let CatalogPayload::Import(first) = catalog.dispatch(import_request(&content)).await.unwrap()
    else {
        panic!("expected import");
    };
    assert_eq!(first.total_configs, 2);
    let CatalogPayload::Import(repeated) =
        catalog.dispatch(import_request(&content)).await.unwrap()
    else {
        panic!("expected import");
    };
    assert_eq!(repeated.total_configs, 2);
    let CatalogPayload::Import(reconciled) = catalog.dispatch(import_request(FIRST)).await.unwrap()
    else {
        panic!("expected import");
    };
    assert_eq!(reconciled.total_configs, 1);
    assert_eq!(reconciled.removed_configs, 1);
    assert_eq!(
        context
            .db
            .list_configs(&Default::default())
            .await
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn source_file_metadata_is_never_opened_by_the_privileged_catalog() {
    let (_context, catalog, root) = fixture().await;
    let path = root.path().join("not-an-import.txt");
    std::fs::write(&path, r#"{"outbounds":[{"protocol":"freedom"}]}"#).unwrap();
    let request = CatalogRequest::Import {
        contents: vec![FIRST.into()],
        source: ImportSource {
            kind: SourceKind::File,
            value: path.display().to_string(),
            name: None,
        },
        name: None,
        headers: BTreeMap::new(),
    };
    let CatalogPayload::Import(summary) = catalog.dispatch(request).await.unwrap() else {
        panic!("expected import");
    };
    assert_eq!(summary.total_configs, 1);
}

#[tokio::test]
async fn unsafe_extensions_and_root_json_fail_before_any_catalog_write() {
    let (context, catalog, _root) = fixture().await;
    for content in [
        r#"{"outbounds":[{"protocol":"freedom"}]}"#.to_string(),
        format!(
            "{FIRST}\nvless://id@bad.example:443?type=xhttp&extra=%7B%22downloadSettings%22%3A%7B%22address%22%3A%22file%3A%2F%2F%2Fsecret%22%7D%7D"
        ),
        "vless://id@bad.example:443?fm=%7B%22type%22%3A%22file%22%7D".into(),
    ] {
        assert!(catalog.dispatch(import_request(&content)).await.is_err());
        assert!(context.db.list_subscriptions().await.unwrap().is_empty());
        assert!(
            context
                .db
                .list_configs(&Default::default())
                .await
                .unwrap()
                .is_empty()
        );
    }
}

#[tokio::test]
async fn filtering_pagination_lifecycle_and_export_preserve_shared_results() {
    let (context, catalog, _root) = fixture().await;
    catalog
        .dispatch(import_request(&format!("{FIRST}\n{SECOND}")))
        .await
        .unwrap();
    let records = context.db.list_configs(&Default::default()).await.unwrap();
    let reference = ConfigRef::from(records[0].r#ref.as_str());
    let CatalogPayload::Toggle { outcome, .. } = catalog
        .dispatch(CatalogRequest::Disable {
            reference: reference.clone(),
        })
        .await
        .unwrap()
    else {
        panic!("expected toggle");
    };
    assert_eq!(outcome, ToggleOutcome::Changed);
    let CatalogPayload::Toggle { outcome, .. } = catalog
        .dispatch(CatalogRequest::Disable {
            reference: reference.clone(),
        })
        .await
        .unwrap()
    else {
        panic!("expected toggle");
    };
    assert_eq!(outcome, ToggleOutcome::AlreadyDisabled);
    let filter = ConfigListRequest {
        only_enabled: true,
        ..page(0, 1)
    };
    let wire = serde_json::to_vec(&CatalogRequest::List {
        filter: filter.clone(),
    })
    .unwrap();
    let CatalogPayload::Configs(actual) = catalog
        .dispatch(serde_json::from_slice(&wire).unwrap())
        .await
        .unwrap()
    else {
        panic!("expected list");
    };
    let expected = context.services().configs.list(&filter).await.unwrap();
    assert_eq!(actual.total, 1);
    assert_eq!(actual.items, expected.items);
    let CatalogPayload::ExportPage { total, items } = catalog
        .dispatch(CatalogRequest::ExportPage { filter })
        .await
        .unwrap()
    else {
        panic!("expected export");
    };
    assert_eq!(total, 1);
    assert_eq!(items.len(), 1);
    let enabled = records
        .iter()
        .find(|record| record.r#ref != reference.as_ref())
        .unwrap();
    assert_eq!(items.as_slice(), std::slice::from_ref(&enabled.raw_config));
    catalog
        .dispatch(CatalogRequest::Delete {
            reference: reference.clone(),
            hard: false,
        })
        .await
        .unwrap();
    catalog
        .dispatch(CatalogRequest::Enable {
            reference: reference.clone(),
        })
        .await
        .unwrap();
    assert!(
        context
            .services()
            .lifecycle
            .config(&reference)
            .await
            .unwrap()
            .is_deleted
    );
    catalog
        .dispatch(CatalogRequest::Restore { reference })
        .await
        .unwrap();
    let CatalogPayload::Configs(empty) = catalog
        .dispatch(CatalogRequest::List {
            filter: page(100, 1),
        })
        .await
        .unwrap()
    else {
        panic!("expected list");
    };
    assert_eq!(empty.total, 2);
    assert!(empty.items.is_empty());
}

#[tokio::test]
async fn invalid_batch_and_page_inputs_do_not_mutate_the_catalog() {
    let (context, catalog, _root) = fixture().await;
    for request in [
        CatalogRequest::List {
            filter: ConfigListRequest::default(),
        },
        CatalogRequest::List {
            filter: page(-1, 1),
        },
        CatalogRequest::List {
            filter: page(0, 201),
        },
        CatalogRequest::DeleteMany {
            ids: vec![(-1).into()],
            hard: true,
        },
        CatalogRequest::RestoreMany {
            ids: vec![1.into(); 10_001],
        },
    ] {
        assert!(catalog.dispatch(request).await.is_err());
    }
    assert!(context.db.list_subscriptions().await.unwrap().is_empty());
}

#[test]
fn catalog_wire_does_not_accept_privileged_paths_sql_or_unknown_fields() {
    for value in [
        r#"{"type":"list","filter":{"sql":"SELECT *"}}"#,
        r#"{"type":"import","contents":["nodes"],"source":{"kind":"raw_text","value":"nodes","name":null,"path":"secret"},"name":null,"headers":{}}"#,
        r#"{"type":"import","contents":["nodes"],"source":{"kind":"raw_text","value":"nodes","name":null},"name":null,"headers":{},"executable":"cmd.exe"}"#,
    ] {
        assert!(serde_json::from_str::<CatalogRequest>(value).is_err());
    }
}

#[tokio::test]
async fn multipart_import_decodes_each_body_and_validates_all_before_persistence() {
    use base64::Engine;
    let (context, catalog, _root) = fixture().await;
    let mut request = import_request(FIRST);
    let CatalogRequest::Import {
        contents, headers, ..
    } = &mut request
    else {
        unreachable!()
    };
    contents.push(base64::engine::general_purpose::STANDARD.encode(SECOND));
    headers.insert("x-subscription-key".into(), "fixture".into());
    let CatalogPayload::Import(summary) = catalog.dispatch(request).await.unwrap() else {
        panic!("expected import")
    };
    assert_eq!(summary.total_configs, 2);
    let mut request = import_request(FIRST);
    let CatalogRequest::Import { contents, .. } = &mut request else {
        unreachable!()
    };
    contents.push(r#"{"outbounds":[{"protocol":"freedom"}]}"#.into());
    assert!(catalog.dispatch(request).await.is_err());
    assert_eq!(
        context
            .db
            .list_configs(&Default::default())
            .await
            .unwrap()
            .len(),
        2
    );
}

#[tokio::test]
async fn display_names_preserve_shared_empty_and_whitespace_handling() {
    for name in ["", "  "] {
        let (context, catalog, _root) = fixture().await;
        let (local, _local_root) = TestAppBuilder::new("catalog-name-parity")
            .build_with_root()
            .await;
        let source = ImportSource {
            kind: SourceKind::RawText,
            value: FIRST.into(),
            name: None,
        };
        crate::app::import::persist_nodes(
            &local.db,
            source.clone(),
            &crate::app::import::parse_nodes(FIRST.as_bytes()).unwrap(),
            Some(name),
            None,
        )
        .await
        .unwrap();
        catalog
            .dispatch(CatalogRequest::Import {
                contents: vec![FIRST.into()],
                source,
                name: Some(name.into()),
                headers: BTreeMap::new(),
            })
            .await
            .unwrap();
        let remote = context.db.list_subscriptions().await.unwrap().remove(0);
        let expected = local.db.list_subscriptions().await.unwrap().remove(0);
        assert_eq!(remote.name, expected.name);
        catalog
            .dispatch(CatalogRequest::RenameSubscription {
                reference: remote.id.to_string(),
                name: name.into(),
            })
            .await
            .unwrap();
        local
            .services()
            .lifecycle
            .rename_subscription(&expected.id.to_string(), name)
            .await
            .unwrap();
        assert_eq!(
            context.db.list_subscriptions().await.unwrap()[0].name,
            local.db.list_subscriptions().await.unwrap()[0].name
        );
    }
}
