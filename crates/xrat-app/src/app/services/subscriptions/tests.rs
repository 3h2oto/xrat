use super::*;
use std::sync::{Arc, Mutex};

fn subscription(id: i64, url: Option<&str>, name: Option<&str>) -> SubscriptionRecord {
    SubscriptionRecord {
        id: id.into(),
        r#ref: format!("ref-{id}"),
        source_kind: "url".into(),
        source_url: url.map(str::to_owned),
        name: name.map(str::to_owned),
        headers: BTreeMap::new(),
        created_at: String::new(),
        updated_at: String::new(),
        config_count: 0,
    }
}

#[tokio::test]
async fn refresh_deduplicates_orders_skips_non_urls_and_keeps_partial_failures() {
    let mut replacement = subscription(1, Some("https://one.example/new"), Some("latest"));
    replacement
        .headers
        .insert("x-key".into(), "latest-header".into());
    let targets = vec![
        subscription(3, Some("https://three.example/fail"), None),
        subscription(1, Some("https://one.example/old"), Some("old")),
        subscription(2, None, None),
        subscription(4, Some(""), None),
        replacement,
    ];
    let calls = Arc::new(Mutex::new(Vec::new()));
    let recorded = calls.clone();
    let summary = refresh_targets(targets, move |url, headers| {
        recorded.lock().unwrap().push((url.clone(), headers));
        async move {
            if url.ends_with("/fail") {
                return Err(crate::app::AppError::InvalidArgument(
                    "fixture failure".into(),
                ));
            }
            Ok(ImportSummary {
                imported_configs: 2,
                removed_configs: 1,
                ..Default::default()
            })
        }
    })
    .await;
    assert_eq!(
        (summary.attempted, summary.succeeded, summary.failed),
        (2, 1, 1)
    );
    assert_eq!((summary.imported_configs, summary.removed_configs), (2, 1));
    assert_eq!(summary.successes[0].subscription_name, "latest");
    assert_eq!(summary.failures[0].subscription_name, "ref-3");
    let calls = calls.lock().unwrap();
    assert_eq!(calls[0].0, "https://one.example/new");
    assert_eq!(calls[0].1["x-key"], "latest-header");
    assert_eq!(calls[1].0, "https://three.example/fail");
}
