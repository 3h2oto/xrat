use super::*;
use std::io;
use std::os::unix::process::ExitStatusExt;
use std::process::ExitStatus;
use std::sync::atomic::{AtomicUsize, Ordering};
struct FakeChild {
    exited: bool,
    poll_error: bool,
}
impl ChildHandle for FakeChild {
    fn id(&self) -> u32 {
        42
    }
    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        if self.poll_error {
            Err(io::Error::other("inspect failed"))
        } else {
            Ok(self.exited.then(|| ExitStatus::from_raw(7 << 8)))
        }
    }
    fn wait(&mut self) -> io::Result<ExitStatus> {
        Ok(ExitStatus::from_raw(0))
    }
    fn kill(&mut self) -> io::Result<()> {
        self.exited = true;
        Ok(())
    }
}
struct FakeConnector {
    failures: AtomicUsize,
    hang: bool,
}
#[async_trait]
impl TcpConnector for FakeConnector {
    async fn connect(&self, endpoint: &NetworkEndpoint) -> io::Result<()> {
        if self.hang {
            std::future::pending::<()>().await;
        }
        let remaining = self
            .failures
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |count| {
                Some(count.saturating_sub(1))
            })
            .unwrap();
        if remaining > 0 || endpoint.port == 2 {
            Err(io::Error::new(
                io::ErrorKind::ConnectionRefused,
                "not ready",
            ))
        } else {
            Ok(())
        }
    }
}
#[tokio::test(start_paused = true)]
async fn readiness_retries_then_succeeds_with_fake_connector() {
    let waiter = TcpPortWaiter::new(Arc::new(FakeConnector {
        failures: AtomicUsize::new(2),
        hang: false,
    }));
    let mut child = FakeChild {
        exited: false,
        poll_error: false,
    };
    let start = tokio::time::Instant::now();
    waiter
        .wait(
            &mut child,
            ReadinessRequest::single("fixture", 1, Duration::from_secs(1)),
        )
        .await
        .unwrap();
    assert_eq!(start.elapsed(), Duration::from_millis(200));
}
#[tokio::test(start_paused = true)]
async fn stalled_connection_and_partial_readiness_are_bounded() {
    for hang in [true, false] {
        let waiter = TcpPortWaiter::new(Arc::new(FakeConnector {
            failures: AtomicUsize::new(0),
            hang,
        }));
        let mut child = FakeChild {
            exited: false,
            poll_error: false,
        };
        let mut request = ReadinessRequest::single("fixture", 1, Duration::from_millis(250));
        request.endpoints.push(NetworkEndpoint {
            host: "fixture".into(),
            port: 2,
        });
        let start = tokio::time::Instant::now();
        assert!(matches!(
            waiter.wait(&mut child, request).await,
            Err(ReadinessError::Timeout { .. })
        ));
        assert_eq!(start.elapsed(), Duration::from_millis(250));
    }
}
#[tokio::test]
async fn process_exit_and_poll_errors_preserve_caller_policy() {
    let waiter = TcpPortWaiter::new(Arc::new(FakeConnector {
        failures: AtomicUsize::new(0),
        hang: false,
    }));
    let mut child = FakeChild {
        exited: true,
        poll_error: false,
    };
    assert!(matches!(
        waiter
            .wait(
                &mut child,
                ReadinessRequest::single("fixture", 1, Duration::from_secs(1))
            )
            .await,
        Err(ReadinessError::ProcessExited(_))
    ));
    child.exited = false;
    child.poll_error = true;
    assert!(matches!(
        waiter
            .wait(
                &mut child,
                ReadinessRequest::single("fixture", 1, Duration::from_secs(1))
            )
            .await,
        Err(ReadinessError::Io(_))
    ));
    let mut request = ReadinessRequest::single("fixture", 1, Duration::from_secs(1));
    request.child_errors = ChildPollErrorPolicy::Ignore;
    waiter.wait(&mut child, request).await.unwrap();
}
