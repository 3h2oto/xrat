use crate::app::{context::AppContext, runtime_service::RuntimeStatusSnapshot};
use crate::server::ServerResult;
use xrat_web::types::DashboardData;

pub async fn load(
    context: &AppContext,
    snapshot: &RuntimeStatusSnapshot,
) -> ServerResult<DashboardData> {
    Ok(crate::app::services::runtime_dashboard::load(context, snapshot).await?)
}
