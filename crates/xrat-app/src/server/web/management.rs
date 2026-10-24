use super::{identity::audit, security::*};
use crate::{
    app::{self, services::ConfigListRequest},
    server::{ServerError, ServerResult, ServerState},
};
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::HeaderMap,
};
use xrat_web::types::*;

pub async fn overview(
    State(state): State<ServerState>,
    Extension(principal): Extension<Principal>,
    axum::extract::Query(query): axum::extract::Query<OverviewQuery>,
) -> ServerResult<Json<Overview>> {
    let context = web(&state)?.context.as_ref().ok_or(ServerError::NotFound)?;
    let snapshot = app::runtime_service::RuntimeService::new(context)
        .status()
        .await?;
    let configs = state
        .services
        .configs
        .list(&ConfigListRequest {
            include_deleted: true,
            ..Default::default()
        })
        .await?
        .items
        .into_iter()
        .map(super::rows::config)
        .collect();
    let subscriptions = state
        .services
        .configs
        .subscriptions()
        .await?
        .into_iter()
        .map(super::rows::subscription)
        .collect();
    let app::services::diagnostics::DiagnosticsPayload::Events(mut events) =
        app::services::diagnostics::DiagnosticsService::new(context.clone())
            .dispatch(app::services::diagnostics::DiagnosticsRequest::Events {
                after: None,
                limit: 100,
                level: None,
            })
            .await?
    else {
        return Err(ServerError::InvalidQuery(
            "unexpected event response".into(),
        ));
    };
    events.reverse();
    let events = events
        .into_iter()
        .map(|row| EventRow {
            id: row.id,
            level: row.level,
            message: format!("{} / {}: {}", row.source, row.kind, row.message),
            created_at: row.created_at,
        })
        .collect();
    let _ = principal;
    let mut current = context.clone();
    if query.dashboard && !context.is_machine_client() {
        current.app_config = app::config::load(&context.runtime_paths.config_path)?;
    }
    let dashboard = if query.dashboard {
        Some(super::dashboard::load(&current, &snapshot).await?)
    } else {
        None
    };
    let traffic = super::traffic::sample(&current, &snapshot).await;
    let mut dashboard = dashboard;
    if let Some(meta) = &mut dashboard {
        meta.traffic_history = web(&state)?
            .traffic
            .snapshot(meta.session_id, meta.stats_enabled);
    }
    Ok(Json(Overview {
        dashboard,
        version: env!("CARGO_PKG_VERSION").into(),
        running: snapshot.pid_running,
        traffic,
        active_config: snapshot.active_config.map(|row| row.id.0),
        configs,
        subscriptions,
        events,
        capabilities: vec![
            Capability {
                name: "runtime".into(),
                available: true,
                reason: None,
            },
            Capability {
                name: "host_maintenance".into(),
                available: !context.is_machine_client(),
                reason: context
                    .is_machine_client()
                    .then(|| "Machine maintenance requires a local Windows administrator.".into()),
            },
        ],
    }))
}

