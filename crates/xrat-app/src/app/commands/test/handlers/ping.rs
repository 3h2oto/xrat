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
    let shutdown = signal.wait();
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            _ = &mut shutdown => {
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

#[cfg(test)]
mod shutdown_tests {
    use super::*;
    use crate::app::tests::{
        TestAppBuilder,
        fixtures::{test_node_with, test_source},
        signals::FixtureShutdown,
    };
    use clap::Parser;
    use std::sync::atomic::Ordering;

    #[tokio::test]
    async fn injected_shutdown_and_registration_failure_end_ping_loop() {
        for fail in [false, true] {
            let (context, _root) = TestAppBuilder::new("ping-shutdown").build_with_root().await;
            context
                .db
                .import_nodes(
                    &test_source(),
                    &[test_node_with("fixture.invalid", "fixture")],
                )
                .await
                .unwrap();
            let config = context
                .db
                .list_configs(&Default::default())
                .await
                .unwrap()
                .remove(0);
            let cli = crate::cli::Cli::try_parse_from([
                "xrat",
                "test",
                "--skip-icmp",
                "--skip-tcp",
                "--skip-real-delay",
                "--skip-download",
                "--skip-upload",
            ])
            .unwrap();
            let crate::cli::Command::Test(args) = cli.command else {
                panic!("test args expected")
            };
            let settings = resolve_test_settings(
                &TestRunRequest::from(args.as_ref()),
                &context.app_config,
                &context.runtime_paths,
            )
            .unwrap();
            let signal = FixtureShutdown {
                fail,
                ..Default::default()
            };
            let ping = run_ping_loop_with_signal(&args, &context, settings, config.id, &signal);
            tokio::pin!(ping);
            tokio::select! {
                _ = signal.registered.notified() => {},
                result = &mut ping => panic!("ping stopped before signal: {result:?}"),
                _ = tokio::time::sleep(Duration::from_secs(2)) => panic!("shutdown was not registered"),
            }
            signal.shutdown.notify_one();
            tokio::time::timeout(Duration::from_secs(2), ping)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(signal.calls.load(Ordering::SeqCst), 1);
        }
    }
}
