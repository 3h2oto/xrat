mod assets;
mod dashboard;
mod host;
mod identity;
mod management;
mod runtime;
mod security;
mod settings;
mod stream;
pub(super) mod traffic;
mod version;

use crate::server::ServerState;
use axum::{
    Router, middleware,
    routing::{delete, get, post},
};
pub use security::{WebState, hash_password};

pub fn router(state: ServerState) -> Router<ServerState> {
    let protected = Router::new()
        .route("/me", get(identity::me))
        .route("/version", get(version::get))
        .route("/logout", post(identity::logout))
        .route("/host/unlock", post(host::unlock))
        .route("/host/lock", post(host::lock))
        .route("/password", post(identity::password))
        .route("/users", get(identity::users).post(identity::save_user))
        .route(
            "/tokens",
            get(identity::tokens).post(identity::create_token),
        )
        .route("/tokens/{id}", delete(identity::delete_token))
        .route("/settings", get(settings::get).post(settings::save))
        .route("/settings/batch", post(settings::save_batch))
        .route("/export", get(settings::export))
        .route("/configs/{id}/raw", get(settings::raw_config))
        .route("/overview", get(management::overview))
        .route("/overview/runtime", get(runtime::get))
        .route("/logs/engine", get(management::engine_logs))
        .route("/actions", post(management::action))
        .route("/jobs/events", get(stream::events))
        .route("/jobs", get(management::jobs))
        .route("/jobs/{id}/cancel", post(management::cancel))
        .route_layer(middleware::from_fn_with_state(
            state,
            security::authenticate,
        ));
    Router::new()
        .nest("/api/v1", protected.route("/login", post(identity::login)))
        .fallback(assets::serve)
}

#[cfg(test)]
mod tests;

pub(crate) use security::now as security_now;
