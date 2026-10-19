use super::security::*;
use crate::server::{ServerError, ServerResult, ServerState};
use axum::{
    Extension,
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};

pub async fn events(
    State(state): State<ServerState>,
    Extension(principal): Extension<Principal>,
) -> ServerResult<Sse<impl futures_core::Stream<Item = Result<Event, std::convert::Infallible>>>> {
    let permit = web(&state)?
        .streams
        .clone()
        .try_acquire_owned()
        .map_err(|_| ServerError::RateLimited)?;
    let stream = async_stream::stream! {
        let _permit = permit;
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(2));
        loop {
            interval.tick().await;
            if !valid(&state, &principal).await { break; }
            let Ok(rows) = state.db.web_jobs().await else { break; };
            let jobs = rows.into_iter().filter(|row| principal.user.role.is_admin() || row.username == principal.user.username).map(|row| super::management::job(row, principal.user.role.can_operate())).collect::<Vec<_>>();
            if let Ok(event) = Event::default().json_data(jobs) { yield Ok(event); }
        }
    };
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
async fn valid(state: &ServerState, principal: &Principal) -> bool {
    let Ok(Some(user)) = user(state, &principal.user.username).await else {
        return false;
    };
    if !user.enabled || !equal(&user.credential_version, &principal.version) {
        return false;
    }
    if let Some(hash) = &principal.session_hash {
        return state
            .db
            .web_session(hash)
            .await
            .ok()
            .flatten()
            .is_some_and(|session| {
                now() - session.created_at < 604800 && now() - session.accessed_at < 43200
            });
    }
    if let Some(hash) = &principal.token_hash {
        return state.db.web_token(hash).await.ok().flatten().is_some();
    }
    false
}
