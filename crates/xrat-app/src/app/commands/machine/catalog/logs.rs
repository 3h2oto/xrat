use crate::app::commands::{logs, output};
use crate::app::services::diagnostics::{
    DiagnosticsPayload, DiagnosticsRequest, EventLevel, LogCursor, LogFeed, MAX_LOG_LINES,
};
use crate::app::services::machine_service::MachineClient;
use crate::app::{AppError, Result};
use crate::cli::{ListFormat, LogLevel, LogSource, LogsArgs, LogsCommand};

pub(super) async fn run(client: &MachineClient, args: &LogsArgs) -> Result<()> {
    if let Some(LogsCommand::Clear(clear)) = &args.command {
        let pending = events(client, None, 1, None).await?;
        if pending.is_empty() {
            println!(
                "{}",
                output::notice("No persisted events to clear.", output::color_enabled())
            );
            return Ok(());
        }
        if !clear.yes && !output::confirm("Permanently delete all persisted app events?")? {
            println!("{}", output::notice("Aborted.", output::color_enabled()));
            return Ok(());
        }
        match client
            .diagnostics(DiagnosticsRequest::ClearEvents {})
            .await?
        {
            DiagnosticsPayload::Cleared(count) => println!(
                "{}",
                output::success(
                    format!("Cleared {count} event(s)."),
                    output::color_enabled()
                )
            ),
            _ => return Err(unexpected()),
        }
        return Ok(());
    }
    if args.lines == 0 || args.lines > usize::from(MAX_LOG_LINES) {
        return Err(AppError::InvalidArgument(
            "machine logs require --lines 1..2000".into(),
        ));
    }
    follow_with_signal(client, args, &xrat_support::signals::CtrlCShutdown).await
}

async fn events(
    client: &MachineClient,
    after: Option<i64>,
    limit: u16,
    level: Option<EventLevel>,
) -> Result<Vec<xrat_db::EventRecord>> {
    match client
        .diagnostics(DiagnosticsRequest::Events {
            after,
            limit,
            level,
        })
        .await?
    {
        DiagnosticsPayload::Events(events) => Ok(events),
        _ => Err(unexpected()),
    }
}

pub(super) async fn follow_with_signal(
    client: &MachineClient,
    args: &LogsArgs,
    signal: &dyn xrat_support::signals::ShutdownSignal,
) -> Result<()> {
    let level = args.level.map(|level| match level {
        LogLevel::Info => EventLevel::Info,
        LogLevel::Warn => EventLevel::Warn,
        LogLevel::Error => EventLevel::Error,
    });
    let with_events = matches!(args.source, LogSource::All | LogSource::App);
    let feeds: &[LogFeed] = match args.source {
        LogSource::All => &[
            LogFeed::Daemon,
            LogFeed::XrayOut,
            LogFeed::XrayErr,
            LogFeed::SingboxOut,
            LogFeed::SingboxErr,
        ],
        LogSource::App => &[],
        LogSource::Daemon => &[LogFeed::Daemon],
        LogSource::Xray => &[LogFeed::XrayOut, LogFeed::XrayErr],
        LogSource::Singbox => &[LogFeed::SingboxOut, LogFeed::SingboxErr],
    };
    let color = output::color_enabled();
    let mut event_cursor = 0;
    let mut cursors: Vec<Option<LogCursor>> = vec![None; feeds.len()];
    let mut initial = true;
    let shutdown = signal.wait();
    tokio::pin!(shutdown);
    loop {
        let read = async {
            if with_events {
                let rows = events(
                    client,
                    (!initial).then_some(event_cursor),
                    args.lines as u16,
                    level,
                )
                .await?;
                event_cursor = rows.last().map_or(event_cursor, |event| event.id);
                if args.follow {
                    for row in rows {
                        println!("{}", logs::format_event_line(&row, color));
                    }
                } else {
                    println!("{}", logs::render_events(&rows, args.format)?);
                }
            }
            if !(with_events
                && !args.follow
                && matches!(args.format, ListFormat::Json | ListFormat::Tsv))
            {
                for (index, feed) in feeds.iter().enumerate() {
                    match client
                        .diagnostics(DiagnosticsRequest::File {
                            feed: *feed,
                            cursor: cursors[index].clone(),
                            lines: args.lines as u16,
                        })
                        .await?
                    {
                        DiagnosticsPayload::File {
                            lines,
                            cursor,
                            truncated,
                        } => {
                            cursors[index] = Some(cursor);
                            if truncated {
                                eprintln!(
                                    "{}: older log bytes omitted by the bounded tail",
                                    feed.label()
                                );
                            }
                            if args.follow {
                                for line in lines {
                                    if let Some(line) =
                                        logs::emit_file_line(feed.label(), &line, color)
                                    {
                                        println!("{line}");
                                    }
                                }
                            } else if !lines.is_empty() {
                                println!("\n{} (machine service)", feed.label());
                                if matches!(feed, LogFeed::Daemon) {
                                    println!("{}", output::format_list("daemon", &lines, color));
                                } else {
                                    println!(
                                        "{}",
                                        logs::format_engine_table(&logs::engine_rows(
                                            feed.label(),
                                            &lines
                                        ))
                                    );
                                }
                            }
                        }
                        _ => return Err(unexpected()),
                    }
                }
            }
            Ok::<(), AppError>(())
        };
        if args.follow {
            tokio::select! { _ = &mut shutdown => return Ok(()), result = read => result? }
        } else {
            return read.await;
        }
        initial = false;
        tokio::select! {
            _ = &mut shutdown => return Ok(()),
            _ = tokio::time::sleep(std::time::Duration::from_millis(500)) => {}
        }
    }
}

fn unexpected() -> AppError {
    AppError::InvalidArgument("machine service returned unexpected diagnostics".into())
}
