use super::{
    TestActivity, TestActivitySender, TestRunRequest, bulk::BulkProgress, progress::TestRunOutcome,
};
use crate::app::services::machine_service::{MachineClient, ProbeStages, ProbeState};
use crate::app::{AppError, Result};
use std::collections::BTreeMap;
use xrat_model::ConfigId;
use xrat_support::cancel::CancellationReceiver;

pub(super) async fn run(
    request: &TestRunRequest,
    client: &MachineClient,
    ids: &[ConfigId],
    cancel: CancellationReceiver,
    mut progress: BulkProgress,
    activity: Option<TestActivitySender>,
) -> Result<TestRunOutcome> {
    if request.test_url.is_some()
        || request.download_url.is_some()
        || request.upload_url.is_some()
        || request.icmp_timeout_ms.is_some()
        || request.tcp_timeout_ms.is_some()
        || request.real_delay_timeout_ms.is_some()
        || request.download_timeout_ms.is_some()
        || request.upload_timeout_ms.is_some()
        || request.concurrency.is_some()
    {
        return Err(AppError::InvalidArgument("machine probes use administrator-controlled URLs, timeouts, and concurrency; remove overrides".into()));
    }
    if ids.is_empty() || cancel.is_cancelled() {
        return Ok(TestRunOutcome {
            job_id: None,
            local_results: None,
            run_id: None,
            completed: 0,
            total: 0,
        });
    }
    let start = client
        .start_probe(
            ids.to_vec(),
            ProbeStages {
                skip_icmp: request.skip_icmp,
                skip_tcp: request.skip_tcp,
                skip_real_delay: request.skip_real_delay,
                skip_download: request.skip_download,
                skip_upload: request.skip_upload,
            },
        )
        .await?;
    let mut cancellation_sent = false;
    let mut completed = 0;
    let mut previous = BTreeMap::new();
    loop {
        let read = async {
            let snapshot = client
                .probe_status(Some(start.job_id.clone()))
                .await?
                .ok_or_else(|| {
                    AppError::InvalidArgument("machine probe job is no longer available".into())
                })?;
            let rows = if activity.is_some() {
                client.probe_progress(&start.job_id).await?.ok_or_else(|| {
                    AppError::InvalidArgument(
                        "machine probe progress is no longer available".into(),
                    )
                })?
            } else {
                Vec::new()
            };
            Ok::<_, AppError>((snapshot, rows))
        };
        let (snapshot, rows) = tokio::select! {
            _ = cancel.cancelled(), if !cancellation_sent => {
                client.cancel_probe(start.job_id.clone()).await?
                    .ok_or_else(|| AppError::InvalidArgument("machine probe cancellation was not confirmed".into()))?;
                cancellation_sent = true;
                continue;
            }
            result = read => result?,
        };
        if let Some(run_id) = snapshot.run_id
            && let Some(started) = progress.started.take()
        {
            let _ = started.send(run_id);
        }
        if snapshot.completed != completed {
            completed = snapshot.completed;
            if let Some(config_id) = snapshot.last_config_id {
                let _ = progress.updates.send(super::TestProgressUpdate {
                    config_id,
                    done: completed,
                    total: snapshot.total,
                });
            }
        }
        if let Some(activity) = &activity {
            for row in rows {
                if previous.get(&row.config_id) != Some(&row) {
                    previous.insert(row.config_id, row.clone());
                    let _ = activity.send(TestActivity::Snapshot { row });
                }
            }
        }
        match snapshot.state {
            ProbeState::Completed | ProbeState::Cancelled => {
                return Ok(TestRunOutcome {
                    job_id: Some(start.job_id.clone()),
                    local_results: None,
                    run_id: snapshot.run_id,
                    completed: snapshot.completed,
                    total: snapshot.total,
                });
            }
            ProbeState::Failed => {
                return Err(AppError::InvalidArgument(
                    snapshot
                        .error
                        .unwrap_or_else(|| "machine probe failed".into()),
                ));
            }
            ProbeState::Running | ProbeState::Cancelling => {}
        }
        tokio::select! {
            _ = cancel.cancelled(), if !cancellation_sent => {},
            _ = tokio::time::sleep(std::time::Duration::from_millis(200)) => {},
        }
    }
}

pub(crate) async fn run_for_client(
    request: &TestRunRequest,
    client: &MachineClient,
    ids: &[ConfigId],
    cancel: CancellationReceiver,
    updates: tokio::sync::mpsc::UnboundedSender<super::TestProgressUpdate>,
) -> Result<TestRunOutcome> {
    run(
        request,
        client,
        ids,
        cancel,
        BulkProgress {
            updates,
            started: None,
        },
        None,
    )
    .await
}
