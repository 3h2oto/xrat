use super::{identity::audit, security::*};
use crate::{
    app::config::{ConfigEditSession, SettingKind, SettingValue},
    server::{ServerError, ServerResult, ServerState},
};
use axum::{Extension, Json, extract::State};
use xrat_web::types::{Setting, SettingChange, SettingUpdate, Settings, SettingsBatchUpdate};

pub async fn get(
    State(state): State<ServerState>,
    Extension(principal): Extension<Principal>,
) -> ServerResult<Json<Settings>> {
    require_admin(&principal)?;
    let context = web(&state)?.context.as_ref().ok_or(ServerError::NotFound)?;
    let editor = ConfigEditSession::open(&context.runtime_paths.config_path)
        .map_err(ServerError::InvalidQuery)?;
    let items = editor
        .settings
        .iter()
        .filter(|setting| !setting.path.starts_with("server.web."))
        .filter(|setting| context.allows_setting(&setting.path))
        .map(|setting| {
            let (kind, options) = match &setting.kind {
                SettingKind::Secret => ("secret", vec![]),
                SettingKind::Bool => ("bool", vec!["true".into(), "false".into()]),
                SettingKind::Enum(values) => ("enum", values.clone()),
                SettingKind::Integer => ("integer", vec![]),
                SettingKind::List { .. } => ("list", vec![]),
                SettingKind::Text => ("text", vec![]),
            };
            Setting {
                path: setting.path.clone(),
                label: setting.label.clone(),
                value: if matches!(setting.kind, SettingKind::Secret) {
                    String::new()
                } else {
                    setting.value.edit_text()
                },
                kind: kind.into(),
                options,
                effect: setting.effect.label().into(),
            }
        })
        .collect();
    Ok(Json(Settings {
        revision: digest(&editor.original_contents),
        items,
    }))
}
pub async fn save(
    State(state): State<ServerState>,
    Extension(principal): Extension<Principal>,
    Json(request): Json<SettingUpdate>,
) -> ServerResult<Json<String>> {
    save_changes(
        &state,
        &principal,
        request.revision,
        vec![SettingChange {
            path: request.path,
            value: request.value,
        }],
    )
    .await
}

pub async fn save_batch(
    State(state): State<ServerState>,
    Extension(principal): Extension<Principal>,
    Json(request): Json<SettingsBatchUpdate>,
) -> ServerResult<Json<String>> {
    save_changes(&state, &principal, request.revision, request.changes).await
}

async fn save_changes(
    state: &ServerState,
    principal: &Principal,
    revision: String,
    changes: Vec<SettingChange>,
) -> ServerResult<Json<String>> {
    require_admin(principal)?;
    if changes.is_empty() || changes.len() > 256 {
        return Err(ServerError::InvalidQuery(
            "Expected between 1 and 256 setting changes".into(),
        ));
    }
    let mut paths = std::collections::HashSet::new();
    for change in &changes {
        if change.path.starts_with("server.web.") || change.value.len() > 65536 {
            return Err(ServerError::Forbidden);
        }
        if !paths.insert(&change.path) {
            return Err(ServerError::InvalidQuery("Duplicate setting path".into()));
        }
    }
    let context = web(state)?.context.as_ref().ok_or(ServerError::NotFound)?;
    if changes
        .iter()
        .any(|change| !context.allows_setting(&change.path))
    {
        return Err(ServerError::Forbidden);
    }
    let mut editor = ConfigEditSession::open(&context.runtime_paths.config_path)
        .map_err(ServerError::InvalidQuery)?;
    if digest(&editor.original_contents) != revision {
        return Err(ServerError::InvalidQuery(
            "Settings changed. Reload before saving.".into(),
        ));
    }
    for request in changes {
        let setting = editor
            .settings
            .iter_mut()
            .find(|setting| setting.path == request.path)
            .ok_or(ServerError::NotFound)?;
        match &setting.kind {
            SettingKind::Bool => {
                setting.value = SettingValue::Bool(request.value.parse().map_err(|_| {
                    ServerError::InvalidQuery(format!("{}: Expected true or false", request.path))
                })?);
            }
            SettingKind::Enum(options) => {
                if !options.contains(&request.value) {
                    return Err(ServerError::InvalidQuery(format!(
                        "{}: Invalid option",
                        request.path
                    )));
                }
                setting.value = SettingValue::Text(request.value);
            }
            _ => {
                setting.set_from_input(&request.value).map_err(|message| {
                    ServerError::InvalidQuery(format!("{}: {message}", request.path))
                })?;
            }
        }
    }
    let outcome = editor.save().map_err(ServerError::InvalidQuery)?;
    for path in &outcome.changed_paths {
        audit(
            state,
            &principal.user.username,
            &format!("setting_updated:{path}"),
        )
        .await;
    }
    Ok(Json(format!(
        "Saved {} setting(s). {}",
        outcome.changed_paths.len(),
        outcome
            .effects
            .iter()
            .map(|effect| effect.label())
            .collect::<Vec<_>>()
            .join(", ")
    )))
}
pub async fn export(
    State(state): State<ServerState>,
    Extension(principal): Extension<Principal>,
) -> ServerResult<String> {
    require_operator(&principal)?;
    Ok(state
        .services
        .configs
        .export_subscription(&Default::default())
        .await?)
}

pub async fn raw_config(
    State(state): State<ServerState>,
    Extension(principal): Extension<Principal>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> ServerResult<String> {
    require_operator(&principal)?;
    if id <= 0 {
        return Err(ServerError::NotFound);
    }
    state
        .services
        .configs
        .raw_configs_by_ids(&[xrat_model::ConfigId(id)])
        .await?
        .and_then(|mut rows| rows.pop())
        .ok_or(ServerError::NotFound)
}
