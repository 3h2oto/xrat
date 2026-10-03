use super::super::*;

pub(super) async fn run_ping_loop(
    args: &TestArgs,
    context: &AppContext,
    settings: ResolvedTestSettings,
    config_id: ConfigId,
) -> crate::app::Result<()> {
    run_ping_loop_with_signal(
        args,
        context,
        settings,
        config_id,
        &xrat_support::signals::CtrlCShutdown,
    )
    .await
}

pub(super) async fn run_ping_loop_with_signal(
    args: &TestArgs,
    context: &AppContext,
    settings: ResolvedTestSettings,
    config_id: ConfigId,
    signal: &dyn xrat_support::signals::ShutdownSignal,
) -> crate::app::Result<()> {
    let config = context
        .db
        .get_config_by_id(config_id)
        .await?
        .ok_or_else(|| AppError::InvalidArgument(format!("config id {config_id} not found")))?;
    let run_id = context
        .db
        .insert_connection_test_run(&ConnectionTestRunInsert {
            kind: "ping_loop".to_string(),
        })
        .await?;
    let interval_ms = args.ping_interval_ms.max(100);
    println!(
        "Starting ping loop for config {} (interval={}ms). Press Ctrl+C to stop.",
        config.r#ref, interval_ms
    );

    let mut ticker = tokio::time::interval(Duration::from_millis(interval_ms));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let started_at = Instant::now();
    let mut total = 0usize;
    let mut ok = 0usize;
    let mut failed = 0usize;

    loop {
        tokio::select! {
            _ = signal.wait() => {
                break;
            }
            _ = ticker.tick() => {
                total += 1;
                let output = test_and_record_config(
                    context.db.clone(),
                    config.clone(),
                    settings.clone(),
                    false,
                    Some(run_id),
                ).await?;
                match output.status {
                    TestStatus::Ok => {
                        ok += 1;
                        println!(
                            "#{total} ok icmp={}ms real_delay={}ms download={}Mbps",
                            optional_number(output.icmp_ms),
                            optional_number(output.real_delay_ms),
                            optional_float(output.download_mbps),
                        );
                    }
                    TestStatus::Failed => {
                        failed += 1;
                        println!("#{total} failed {}", output.error.as_deref().unwrap_or("failed"));
                    }
                    TestStatus::Skipped => {
                        println!("#{total} skipped");
                    }
                }
            }
        }
    }

    let elapsed = started_at.elapsed().as_secs_f64();
    println!(
        "Ping loop stopped: total={}, ok={}, failed={}, elapsed={:.2}s, run_id={}",
        total, ok, failed, elapsed, run_id
    );
    Ok(())
}
