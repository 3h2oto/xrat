use super::*;

pub(crate) async fn roundtrip<T>(
    supervisor_tx: mpsc::Sender<SupervisorEvent>,
    build_event: impl FnOnce(oneshot::Sender<T>) -> SupervisorEvent,
) -> crate::app::Result<T> {
    let (tx, rx) = oneshot::channel();
    supervisor_tx.send(build_event(tx)).await.map_err(|_| {
        crate::app::AppError::InvalidArgument("supervisor is not running".to_string())
    })?;
    rx.await.map_err(|_| {
        crate::app::AppError::InvalidArgument("supervisor response channel closed".to_string())
    })
}
