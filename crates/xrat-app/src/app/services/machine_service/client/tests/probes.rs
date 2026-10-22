use super::*;
use crate::app::services::machine_service::{ProbeStages, ProbeState};

async fn probe(client: &MachineClient, ids: Vec<xrat_model::ConfigId>) -> String {
    let job = client
        .start_probe(
            ids,
            ProbeStages {
                skip_icmp: true,
                skip_tcp: true,
                skip_real_delay: true,
                skip_download: true,
                skip_upload: true,
            },
        )
        .await
        .unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            let status = client
                .probe_status(Some(job.job_id.clone()))
                .await
                .unwrap()
                .unwrap();
            if status.state == ProbeState::Completed {
                break;
            }
            assert!(matches!(
                status.state,
                ProbeState::Running | ProbeState::Cancelling
            ));
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    job.job_id
}

#[tokio::test]
async fn machine_probe_results_shrink_pages_and_expire_only_volatile_output() {
    let (context, client, calls, _root) = fixture(None).await;
    for index in 1..=5 {
        import(&client, index).await;
    }
    let ids = context
        .db
        .list_configs(&Default::default())
        .await
        .unwrap()
        .into_iter()
        .map(|row| row.id)
        .collect::<Vec<_>>();
    let id = probe(&client, ids.clone()).await;
    let status = client
        .probe_status(Some(id.clone()))
        .await
        .unwrap()
        .unwrap();
    let rows = client.probe_results(&id).await.unwrap().unwrap();
    assert_eq!(rows.len(), ids.len());
    assert_eq!(
        rows.iter()
            .map(|row| row.id)
            .collect::<std::collections::HashSet<_>>(),
        ids.iter().copied().collect()
    );
    assert!(rows.iter().all(|row| row.status
        == crate::app::services::testing::TestStatus::Skipped
        && row.elapsed_secs.is_finite()));
    assert!(calls.lock().unwrap().iter().any(|call| matches!(
        call.request,
        GatewayRequestKind::ProbeResults { limit: 1..=2, .. }
    )));
    let next = probe(&client, ids).await;
    assert!(client.probe_results(&id).await.unwrap().is_none());
    assert!(client.probe_status(Some(id)).await.unwrap().is_some());
    assert_eq!(
        client
            .probe_history(
                xrat_db::ConnectionTestSelection::Run(status.run_id.unwrap()),
                None
            )
            .await
            .unwrap()
            .len(),
        5
    );
    assert_eq!(client.probe_results(&next).await.unwrap().unwrap().len(), 5);
}

#[tokio::test]
async fn machine_probe_results_return_no_partial_rows_after_later_page_failure() {
    let (context, client, _calls, _root) = fixture(Some(2)).await;
    for index in 1..=5 {
        import(&client, index).await;
    }
    let ids = context
        .db
        .list_configs(&Default::default())
        .await
        .unwrap()
        .into_iter()
        .map(|row| row.id)
        .collect();
    let id = probe(&client, ids).await;
    assert!(
        client
            .probe_results(&id)
            .await
            .unwrap_err()
            .to_string()
            .contains("pipe disconnected")
    );
}
