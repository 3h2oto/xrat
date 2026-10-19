use crate::{
    app::{config::ServerSettings, context::AppContext},
    server::{ServerError, ServerResult, ServerState},
};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, Method},
    middleware::Next,
    response::Response,
};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use subtle::ConstantTimeEq;
use xrat_db::WebUser;
use xrat_web::types::{Role, User};

pub struct WebState {
    pub admin: WebUser,
    pub origin: String,
    pub secure: bool,
    pub local_origins: Vec<String>,
    pub context: Option<AppContext>,
    pub traffic: super::traffic::Collector,
    pub hashing: Arc<tokio::sync::Semaphore>,
    pub jobs: Arc<tokio::sync::Semaphore>,
    pub streams: Arc<tokio::sync::Semaphore>,
    pub cancellations: Mutex<HashMap<String, xrat_support::cancel::CancellationFlag>>,
    attempts: Mutex<HashMap<String, (i64, u32)>>,
    pub host_unlocks: Mutex<HashMap<String, (String, i64)>>,
}
#[derive(Clone)]
pub struct Principal {
    pub user: User,
    pub version: String,
    pub session_hash: Option<String>,
    pub csrf_hash: Option<String>,
    pub read_only_token: bool,
    pub token_hash: Option<String>,
}
impl WebState {
    pub fn new(settings: &ServerSettings) -> Result<Self, String> {
        let config = &settings.web;
        validate_username(&config.admin_username)?;
        let hash = config
            .admin_password_hash
            .as_ref()
            .ok_or("web admin_password_hash is required")?
            .resolve()
            .map_err(|_| "cannot resolve web administrator password hash")?;
        let parsed = PasswordHash::new(&hash).map_err(|_| "invalid administrator password hash")?;
        if parsed.algorithm.as_str() != "argon2id" || parsed.salt.is_none() || parsed.hash.is_none()
        {
            return Err("administrator password must use Argon2id".into());
        }
        Ok(Self {
            admin: WebUser {
                username: config.admin_username.clone(),
                credential_version: digest(&hash),
                password_hash: hash,
                role: "admin".into(),
                enabled: true,
                must_change_password: false,
            },
            origin: config.origin(&settings.host, settings.port)?,
            secure: config.secure(),
            context: None,
            traffic: Default::default(),
            local_origins: if !config.secure() && config.public_url.is_none() {
                ["localhost", "127.0.0.1", "[::1]"]
                    .into_iter()
                    .map(|host| format!("http://{host}:{}", settings.port))
                    .collect()
            } else {
                vec![]
            },
            hashing: Arc::new(tokio::sync::Semaphore::new(2)),
            jobs: Arc::new(tokio::sync::Semaphore::new(4)),
            streams: Arc::new(tokio::sync::Semaphore::new(64)),
            cancellations: Mutex::new(HashMap::new()),
            attempts: Mutex::new(HashMap::new()),
            host_unlocks: Mutex::new(HashMap::new()),
        })
    }
    pub fn check_login_rate(&self, username: &str) -> ServerResult<()> {
        let mut attempts = self.attempts.lock().map_err(|_| ServerError::RateLimited)?;
        let now = now();
        attempts.retain(|_, (start, _)| now - *start < 60);
        if attempts.len() >= 1024 {
            return Err(ServerError::RateLimited);
        }
        for (key, limit) in [("".to_string(), 30), (username.to_string(), 10)] {
            let count = attempts.entry(key).or_insert((now, 0));
            count.1 += 1;
            if count.1 > limit {
                return Err(ServerError::RateLimited);
            }
        }
        Ok(())
    }
}
pub fn validate_username(username: &str) -> Result<(), String> {
    if !(3..=64).contains(&username.len())
        || !username
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b))
    {
        return Err(
            "username must be 3-64 lowercase ASCII letters, digits, dots, underscores or hyphens"
                .into(),
        );
    }
    Ok(())
}
pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
pub fn random_token() -> String {
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}
pub fn digest(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}
pub fn equal(left: &str, right: &str) -> bool {
    bool::from(left.as_bytes().ct_eq(right.as_bytes()))
}
pub fn hash_password(password: &str) -> Result<String, String> {
    if !(12..=1024).contains(&password.len()) {
        return Err("password must contain 12-1024 bytes".into());
    }
    let salt = SaltString::encode_b64(uuid::Uuid::new_v4().as_bytes())
        .map_err(|_| "cannot generate password salt")?;
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| "cannot hash password".into())
}
pub async fn verify(web: &WebState, password: String, hash: String) -> ServerResult<bool> {
    if password.len() > 1024 {
        return Err(ServerError::InvalidApiKey);
    }
    let permit = web
        .hashing
        .clone()
        .try_acquire_owned()
        .map_err(|_| ServerError::RateLimited)?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        PasswordHash::new(&hash).is_ok_and(|hash| {
            Argon2::default()
                .verify_password(password.as_bytes(), &hash)
                .is_ok()
        })
    })
    .await
    .map_err(|_| ServerError::InvalidApiKey)
}
pub fn web(state: &ServerState) -> ServerResult<&WebState> {
    state.web.as_deref().ok_or(ServerError::NotFound)
}
pub async fn user(state: &ServerState, username: &str) -> ServerResult<Option<WebUser>> {
    let web = web(state)?;
    if username == web.admin.username {
        return Ok(Some(web.admin.clone()));
    }
    Ok(state
        .db
        .web_users()
        .await?
        .into_iter()
        .find(|user| user.username == username))
}
pub fn public_user(user: &WebUser, configured: bool) -> ServerResult<User> {
    Ok(User {
        username: user.username.clone(),
        role: user.role.parse().map_err(|_| ServerError::Forbidden)?,
        enabled: user.enabled,
        must_change_password: user.must_change_password,
        configured,
    })
}
pub fn cookie(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("cookie")?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|cookie| cookie.trim().strip_prefix("xrat_session="))
}
pub fn require_origin(web: &WebState, headers: &HeaderMap) -> ServerResult<()> {
    let origin = headers
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .ok_or(ServerError::Forbidden)?;
    if origin != web.origin && !web.local_origins.iter().any(|allowed| allowed == origin) {
        return Err(ServerError::Forbidden);
    }
    if !web.local_origins.is_empty() {
        let host = headers.get("host").and_then(|value| value.to_str().ok());
        if let Some(host) = host
            && origin != format!("http://{host}")
        {
            return Err(ServerError::Forbidden);
        }
    }
    Ok(())
}
pub fn require_operator(principal: &Principal) -> ServerResult<()> {
    if principal.user.role.can_operate() && !principal.read_only_token {
        Ok(())
    } else {
        Err(ServerError::Forbidden)
    }
}
pub fn require_admin(principal: &Principal) -> ServerResult<()> {
    if principal.user.role == Role::Admin && !principal.read_only_token {
        Ok(())
    } else {
        Err(ServerError::Forbidden)
    }
}
pub(crate) async fn token_principal(state: &ServerState, secret: &str) -> ServerResult<Principal> {
    let token = state
        .db
        .web_token(&digest(secret))
        .await?
        .ok_or(ServerError::InvalidApiKey)?;
    principal_for_token(state, token).await
}