#[derive(Default, serde::Deserialize)]
pub struct OverviewQuery {
    #[serde(default)]
    dashboard: bool,
}
pub(super) fn job(row: xrat_db::WebJob, expose_result: bool) -> Job {
    let progress = row
        .result
        .as_ref()
        .and_then(|value| serde_json::from_str::<TestJobProgress>(value).ok());
    let (result, tests) = if !expose_result {
        (None, Vec::new())
    } else if let Some(progress) = progress {
        (Some(progress.message), progress.tests)
    } else {
        (row.result, Vec::new())
    };
    Job {
        id: row.id,
        username: row.username,
        kind: row.kind,
        state: row.state,
        result,
        tests,
        created_at: row.created_at,
    }
}
pub async fn jobs(
    State(state): State<ServerState>,
    Extension(principal): Extension<Principal>,
) -> ServerResult<Json<Vec<Job>>> {
    Ok(Json(
        state
            .db
            .web_jobs()
            .await?
            .into_iter()
            .filter(|row| principal.user.role.is_admin() || row.username == principal.user.username)
            .map(|row| job(row, principal.user.role.can_operate()))
            .collect(),
    ))
}
pub async fn cancel(
    State(state): State<ServerState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<String>,
) -> ServerResult<Json<bool>> {
    require_operator(&principal)?;
    let record = state
        .db
        .web_jobs()
        .await?
        .into_iter()
        .find(|row| row.id == id)
        .ok_or(ServerError::NotFound)?;
    if record.username != principal.user.username && !principal.user.role.is_admin() {
        return Err(ServerError::Forbidden);
    }
    let web = web(&state)?;
    let cancellations = web
        .cancellations
        .lock()
        .map_err(|_| ServerError::Forbidden)?;
    let flag = cancellations.get(&id).ok_or_else(|| {
        ServerError::InvalidQuery("this job is not cancellable or has finished".into())
    })?;
    flag.cancel();
    Ok(Json(true))
}
pub async fn action(
    State(state): State<ServerState>,
    Extension(principal): Extension<Principal>,
    headers: HeaderMap,
    Json(action): Json<Action>,
) -> ServerResult<Json<Job>> {
    require_operator(&principal)?;
    if super::host::protected(&action) {
        super::host::require_unlocked(&state, &principal, &headers)?;
    }
    if matches!(
        action,
        Action::Delete { hard: true, .. }
            | Action::Purge
            | Action::ClearEvents
            | Action::InstallEngine { .. }
            | Action::Tun { .. }
            | Action::Validate
            | Action::Upgrade { .. }
            | Action::GeoIpUpdate
    ) {
        require_admin(&principal)?;
    }
    if let Action::Bulk { ids, .. } = &action
        && (ids.is_empty()
            || ids.len() > 500
            || ids.iter().any(|id| *id <= 0)
            || ids.iter().collect::<std::collections::HashSet<_>>().len() != ids.len())
    {
        return Err(ServerError::InvalidQuery(
            "Select 1–500 unique rows for a bulk operation.".into(),
        ));
    }
    let bulk = matches!(&action, Action::Bulk { .. });
    let web = web(&state)?;
    let context = web.context.clone().ok_or(ServerError::NotFound)?;
    let permit = web
        .jobs
        .clone()
        .try_acquire_owned()
        .map_err(|_| ServerError::RateLimited)?;
    let mut record = xrat_db::WebJob {
        id: uuid::Uuid::new_v4().to_string(),
        username: principal.user.username.clone(),
        kind: action.kind().into(),
        state: "running".into(),
        result: None,
        created_at: now(),
        updated_at: now(),
    };
    state.db.web_save_job(&record).await?;
    let response = job(record.clone(), true);
    let flag = xrat_support::cancel::CancellationFlag::new();
    if matches!(
        action,
        Action::Test { .. } | Action::Scan { .. } | Action::Bulk { .. }
    ) {
        web.cancellations
            .lock()
            .map_err(|_| ServerError::Forbidden)?
            .insert(record.id.clone(), flag.clone());
    }
    audit(
        &state,
        &principal.user.username,
        &format!("job_started:{}:{}", record.id, record.kind),
    )
    .await;
    tokio::spawn(async move {
        let _permit = permit;
        let result =
            app::services::web_management::execute(&context, action, &flag, &record.id).await;
        record.updated_at = now();
        match result {
            Ok(message) => {
                record.state = if flag.is_cancelled() {
                    "cancelled"
                } else {
                    "succeeded"
                }
                .into();
                record.result = Some(message);
            }
            Err(error) => {
                tracing::error!(%error, job_id = record.id, "management job failed");
                record.state = if flag.is_cancelled() && !context.is_machine_client() {
                    "cancelled"
                } else {
                    "failed"
                }
                .into();
                record.result = Some(if bulk { "Bulk operation failed. Some items may have completed; inspect server logs for details." } else { "Operation failed; inspect server logs for details" }.into());
            }
        }
        if let Err(error) = state.db.web_save_job(&record).await {
            tracing::error!(%error, "failed to persist job completion");
        }
        if let Ok(web) = super::security::web(&state)
            && let Ok(mut flags) = web.cancellations.lock()
        {
            flags.remove(&record.id);
        }
        audit(
            &state,
            &record.username,
            &format!("job_{}:{}", record.state, record.id),
        )
        .await;
    });
    Ok(Json(response))
}

pub async fn engine_logs(
    State(state): State<ServerState>,
    Extension(principal): Extension<Principal>,
) -> ServerResult<Json<Vec<(String, String)>>> {
    use app::services::diagnostics::{
        DiagnosticsPayload, DiagnosticsRequest, DiagnosticsService, LogFeed,
    };
    require_operator(&principal)?;
    let context = web(&state)?.context.as_ref().ok_or(ServerError::NotFound)?;
    let service = DiagnosticsService::new(context.clone());
    let mut logs = Vec::new();
    for (feed, suffix) in [
        (LogFeed::XrayOut, "out.log"),
        (LogFeed::XrayErr, "err.log"),
        (LogFeed::SingboxOut, "singbox.out.log"),
        (LogFeed::SingboxErr, "singbox.err.log"),
    ] {
        let DiagnosticsPayload::File { lines, cursor, .. } = service
            .dispatch(DiagnosticsRequest::File {
                feed,
                cursor: None,
                lines: 2000,
            })
            .await?
        else {
            return Err(ServerError::InvalidQuery(
                "unexpected engine log response".into(),
            ));
        };
        if let Some(session) = cursor.session_id
            && !lines.is_empty()
        {
            logs.push((format!("Session {session} · {suffix}"), lines.join("\n")));
        }
    }
    Ok(Json(logs))
}
