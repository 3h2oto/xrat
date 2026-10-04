use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use tokio::sync::oneshot;
use tracing::instrument::WithSubscriber;

use super::super::handle_event;
use super::super::test_support::test_context;
use crate::app::daemon::supervisor::{ProxyStatusResult, SupervisorEvent, SupervisorState};

#[derive(Clone, Default)]
struct LogBuffer(Arc<Mutex<Vec<u8>>>);

impl Write for LogBuffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl LogBuffer {
    fn subscriber(&self) -> impl tracing::Subscriber + Send + Sync + 'static {
        let buffer = self.clone();
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_ansi(false)
            .without_time()
            .with_writer(move || buffer.clone())
            .finish()
    }

    fn text(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }
}

#[tokio::test]
async fn dropped_responses_keep_proxy_transitions_and_log_operations() {
    let context = test_context("supervisor-dropped-responses").await;
    let mut state = SupervisorState::new("test".to_string());
    let logs = LogBuffer::default();
    async {
        let (respond_to, receiver) = oneshot::channel();
        drop(receiver);
        handle_event(
            &mut state,
            SupervisorEvent::DaemonPing { respond_to },
            &context,
        )
        .await;
        assert!(state.ready);

        let (respond_to, receiver) = oneshot::channel();
        drop(receiver);
        handle_event(
            &mut state,
            SupervisorEvent::ProxyStart { respond_to },
            &context,
        )
        .await;
        assert!(state.rotation_enabled);
        assert!(state.next_timer_epoch_secs.is_some());

        let (respond_to, receiver) = oneshot::channel();
        drop(receiver);
        handle_event(
            &mut state,
            SupervisorEvent::ProxyStop { respond_to },
            &context,
        )
        .await;
        assert!(!state.rotation_enabled);
        assert!(state.next_timer_epoch_secs.is_none());
    }
    .with_subscriber(logs.subscriber())
    .await;

    let text = logs.text();
    for operation in [
        "daemon_ping_response",
        "proxy_start_response",
        "proxy_stop_response",
    ] {
        assert!(
            text.contains(&format!("operation=\"{operation}\"")),
            "{text}"
        );
    }
    assert!(
        text.contains("error=\"response receiver dropped\""),
        "{text}"
    );
}

#[tokio::test]
async fn failed_optional_query_keeps_proxy_status_fallback_and_logs_error() {
    let context = test_context("supervisor-query-failure").await;
    let pool = sqlx::SqlitePool::connect(&format!(
        "sqlite://{}",
        context.runtime_paths.database_path.display()
    ))
    .await
    .unwrap();
    sqlx::query("DROP TABLE configs")
        .execute(&pool)
        .await
        .unwrap();
    let mut state = SupervisorState::new("test".to_string());
    state.rotation_enabled = true;
    let logs = LogBuffer::default();
    let (respond_to, receiver) = oneshot::channel();
    handle_event(
        &mut state,
        SupervisorEvent::ProxyStatus { respond_to },
        &context,
    )
    .with_subscriber(logs.subscriber())
    .await;

    let ProxyStatusResult::Ok(payload) = receiver.await.unwrap() else {
        panic!("query failure should preserve successful status fallback");
    };
    assert!(payload.active_config_id.is_none());
    assert!(payload.rotation_enabled);
    assert!(state.rotation_enabled);
    let text = logs.text();
    assert!(text.contains("operation=\"proxy_status_config\""), "{text}");
    assert!(text.contains("error="), "{text}");
    assert!(text.contains("no such table"), "{text}");
}