pub(crate) async fn subscription_principal(
    state: &ServerState,
    secret: &str,
) -> ServerResult<Principal> {
    if !secret.starts_with("sub_") {
        return token_principal(state, secret).await;
    }
    let token = state
        .db
        .web_subscription_token(&digest(secret))
        .await?
        .ok_or(ServerError::InvalidApiKey)?;
    principal_for_token(state, token).await
}

async fn principal_for_token(
    state: &ServerState,
    token: xrat_db::WebToken,
) -> ServerResult<Principal> {
    let web = web(state)?;
    let record = user(state, &token.username)
        .await?
        .ok_or(ServerError::InvalidApiKey)?;
    if !record.enabled || !equal(&record.credential_version, &token.credential_version) {
        return Err(ServerError::InvalidApiKey);
    }
    Ok(Principal {
        user: public_user(&record, token.username == web.admin.username)?,
        version: token.credential_version,
        session_hash: None,
        csrf_hash: None,
        read_only_token: token.scope == "read",
        token_hash: Some(token.token_hash),
    })
}

pub async fn authenticate(
    State(state): State<ServerState>,
    mut request: Request,
    next: Next,
) -> ServerResult<Response> {
    let web = web(&state)?;
    let principal = if let Some(token) = request
        .headers()
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
    {
        token_principal(&state, token).await?
    } else {
        let token = cookie(request.headers()).ok_or(ServerError::MissingApiKey)?;
        let hash = digest(token);
        let session = state
            .db
            .web_session(&hash)
            .await?
            .ok_or(ServerError::InvalidApiKey)?;
        if now() - session.created_at >= 604800 || now() - session.accessed_at >= 43200 {
            state.db.web_delete_session(&hash).await?;
            return Err(ServerError::InvalidApiKey);
        }
        if now() - session.accessed_at >= 60 {
            state.db.web_touch_session(&hash, now()).await?;
        }
        let record = user(&state, &session.username)
            .await?
            .ok_or(ServerError::InvalidApiKey)?;
        if !record.enabled || !equal(&record.credential_version, &session.credential_version) {
            return Err(ServerError::InvalidApiKey);
        }
        Principal {
            user: public_user(&record, session.username == web.admin.username)?,
            version: session.credential_version,
            session_hash: Some(hash),
            csrf_hash: Some(session.csrf_hash),
            read_only_token: false,
            token_hash: None,
        }
    };
    let mutating = !matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    );
    if mutating {
        if principal.read_only_token {
            return Err(ServerError::Forbidden);
        }
        if let Some(expected) = &principal.csrf_hash {
            require_origin(web, request.headers())?;
            let csrf = request
                .headers()
                .get("x-csrf-token")
                .and_then(|v| v.to_str().ok())
                .ok_or(ServerError::Forbidden)?;
            if !equal(&digest(csrf), expected) {
                return Err(ServerError::Forbidden);
            }
        }
    }
    let original = request
        .extensions()
        .get::<axum::extract::OriginalUri>()
        .map(|uri| uri.0.path())
        .unwrap_or(request.uri().path());
    if principal.user.must_change_password
        && !matches!(
            original,
            "/api/v1/me" | "/api/v1/password" | "/api/v1/logout"
        )
    {
        return Err(ServerError::Forbidden);
    }
    if matches!(original, "/api/v1/users" | "/api/v1/settings") {
        require_admin(&principal)?;
    }
    if original == "/api/v1/actions"
        || original.starts_with("/api/v1/tokens")
        || original == "/api/v1/export"
        || original.ends_with("/raw")
        || original.ends_with("/cancel")
    {
        require_operator(&principal)?;
    }
    request.extensions_mut().insert(principal);
    Ok(next.run(request).await)
}
