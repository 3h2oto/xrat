use async_trait::async_trait;
use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Notify;
use xrat_support::signals::ShutdownSignal;

#[derive(Default)]
pub(crate) struct FixtureShutdown {
    pub(crate) registered: Notify,
    pub(crate) shutdown: Notify,
    pub(crate) calls: AtomicUsize,
    pub(crate) fail: bool,
}
#[async_trait]
impl ShutdownSignal for FixtureShutdown {
    async fn wait(&self) -> io::Result<()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.registered.notify_one();
        self.shutdown.notified().await;
        if self.fail {
            Err(io::Error::other("fixture registration failure"))
        } else {
            Ok(())
        }
    }
}
