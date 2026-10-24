use crate::app::commands::test::{
    print_run_summary, print_single_header, print_single_summary, sort_results, write_results,
};
use crate::app::services::{ConfigListRequest, machine_service::MachineClient, testing};
use crate::app::{AppError, Result};
use crate::cli::TestArgs;
use xrat_model::ConfigRef;
use xrat_support::cancel::{CancellationFlag, CancellationReceiver};
use xrat_support::signals::ShutdownSignal;

pub(super) async fn run(client: &MachineClient, args: &TestArgs) -> Result<()> {
    run_with_signal(client, args, &xrat_support::signals::CtrlCShutdown).await
}

pub(super) async fn run_with_signal(
    client: &MachineClient,
    args: &TestArgs,
    signal: &dyn ShutdownSignal,
) -> Result<()> {
    if args.latest_run_summary {
        if let Some(run) = client.latest_probe_run().await? {
            let rows = client
                .probe_history(xrat_db::ConnectionTestSelection::Run(run.id), None)
                .await?;
            print_run_summary(&run, rows, args);
        } else {
            println!("No persisted test runs found.");
        }
        return Ok(());
    }
    if args.ping {
        return Err(AppError::InvalidArgument("machine test --ping is not supported; use explicit service probe jobs or a user-profile ping loop".into()));
    }
    let config = match &args.id {
        Some(reference) => Some(client.config(&ConfigRef::from(reference.as_str())).await?),
        None => None,
    };
    let subscription_id = match &args.subscription {
        Some(reference) => Some(client.subscription(reference).await?.id),
        None => None,
    };
    let ids = if let Some(config) = &config {
        vec![config.id]
    } else {
        client
            .list(&ConfigListRequest {
                only_enabled: args.enabled_only,
                only_active: args.active_only,
                subscription_id,
                ..Default::default()
            })
            .await?
            .items
            .into_iter()
            .map(|row| row.summary.id)
            .collect()
    };
    let cancel = CancellationFlag::new();
    let (updates, mut received) = tokio::sync::mpsc::unbounded_channel();
    let request = testing::TestRunRequest::from(args);
    let operation = testing::run_machine_for_config_ids(
        &request,
        client,
        &ids,
        CancellationReceiver::from_flag(&cancel),
        updates,
    );
    tokio::pin!(operation);
    let shutdown = signal.wait();
    tokio::pin!(shutdown);
    let mut cancelling = false;
    if let Some(config) = &config {
        print_single_header(config);
    }
    let outcome = loop {
        tokio::select! {
            result = &mut operation => break result?,
            _ = &mut shutdown, if !cancelling => {
                cancel.cancel(); cancelling = true;
            }
            Some(update) = received.recv() => {
                if !args.no_progress { eprintln!("Tested {}/{} configs", update.done, update.total); }
            }
        }
    };
    let mut rows = if let Some(job_id) = outcome.job_id {
        client.probe_results(&job_id).await?.ok_or_else(|| AppError::InvalidArgument("probe output is no longer retained; no results were written. Rerun explicitly if needed".into()))?
    } else {
        Vec::new()
    };
    if rows.len() != outcome.completed {
        return Err(AppError::InvalidArgument(
            "machine probe output count does not match completion".into(),
        ));
    }
    if config.is_some() {
        if let Some(row) = rows.first() {
            print_single_summary(row);
        }
    } else {
        sort_results(&mut rows, args.sort_by);
        write_results(args, &rows)?;
    }
    if cancelling {
        eprintln!(
            "Probe job stopped after cancellation request; {} configs completed",
            outcome.completed
        );
    }
    Ok(())
}
