use crate::app::context::AppContext;
use crate::app::{AppError, Result};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use xrat_db::{EventFilter, EventRecord};

pub const MAX_LOG_LINES: u16 = 2000;
const MAX_LOG_BYTES: u64 = 512 * 1024;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogFeed {
    Daemon,
    XrayOut,
    XrayErr,
    SingboxOut,
    SingboxErr,
}

impl LogFeed {
    pub fn label(self) -> &'static str {
        match self {
            Self::Daemon => "daemon",
            Self::XrayOut | Self::XrayErr => "xray",
            Self::SingboxOut | Self::SingboxErr => "singbox",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventLevel {
    Info,
    Warn,
    Error,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogCursor {
    pub session_id: Option<i64>,
    pub offset: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum DiagnosticsRequest {
    Events {
        after: Option<i64>,
        limit: u16,
        level: Option<EventLevel>,
    },
    File {
        feed: LogFeed,
        cursor: Option<LogCursor>,
        lines: u16,
    },
    LatestProbeRun {},
    Engines {},
    Pac {},
    ProbePage {
        selection: xrat_db::ConnectionTestSelection,
        before: Option<xrat_db::ConnectionTestCursor>,
        limit: u16,
    },
    ClearEvents {},
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum DiagnosticsPayload {
    Events(Vec<EventRecord>),
    File {
        lines: Vec<String>,
        cursor: LogCursor,
        truncated: bool,
    },
    LatestProbeRun(Option<xrat_db::ConnectionTestRunRecord>),
    Engines(Vec<xrat_web::types::RuntimeEngine>),
    Pac(String),
    ProbePage(Vec<xrat_db::ConnectionTestRecord>),
    Cleared(u64),
}

#[derive(Clone)]
pub struct DiagnosticsService {
    context: AppContext,
}

impl DiagnosticsService {
    pub fn new(context: AppContext) -> Self {
        Self { context }
    }

    pub async fn latest_probe_run(&self) -> Result<Option<xrat_db::ConnectionTestRunRecord>> {
        match self.dispatch(DiagnosticsRequest::LatestProbeRun {}).await? {
            DiagnosticsPayload::LatestProbeRun(run) => Ok(run),
            _ => Err(invalid("unexpected latest probe run payload")),
        }
    }

    pub async fn probe_history(
        &self,
        selection: xrat_db::ConnectionTestSelection,
        maximum: Option<usize>,
    ) -> Result<Vec<xrat_db::ConnectionTestRecord>> {
        let id = match selection {
            xrat_db::ConnectionTestSelection::Config(id) => id.0,
            xrat_db::ConnectionTestSelection::Run(id) => id,
        };
        if id <= 0 {
            return Err(invalid("probe selection must be positive"));
        }
        if maximum == Some(0) {
            return Ok(Vec::new());
        }
        #[cfg(any(windows, test))]
        if let Some(client) = &self.context.machine_client {
            return client.probe_history(selection, maximum).await;
        }
        let mut rows = match (selection, maximum) {
            (xrat_db::ConnectionTestSelection::Config(id), Some(limit)) => {
                self.context
                    .db
                    .recent_connection_tests(id, i64::try_from(limit).unwrap_or(i64::MAX))
                    .await?
            }
            (xrat_db::ConnectionTestSelection::Config(id), None) => {
                self.context.db.list_connection_tests(id).await?
            }
            (xrat_db::ConnectionTestSelection::Run(id), _) => {
                self.context.db.list_connection_tests_by_run(id).await?
            }
        };
        if let Some(maximum) = maximum {
            rows.truncate(maximum);
        }
        Ok(rows)
    }

    pub async fn dispatch(&self, request: DiagnosticsRequest) -> Result<DiagnosticsPayload> {
        #[cfg(any(windows, test))]
        if let Some(client) = &self.context.machine_client {
            return client.diagnostics(request).await;
        }
        match request {
            DiagnosticsRequest::Pac {} => {
                let rules =
                    super::proxy_pac::PacRules::from_routing(&self.context.app_config.routing);
                let pac = super::proxy_pac::active_pac(&self.context.db, &rules).await?;
                if pac.len() > MAX_LOG_BYTES as usize {
                    return Err(invalid("machine PAC exceeds response limit"));
                }
                Ok(DiagnosticsPayload::Pac(pac))
            }
            DiagnosticsRequest::Events {
                after,
                limit,
                level,
            } => {
                if !(1..=MAX_LOG_LINES).contains(&limit) || after.is_some_and(|id| id < 0) {
                    return Err(invalid(
                        "events require a limit of 1..2000 and a nonnegative cursor",
                    ));
                }
                let filter = EventFilter {
                    limit: i64::from(limit),
                    source: None,
                    levels: match level {
                        None | Some(EventLevel::Info) => None,
                        Some(EventLevel::Warn) => Some(vec!["warn".into(), "error".into()]),
                        Some(EventLevel::Error) => Some(vec!["error".into()]),
                    },
                };
                let events = if let Some(after) = after {
                    self.context.db.events_after(after, &filter).await?
                } else {
                    let mut events = self.context.db.list_events(&filter).await?;
                    events.reverse();
                    events
                };
                Ok(DiagnosticsPayload::Events(events))
            }
            DiagnosticsRequest::File {
                feed,
                cursor,
                lines,
            } => {
                if !(1..=MAX_LOG_LINES).contains(&lines)
                    || cursor
                        .as_ref()
                        .is_some_and(|cursor| cursor.session_id.is_some_and(|id| id <= 0))
                {
                    return Err(invalid("invalid log line limit or session cursor"));
                }
                self.file(feed, cursor, lines).await
            }
            DiagnosticsRequest::LatestProbeRun {} => Ok(DiagnosticsPayload::LatestProbeRun(
                self.context.db.get_latest_connection_test_run().await?,
            )),
            DiagnosticsRequest::Engines {} => Ok(DiagnosticsPayload::Engines(
                super::engine_probe::probe_local_engines(
                    &self.context,
                    &super::engine_probe::ProcessRuntimeEngineProbe,
                )
                .await
                .into_iter()
                .map(|engine| xrat_web::types::RuntimeEngine {
                    name: engine.name.into(),
                    available: engine.available,
                    version: engine.version.filter(|version| version.len() <= 128),
                })
                .collect(),
            )),
            DiagnosticsRequest::ProbePage {
                selection,
                before,
                limit,
            } => {
                let id = match selection {
                    xrat_db::ConnectionTestSelection::Config(id) => id.0,
                    xrat_db::ConnectionTestSelection::Run(id) => id,
                };
                if id <= 0
                    || !(1..=200).contains(&limit)
                    || before.as_ref().is_some_and(|cursor| {
                        cursor.id <= 0
                            || cursor.tested_at.is_empty()
                            || cursor.tested_at.len() > 128
                    })
                {
                    return Err(invalid("invalid probe selection, cursor, or page limit"));
                }
                Ok(DiagnosticsPayload::ProbePage(
                    self.context
                        .db
                        .connection_test_page(&selection, before.as_ref(), i64::from(limit))
                        .await?,
                ))
            }
            DiagnosticsRequest::ClearEvents {} => Ok(DiagnosticsPayload::Cleared(
                self.context.db.clear_events().await?,
            )),
        }
    }

    async fn file(
        &self,
        feed: LogFeed,
        cursor: Option<LogCursor>,
        lines: u16,
    ) -> Result<DiagnosticsPayload> {
        let session_id = if matches!(feed, LogFeed::Daemon) {
            None
        } else {
            self.context
                .db
                .get_latest_runtime_session()
                .await?
                .map(|session| session.id)
        };
        let filename = match feed {
            LogFeed::Daemon => Some("daemon.log".to_string()),
            LogFeed::XrayOut => session_id.map(|id| format!("session-{id}.out.log")),
            LogFeed::XrayErr => session_id.map(|id| format!("session-{id}.err.log")),
            LogFeed::SingboxOut => session_id.map(|id| format!("session-{id}.singbox.out.log")),
            LogFeed::SingboxErr => session_id.map(|id| format!("session-{id}.singbox.err.log")),
        };
        let empty = || DiagnosticsPayload::File {
            lines: Vec::new(),
            cursor: LogCursor {
                session_id,
                offset: 0,
            },
            truncated: false,
        };
        let Some(filename) = filename else {
            return Ok(empty());
        };
        let path = self.context.runtime_paths.runtime_dir.join(filename);
        let mut file = match tokio::fs::File::open(path).await {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(empty()),
            Err(error) => return Err(error.into()),
        };
        let len = file.metadata().await?.len();
        let incremental =
            cursor.filter(|cursor| cursor.session_id == session_id && cursor.offset <= len);
        let start = incremental
            .as_ref()
            .map_or(len.saturating_sub(MAX_LOG_BYTES), |cursor| cursor.offset);
        file.seek(std::io::SeekFrom::Start(start)).await?;
        let mut bytes = Vec::new();
        file.take((len - start).min(MAX_LOG_BYTES))
            .read_to_end(&mut bytes)
            .await?;
        let mut end = start + bytes.len() as u64;
        let mut truncated = incremental.is_none() && start > 0;
        if incremental.is_none() && start > 0 {
            if let Some(newline) = bytes.iter().position(|byte| *byte == b'\n') {
                bytes.drain(..=newline);
            } else {
                bytes.clear();
            }
        }
        // Retain incomplete appended lines for the next read, unless a single line exceeds the byte bound.
        if incremental.is_some() && !bytes.ends_with(b"\n") && !bytes.is_empty() {
            if let Some(newline) = bytes.iter().rposition(|byte| *byte == b'\n') {
                end -= (bytes.len() - newline - 1) as u64;
                bytes.truncate(newline + 1);
            } else if bytes.len() as u64 == MAX_LOG_BYTES {
                truncated = true;
            } else {
                end = start;
                bytes.clear();
            }
        }
        let mut output: Vec<String> = String::from_utf8_lossy(&bytes)
            .lines()
            .map(str::to_owned)
            .collect();
        if output.len() > usize::from(lines) {
            if incremental.is_some() {
                // Advance only through returned lines so a busy stream does not lose entries.
                let consumed = bytes
                    .iter()
                    .enumerate()
                    .filter(|(_, byte)| **byte == b'\n')
                    .nth(usize::from(lines) - 1)
                    .map(|(index, _)| index + 1)
                    .unwrap_or(bytes.len());
                end = start + consumed as u64;
                output.truncate(usize::from(lines));
            } else {
                output.drain(..output.len() - usize::from(lines));
                truncated = true;
            }
        }
        Ok(DiagnosticsPayload::File {
            lines: output,
            cursor: LogCursor {
                session_id,
                offset: end,
            },
            truncated,
        })
    }
}

fn invalid(message: &str) -> AppError {
    AppError::InvalidArgument(message.into())
}

#[cfg(test)]
mod tests;
