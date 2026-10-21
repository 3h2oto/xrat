use super::super::MachineGateway;
use super::*;
use crate::app::context::AppContext;
use crate::app::tests::TestAppBuilder;
use std::collections::BTreeMap;
use std::sync::Mutex;
use xrat_db::{ImportSource, SourceKind};

const OPERATOR: &str = "S-1-5-21-1-2-3-1001";

struct Loopback {
    gateway: MachineGateway,
    calls: Arc<Mutex<Vec<GatewayRequest>>>,
    page_limit: usize,
    fail_at_offset: Option<i64>,
}

#[async_trait::async_trait]
impl Transport for Loopback {
    async fn exchange(&self, request: GatewayRequest) -> Result<DaemonResponse<GatewayPayload>> {
        self.calls.lock().unwrap().push(request.clone());
        let page = match &request.request {
            GatewayRequestKind::Catalog {
                request: CatalogRequest::List { filter } | CatalogRequest::ExportPage { filter },
            } => Some((
                filter.offset.unwrap_or_default(),
                filter.limit.unwrap_or_default() as usize,
            )),
            GatewayRequestKind::Catalog {
                request: CatalogRequest::Subscriptions { offset, limit },
            } => Some((*offset as i64, *limit as usize)),
            _ => None,
        };
        if let Some((offset, limit)) = page {
            if self.fail_at_offset == Some(offset) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "machine pipe disconnected",
                )
                .into());
            }
            if limit > self.page_limit {
                return Ok(DaemonResponse {
                    protocol_version: GATEWAY_VERSION,
                    ok: false,
                    code: DaemonResponseCode::MessageTooLarge,
                    message: "reply too large".into(),
                    payload: None,
                });
            }
        }
        let decoded = serde_json::from_slice(&serde_json::to_vec(&request)?)?;
        let response = self.gateway.dispatch(OPERATOR, decoded).await?;
        Ok(serde_json::from_slice(&serde_json::to_vec(&response)?)?)
    }
}

async fn fixture(
    fail_at_offset: Option<i64>,
) -> (
    AppContext,
    MachineClient,
    Arc<Mutex<Vec<GatewayRequest>>>,
    tempfile::TempDir,
) {
    let (context, root) = TestAppBuilder::new("machine-client")
        .build_with_root()
        .await;
    let (sender, _receiver) = tokio::sync::mpsc::channel(4);
    let calls = Arc::new(Mutex::new(Vec::new()));
    let client = MachineClient {
        transport: Arc::new(Loopback {
            gateway: MachineGateway::for_test(context.clone(), OPERATOR, sender),
            calls: calls.clone(),
            page_limit: 2,
            fail_at_offset,
        }),
    };
    (context, client, calls, root)
}

async fn import(client: &MachineClient, index: u32) {
    let content = format!(
        "vless://00000000-0000-0000-0000-{index:012}@node{index}.example:443?security=tls#node{index}"
    );
    client
        .catalog(CatalogRequest::Import {
            contents: vec![content.clone()],
            source: ImportSource {
                kind: SourceKind::RawText,
                value: content,
                name: None,
            },
            name: None,
            headers: BTreeMap::new(),
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn client_aggregates_smaller_pages_without_losing_filters_or_export_matches() {
    let (context, client, calls, _root) = fixture(None).await;
    for index in 1..=5 {
        import(&client, index).await;
    }
    let expected = context
        .services()
        .configs
        .list(&Default::default())
        .await
        .unwrap();
    let actual = client.list(&Default::default()).await.unwrap();
    assert_eq!(actual.total, 5);
    assert_eq!(actual.items, expected.items);
    let filtered = ConfigListRequest {
        only_enabled: true,
        offset: Some(1),
        limit: Some(2),
        ..Default::default()
    };
    let local = context.services().configs.list(&filtered).await.unwrap();
    let remote = client.list(&filtered).await.unwrap();
    assert_eq!(local.total, remote.total);
    assert_eq!(local.items, remote.items);
    assert_eq!(
        client.export(&filtered).await.unwrap(),
        context
            .services()
            .configs
            .export_raw_configs(&filtered)
            .await
            .unwrap()
    );
    assert_eq!(
        client.subscriptions().await.unwrap(),
        context.services().configs.subscriptions().await.unwrap()
    );
    assert!(calls.lock().unwrap().iter().any(|request| matches!(&request.request,GatewayRequestKind::Catalog {request:CatalogRequest::List {filter}} if filter.limit==Some(1))));
    let empty = client
        .list(&ConfigListRequest {
            only_active: true,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(empty.total, 0);
    assert!(empty.items.is_empty());
}

#[tokio::test]
async fn client_resolves_references_and_preserves_zero_limit_semantics() {
    let (context, client, _calls, _root) = fixture(None).await;
    import(&client, 1).await;
    let record = context
        .db
        .list_configs(&Default::default())
        .await
        .unwrap()
        .remove(0);
    let reference = ConfigRef::from(record.r#ref.as_str());
    assert_eq!(client.resolve_config(&reference).await.unwrap(), record.id);
    assert_eq!(client.config(&reference).await.unwrap(), record);
    let result = client
        .list(&ConfigListRequest {
            offset: Some(0),
            limit: Some(0),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(result.total, 1);
    assert!(result.items.is_empty());
}

#[tokio::test]
async fn disconnected_later_page_returns_an_error_instead_of_partial_success() {
    let (_context, client, _calls, _root) = fixture(Some(1)).await;
    for index in 1..=4 {
        import(&client, index).await;
    }
    let error = client.list(&Default::default()).await.unwrap_err();
    assert!(error.to_string().contains("pipe disconnected"));
    assert!(error.to_string().contains("user profile was not opened"));
}

struct RejectedReply {
    version: u16,
    code: DaemonResponseCode,
    calls: Arc<std::sync::atomic::AtomicUsize>,
}

#[async_trait::async_trait]
impl Transport for RejectedReply {
    async fn exchange(&self, _: GatewayRequest) -> Result<DaemonResponse<GatewayPayload>> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(DaemonResponse {
            protocol_version: self.version,
            ok: false,
            code: self.code.clone(),
            message: "operation completed but reply too large".into(),
            payload: None,
        })
    }
}

#[tokio::test]
async fn mutation_is_not_replayed_after_a_lost_oversized_result_and_versions_are_checked() {
    let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let client = MachineClient {
        transport: Arc::new(RejectedReply {
            version: GATEWAY_VERSION,
            code: DaemonResponseCode::MessageTooLarge,
            calls: calls.clone(),
        }),
    };
    assert!(
        client
            .catalog(CatalogRequest::Delete {
                reference: ConfigRef::from("1"),
                hard: true
            })
            .await
            .is_err()
    );
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    let client = MachineClient {
        transport: Arc::new(RejectedReply {
            version: GATEWAY_VERSION + 1,
            code: DaemonResponseCode::InvalidState,
            calls,
        }),
    };
    assert!(
        client
            .catalog(CatalogRequest::ResolveConfig {
                reference: ConfigRef::from("1")
            })
            .await
            .unwrap_err()
            .to_string()
            .contains("protocol mismatch")
    );
}
