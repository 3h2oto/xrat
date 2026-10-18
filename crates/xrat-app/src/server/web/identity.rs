use super::security::*;
use crate::server::{ServerError, ServerResult, ServerState};
use axum::{
    Extension, Json,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, header},
    response::{IntoResponse, Response},
};
use xrat_db::{WebSession, WebToken, WebUser};
use xrat_web::types::*;

fn bad(message: impl Into<String>) -> ServerError {
    ServerError::InvalidQuery(message.into())
}
fn session_cookie(token: &str, secure: bool, clear: bool) -> HeaderValue {
    HeaderValue::from_str(&format!(
        "xrat_session={token}; Path=/api/v1; HttpOnly; SameSite=Lax; Max-Age={}{}",
        if clear { 0 } else { 604800 },
        if secure { "; Secure" } else { "" }
    ))
    .expect("generated cookie is ASCII")
}
async fn hashed(web: &WebState, password: String) -> ServerResult<String> {
    let permit = web
        .hashing
        .clone()
        .try_acquire_owned()
        .map_err(|_| ServerError::RateLimited)?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        hash_password(&password)
    })
    .await
    .map_err(|_| bad("password hashing failed"))?
    .map_err(bad)
}
pub async fn login(
    State(state): State<ServerState>,
    headers: HeaderMap,
    Json(request): Json<LoginRequest>,
) -> ServerResult<Response> {
    let web = web(&state)?;
    require_origin(web, &headers)?;
    if request.username.len() > 64 {
        return Err(ServerError::InvalidApiKey);
    }
    web.check_login_rate(&request.username)?;
    let record = user(&state, &request.username).await?;
    let hash = record
        .as_ref()
        .map(|r| &r.password_hash)
        .unwrap_or(&web.admin.password_hash)
        .clone();
    let valid = verify(web, request.password, hash).await?;
    let record = record
        .filter(|r| valid && r.enabled)
        .ok_or(ServerError::InvalidApiKey)?;
    let token = random_token();
    let csrf = digest(&format!("csrf:{token}"));
    state.db.web_prune_sessions(now()).await?;
    if let Some(old) = cookie(&headers) {
        state.db.web_delete_session(&digest(old)).await?;
    }
    state
        .db
        .web_save_session(&WebSession {
            token_hash: digest(&token),
            username: record.username.clone(),
            credential_version: record.credential_version.clone(),
            csrf_hash: digest(&csrf),
            created_at: now(),
            accessed_at: now(),
        })
        .await?;
    audit(&state, &record.username, "login").await;
    let mut response = Json(LoginResponse {
        user: public_user(&record, record.username == web.admin.username)?,
        csrf_token: csrf,
    })
    .into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        session_cookie(&token, web.secure, false),
    );
    Ok(response)
}
pub async fn me(
    State(_state): State<ServerState>,
    Extension(principal): Extension<Principal>,
    headers: HeaderMap,
) -> Json<LoginResponse> {
    Json(LoginResponse {
        user: principal.user,
        csrf_token: cookie(&headers)
            .map(|token| digest(&format!("csrf:{token}")))
            .unwrap_or_default(),
    })
}
pub async fn logout(
    State(state): State<ServerState>,
    Extension(principal): Extension<Principal>,
) -> ServerResult<Response> {
    if let Some(hash) = principal.session_hash {
        state.db.web_delete_session(&hash).await?;
    }
    let mut response = Json(true).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        session_cookie("", web(&state)?.secure, true),
    );
    Ok(response)
}
pub async fn users(
    State(state): State<ServerState>,
    Extension(principal): Extension<Principal>,
) -> ServerResult<Json<Vec<User>>> {
    require_admin(&principal)?;
    let web = web(&state)?;
    let mut users = vec![public_user(&web.admin, true)?];
    for user in state.db.web_users().await? {
        users.push(public_user(&user, false)?);
    }
    Ok(Json(users))
}
pub async fn save_user(
    State(state): State<ServerState>,
    Extension(principal): Extension<Principal>,
    Json(request): Json<SaveUserRequest>,
) -> ServerResult<Json<bool>> {
    require_admin(&principal)?;
    validate_username(&request.username).map_err(bad)?;
    let web = web(&state)?;
    if request.username == web.admin.username {
        return Err(bad("configured administrator is managed in configuration"));
    }
    let existing = user(&state, &request.username).await?;
    let (hash, must_change) = if let Some(password) = request.password {
        (hashed(web, password).await?, true)
    } else {
        let existing = existing.ok_or_else(|| bad("password is required for new users"))?;
        (existing.password_hash, existing.must_change_password)
    };
    let record = WebUser {
        username: request.username.clone(),
        password_hash: hash,
        role: request.role.as_str().into(),
        enabled: request.enabled,
        must_change_password: must_change,
        credential_version: random_token(),
    };
    state.db.web_save_user(&record).await?;
    state.db.web_revoke_user(&record.username).await?;
    audit(
        &state,
        &principal.user.username,
        &format!("user_updated:{}", record.username),
    )
    .await;
    Ok(Json(true))
}
pub async fn password(
    State(state): State<ServerState>,
    Extension(principal): Extension<Principal>,
    Json(request): Json<PasswordRequest>,
) -> ServerResult<Json<bool>> {
    let web = web(&state)?;
    if principal.user.configured {
        return Err(bad(
            "configured administrator password is managed in configuration",
        ));
    }
    web.check_login_rate(&principal.user.username)?;
    let mut record = user(&state, &principal.user.username)
        .await?
        .ok_or(ServerError::InvalidApiKey)?;
    if !verify(web, request.current_password, record.password_hash.clone()).await? {
        return Err(ServerError::InvalidApiKey);
    }
    record.password_hash = hashed(web, request.new_password).await?;
    record.credential_version = random_token();
    record.must_change_password = false;
    state.db.web_save_user(&record).await?;
    state.db.web_revoke_user(&record.username).await?;
    audit(&state, &record.username, "password_changed").await;
    Ok(Json(true))
}
pub async fn tokens(
    State(state): State<ServerState>,
    Extension(principal): Extension<Principal>,
) -> ServerResult<Json<Vec<TokenInfo>>> {
    require_operator(&principal)?;
    Ok(Json(
        state
            .db
            .web_tokens(&principal.user.username)
            .await?
            .into_iter()
            .map(|token| TokenInfo {
                id: token.token_hash,
                name: token.name,
                scope: token.scope,
                created_at: token.created_at,
            })
            .collect(),
    ))
}
pub async fn create_token(
    State(state): State<ServerState>,
    Extension(principal): Extension<Principal>,
    Json(request): Json<TokenRequest>,
) -> ServerResult<Json<CreatedToken>> {
    require_operator(&principal)?;
    if request.name.is_empty() || request.name.len() > 100 {
        return Err(bad("token name must contain 1-100 bytes"));
    }
    if state.db.web_tokens(&principal.user.username).await?.len() >= 32 {
        return Err(bad("maximum 32 tokens per user"));
    }
    let token = random_token();
    state
        .db
        .web_save_token(&WebToken {
            token_hash: digest(&token),
            username: principal.user.username.clone(),
            credential_version: principal.version,
            scope: match request.scope {
                TokenScope::Read => "read",
                TokenScope::Manage => "manage",
            }
            .into(),
            name: request.name,
            created_at: now(),
        })
        .await?;
    audit(&state, &principal.user.username, "token_created").await;
    Ok(Json(CreatedToken { token }))
}
pub async fn delete_token(
    State(state): State<ServerState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<String>,
) -> ServerResult<Json<bool>> {
    require_operator(&principal)?;
    state
        .db
        .web_delete_token(&id, &principal.user.username)
        .await?;
    audit(&state, &principal.user.username, "token_revoked").await;
    Ok(Json(true))
}
pub async fn audit(state: &ServerState, actor: &str, action: &str) {
    if let Err(error) = state.db.web_audit(actor, action).await {
        tracing::warn!(%error, "failed to record management audit event");
    }
}
