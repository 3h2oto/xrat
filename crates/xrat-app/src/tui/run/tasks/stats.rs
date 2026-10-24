use crate::app::context::AppContext;
use tokio::sync::mpsc;
use xrat_engines::xray::stats::StatsSample;

pub fn spawn_poll_stats(
    context: AppContext,
    session_id: Option<i64>,
    stats_tx: &mpsc::UnboundedSender<(Option<i64>, StatsSample)>,
) {
    let stats_tx = stats_tx.clone();
    tokio::spawn(async move {
        if let Ok(snapshot) = crate::app::services::runtime_traffic::collect(&context).await
            && snapshot.session_id == session_id
            && let Some(traffic) = snapshot.traffic
        {
            let _ = stats_tx.send((
                session_id,
                StatsSample {
                    uplink_total: traffic.uploaded_bytes,
                    downlink_total: traffic.downloaded_bytes,
                },
            ));
        }
    });
}
