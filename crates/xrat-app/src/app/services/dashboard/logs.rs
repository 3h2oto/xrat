use crate::app::context::AppContext;
use crate::app::services::diagnostics::{
    DiagnosticsPayload, DiagnosticsRequest, DiagnosticsService, LogFeed,
};
use xrat_db::EventRecord;
use xrat_support::engine_log::{EngineLogRow, ProxyStream};

const EVENT_LIMIT: u16 = 200;
const PROXY_LOG_LIMIT: usize = 200;

#[derive(Debug, Clone, Default)]
pub struct DashboardLogs {
    pub events: Vec<DashboardEvent>,
    pub proxy: Vec<EngineLogRow>,
}

impl DashboardLogs {
    pub async fn load(context: &AppContext) -> crate::app::Result<Self> {
        let (events, proxy) = tokio::join!(load_events(context), load_proxy_logs(context));
        Ok(Self {
            events: events?,
            proxy: proxy?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardEvent {
    pub id: i64,
    pub time: String,
    pub level: String,
    pub source: String,
    pub kind: String,
    pub message: String,
}

impl From<EventRecord> for DashboardEvent {
    fn from(value: EventRecord) -> Self {
        Self {
            id: value.id,
            time: value.created_at,
            level: value.level,
            source: value.source,
            kind: value.kind,
            message: value.message,
        }
    }
}

async fn load_events(context: &AppContext) -> crate::app::Result<Vec<DashboardEvent>> {
    let DiagnosticsPayload::Events(mut events) = DiagnosticsService::new(context.clone())
        .dispatch(DiagnosticsRequest::Events {
            after: None,
            limit: EVENT_LIMIT,
            level: None,
        })
        .await?
    else {
        return Err(unexpected());
    };
    events.reverse();
    Ok(events.into_iter().map(DashboardEvent::from).collect())
}

async fn load_proxy_logs(context: &AppContext) -> crate::app::Result<Vec<EngineLogRow>> {
    let service = DiagnosticsService::new(context.clone());
    let mut rows = Vec::new();
    for (feed, engine, stream) in [
        (LogFeed::XrayOut, "xray", ProxyStream::Stdout),
        (LogFeed::XrayErr, "xray", ProxyStream::Stderr),
        (LogFeed::SingboxOut, "sing-box", ProxyStream::Stdout),
        (LogFeed::SingboxErr, "sing-box", ProxyStream::Stderr),
    ] {
        let DiagnosticsPayload::File { lines, .. } = service
            .dispatch(DiagnosticsRequest::File {
                feed,
                cursor: None,
                lines: PROXY_LOG_LIMIT as u16,
            })
            .await?
        else {
            return Err(unexpected());
        };
        rows.extend(
            lines
                .into_iter()
                .map(|line| EngineLogRow::parse(engine, stream, &line)),
        );
    }
    if rows.len() > PROXY_LOG_LIMIT {
        rows.drain(..rows.len() - PROXY_LOG_LIMIT);
    }
    Ok(rows)
}

fn unexpected() -> crate::app::AppError {
    crate::app::AppError::InvalidArgument("unexpected dashboard diagnostics result".into())
}
