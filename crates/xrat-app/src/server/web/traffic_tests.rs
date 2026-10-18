use super::*;

fn point(at: f64, bytes: Option<u64>) -> TrafficSample {
    TrafficSample {
        sampled_at_ms: at,
        traffic: bytes.map(|bytes| Traffic {
            uploaded_bytes: bytes,
            downloaded_bytes: bytes,
        }),
    }
}

#[test]
fn history_retains_five_minutes_and_bounds_sample_count() {
    let mut history = History::default();
    for index in 0..200 {
        history.record(Some(1), point(index as f64 * 3000.0, Some(index)));
    }
    assert_eq!(history.samples.len(), 101);
    assert_eq!(history.samples.first().unwrap().sampled_at_ms, 297_000.0);
    for index in 0..200 {
        history.record(Some(1), point(600_000.0 + index as f64, Some(index)));
    }
    assert_eq!(history.samples.len(), MAX_SAMPLES);
    history.prune(1_000_000.0);
    assert!(history.samples.is_empty());
}

#[test]
fn history_preserves_engine_gaps_and_resets_sessions_and_clock_reversals() {
    let mut history = History::default();
    history.record(Some(1), point(1000.0, Some(100)));
    history.record(Some(1), point(4000.0, None));
    history.record(Some(1), point(7000.0, Some(200)));
    assert_eq!(history.samples.len(), 3);
    assert!(history.samples[1].traffic.is_none());
    history.record(Some(2), point(10000.0, Some(0)));
    assert_eq!(history.samples.len(), 1);
    history.record(Some(2), point(9000.0, Some(1)));
    assert_eq!(history.samples.len(), 1);
    history.record(None, point(12000.0, None));
    assert!(history.samples.is_empty());
}

#[test]
fn snapshot_hides_other_sessions_disabled_stats_and_expired_samples() {
    let collector = Collector::default();
    collector
        .history
        .lock()
        .unwrap()
        .record(Some(1), point(now_ms(), Some(100)));
    assert_eq!(collector.snapshot(Some(1), true).len(), 1);
    assert!(collector.snapshot(Some(2), true).is_empty());
    assert!(collector.snapshot(Some(1), false).is_empty());
    assert!(collector.snapshot(None, true).is_empty());
    collector
        .history
        .lock()
        .unwrap()
        .record(Some(1), point(now_ms() - WINDOW_MS - 1.0, Some(100)));
    assert!(collector.snapshot(Some(1), true).is_empty());
}

#[tokio::test]
async fn collector_samples_without_dashboard_requests_and_stops_when_state_is_dropped() {
    use crate::app::{
        config::{SecretString, ServerSettings},
        tests::TestAppBuilder,
    };
    use std::sync::atomic::{AtomicU64, Ordering};
    let (context, _root) = TestAppBuilder::new("web-traffic-history")
        .build_with_root()
        .await;
    let requests = Arc::new(AtomicU64::new(0));
    let count = requests.clone();
    let engine = axum::Router::new().route(
        "/connections",
        axum::routing::get(move || {
            let index = count.fetch_add(1, Ordering::SeqCst) + 1;
            async move {
                axum::Json(
                    serde_json::json!({"uploadTotal": index * 100, "downloadTotal": index * 1000}),
                )
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let engine_task = tokio::spawn(async move { axum::serve(listener, engine).await.unwrap() });
    std::fs::create_dir_all(context.runtime_paths.config_path.parent().unwrap()).unwrap();
    std::fs::write(&context.runtime_paths.config_path, format!("[runtime]\nengine = 'sing-box'\n[runtime.stats]\nenabled = true\nhost = '127.0.0.1'\nport = {port}\n")).unwrap();
    let session = context
        .db
        .insert_runtime_session(&xrat_db::RuntimeSessionInsert {
            config_id: None,
            status: xrat_db::RuntimeSessionStatus::Running,
            http_host: None,
            http_port: None,
            socks_host: None,
            socks_port: None,
            shadowsocks_host: None,
            shadowsocks_port: None,
            process_id: Some(std::process::id() as i64),
            failure_reason: None,
            started_at: None,
            stopped_at: None,
        })
        .await
        .unwrap();
    let mut settings = ServerSettings::default();
    settings.web.admin_username = "admin".into();
    settings.web.admin_password_hash = Some(SecretString::Literal(
        super::super::hash_password("test password").unwrap(),
    ));
    let mut web = WebState::new(&settings).unwrap();
    web.context = Some(context);
    let web = Arc::new(web);
    start(&web);
    start(&web);
    tokio::time::timeout(Duration::from_secs(8), async {
        loop {
            if web.traffic.snapshot(Some(session), true).len() >= 2 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .unwrap();
    let samples = web.traffic.snapshot(Some(session), true);
    assert_eq!(samples.len(), 2);
    assert_eq!(requests.load(Ordering::SeqCst), 2);
    assert_eq!(samples[0].traffic.as_ref().unwrap().downloaded_bytes, 1000);
    assert_eq!(samples[1].traffic.as_ref().unwrap().downloaded_bytes, 2000);
    let abort = web
        .traffic
        .task
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .abort_handle();
    let weak = Arc::downgrade(&web);
    drop(web);
    tokio::time::timeout(Duration::from_secs(4), async {
        while !abort.is_finished() {
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .unwrap();
    assert!(weak.upgrade().is_none());
    engine_task.abort();
}
