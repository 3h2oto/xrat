use crate::app::{context::AppContext, read_models::TrafficSnapshot, services};
use tokio::sync::oneshot;

pub(crate) async fn handle_runtime_traffic(
    context: &AppContext,
    respond_to: oneshot::Sender<Result<TrafficSnapshot, String>>,
) {
    let result = services::runtime_traffic::collect(context)
        .await
        .map_err(|error| error.to_string());
    if respond_to.send(result).is_err() {
        tracing::debug!("runtime traffic receiver dropped");
    }
}

pub(crate) async fn handle_runtime_dashboard(
    context: &AppContext,
    respond_to: oneshot::Sender<Result<xrat_web::types::DashboardData, String>>,
) {
    let result = services::runtime_dashboard::collect(context)
        .await
        .map_err(|error| error.to_string());
    if respond_to.send(result).is_err() {
        tracing::debug!("runtime dashboard receiver dropped");
    }
}

#[cfg(test)]
mod tests;
