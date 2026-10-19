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
        .db
        .list_subscriptions()
        .await?
        .into_iter()
        .map(super::rows::subscription)
        .collect();
    let events = state
        .db
        .list_events(&xrat_db::EventFilter {
            limit: 100,
            ..Default::default()
        })
        .await?
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
    if query.dashboard {
        current.app_config = app::config::load(&context.runtime_paths.config_path)?;
    }
    let dashboard = if query.dashboard {
        Some(super::dashboard::load(&state.db, &current, &snapshot).await?)
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
        capabilities: vec![Capability {
            name: "runtime".into(),
            available: true,
            reason: None,
        }],
    }))
}

#[derive(Default, serde::Deserialize)]
pub struct OverviewQuery {
    #[serde(default)]
    dashboard: bool,
}
fn job(row: xrat_db::WebJob, expose_result: bool) -> Job {
    Job {
        id: row.id,
        username: row.username,
        kind: row.kind,
        state: row.state,
        result: expose_result.then_some(row.result).flatten(),
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
                record.state = if flag.is_cancelled() {
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
    use tokio::io::{AsyncReadExt, AsyncSeekExt};
    require_operator(&principal)?;
    let context = web(&state)?.context.as_ref().ok_or(ServerError::NotFound)?;
    let Some(session) = state.db.get_latest_runtime_session().await? else {
        return Ok(Json(Vec::new()));
    };
    let mut logs = Vec::new();
    for suffix in ["out.log", "err.log", "singbox.out.log", "singbox.err.log"] {
        let path = context
            .runtime_paths
            .runtime_dir
            .join(format!("session-{}.{suffix}", session.id));
        let mut file = match tokio::fs::File::open(path).await {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(crate::app::AppError::from(error).into()),
        };
        let length = file
            .metadata()
            .await
            .map_err(crate::app::AppError::from)?
            .len();
        file.seek(std::io::SeekFrom::Start(length.saturating_sub(65_536)))
            .await
            .map_err(crate::app::AppError::from)?;
        let mut bytes = Vec::new();
        file.take(65_536)
            .read_to_end(&mut bytes)
            .await
            .map_err(crate::app::AppError::from)?;
        logs.push((
            format!("Session {} · {suffix}", session.id),
            String::from_utf8_lossy(&bytes).into_owned(),
        ));
    }
    Ok(Json(logs))
}
