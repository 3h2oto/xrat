use super::{identity::audit, security::*};
use crate::server::{ServerError, ServerResult, ServerState};
use axum::{Extension, Json, extract::State, http::HeaderMap};
use xrat_web::types::{Action, HostUnlock, HostUnlockRequest};

pub fn protected(action: &Action) -> bool {
    matches!(
        action,
        Action::Rotation { .. }
            | Action::InstallEngine { .. }
            | Action::Tun { .. }
            | Action::Purge
            | Action::ClearEvents
            | Action::Validate
            | Action::Upgrade { .. }
            | Action::GeoIpUpdate
    )
}
pub fn require_unlocked(
    state: &ServerState,
    principal: &Principal,
    headers: &HeaderMap,
) -> ServerResult<()> {
    require_admin(principal)?;
    let session = principal
        .session_hash
        .as_ref()
        .ok_or(ServerError::Forbidden)?;
    let token = headers
        .get("x-xrat-host-unlock")
        .and_then(|value| value.to_str().ok())
        .ok_or(ServerError::Forbidden)?;
    let web = web(state)?;
    let mut grants = web
        .host_unlocks
        .lock()
        .map_err(|_| ServerError::Forbidden)?;
    grants.retain(|_, (_, expires)| *expires > now());
    match grants.get(&digest(token)) {
        Some((owner, _)) if equal(owner, session) => Ok(()),
        _ => Err(ServerError::Forbidden),
    }
}
pub async fn unlock(
    State(state): State<ServerState>,
    Extension(principal): Extension<Principal>,
    Json(request): Json<HostUnlockRequest>,
) -> ServerResult<Json<HostUnlock>> {
    require_admin(&principal)?;
    let session = principal
        .session_hash
        .as_ref()
        .ok_or(ServerError::Forbidden)?;
    let web = web(&state)?;
    web.check_login_rate(&principal.user.username)?;
    let record = user(&state, &principal.user.username)
        .await?
        .ok_or(ServerError::Forbidden)?;
    if !verify(web, request.password, record.password_hash).await? {
        return Err(ServerError::Forbidden);
    }
    let token = random_token();
    let expires_at = now() + 300;
    {
        let mut grants = web
            .host_unlocks
            .lock()
            .map_err(|_| ServerError::Forbidden)?;
        grants.retain(|_, (owner, expires)| *expires > now() && !equal(owner, session));
        if grants.len() >= 1024 {
            return Err(ServerError::RateLimited);
        }
        grants.insert(digest(&token), (session.clone(), expires_at));
    }
    audit(&state, &principal.user.username, "host_unlocked").await;
    Ok(Json(HostUnlock { token, expires_at }))
}
pub async fn lock(
    State(state): State<ServerState>,
    Extension(principal): Extension<Principal>,
) -> ServerResult<Json<bool>> {
    require_admin(&principal)?;
    let session = principal
        .session_hash
        .as_ref()
        .ok_or(ServerError::Forbidden)?;
    web(&state)?
        .host_unlocks
        .lock()
        .map_err(|_| ServerError::Forbidden)?
        .retain(|_, (owner, _)| !equal(owner, session));
    audit(&state, &principal.user.username, "host_locked").await;
    Ok(Json(true))
}
