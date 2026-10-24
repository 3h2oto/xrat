use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

struct Worker(Arc<AtomicBool>);

impl Drop for Worker {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

#[tokio::test]
async fn cancellation_waits_until_probe_worker_resources_are_released() {
    let released = Arc::new(AtomicBool::new(false));
    let worker = Worker(released.clone());
    let (entered, started) = tokio::sync::oneshot::channel();
    let mut workers = JoinSet::new();
    workers.spawn(async move {
        let _worker = worker;
        entered.send(()).unwrap();
        std::future::pending::<crate::app::Result<TestOutputRow>>().await
    });
    started.await.unwrap();
    assert!(!released.load(Ordering::SeqCst));
    stop_workers(&mut workers).await;
    assert!(released.load(Ordering::SeqCst));
    assert!(workers.is_empty());
}
