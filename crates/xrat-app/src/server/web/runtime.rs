use super::security::{Principal, web};
use crate::{
    app::{
        self,
        runtime_service::{RuntimeEndpointState, RuntimeService},
    },
    server::{ServerError, ServerResult, ServerState},
};
use axum::{Extension, Json, extract::State};
use std::{
    sync::LazyLock,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use xrat_web::types::{RuntimeDetails, RuntimeEngine, RuntimeInbound};

pub async fn get(
    State(state): State<ServerState>,
    Extension(_principal): Extension<Principal>,
) -> ServerResult<Json<RuntimeDetails>> {
    static SLOTS: LazyLock<tokio::sync::Semaphore> =
        LazyLock::new(|| tokio::sync::Semaphore::new(2));
    let _permit = SLOTS.try_acquire().map_err(|_| ServerError::RateLimited)?;
    let context = web(&state)?.context.as_ref().ok_or(ServerError::NotFound)?;
    let mut current = context.clone();
    current.app_config = app::config::load(&context.runtime_paths.config_path)?;
    let snapshot = RuntimeService::new(&current).status().await?;
    let socket = app::daemon::ipc::default_socket_path(&context.runtime_paths.runtime_dir);
    let (engines, daemon) = tokio::join!(
        app::services::engine_probe::probe_engines(
            &current,
            &app::services::engine_probe::ProcessRuntimeEngineProbe
        ),
        tokio::time::timeout(
            Duration::from_millis(500),
            app::daemon::ipc::proxy_status_daemon(&socket)
        )
    );
    let daemon = daemon
        .ok()
        .and_then(Result::ok)
        .filter(|response| response.ok)
        .and_then(|response| response.payload);
    let daemon_running = daemon
        .as_ref()
        .map(|value| value.daemon_ready)
        .or_else(|| (!socket.exists()).then_some(false));
    let local_address = xrat_support::net::primary_local_ip();
    let inbounds = [
        ("SOCKS", snapshot.inbound_health.socks.as_ref()),
        ("HTTP", snapshot.inbound_health.http.as_ref()),
        ("Shadowsocks", snapshot.inbound_health.shadowsocks.as_ref()),
    ]
    .into_iter()
    .filter_map(|(name, inbound)| {
        inbound.map(|inbound| RuntimeInbound {
            name: name.into(),
            host: listener_host(&inbound.endpoint.host, local_address.as_deref()),
            port: inbound.endpoint.port,
            reachable: match inbound.state {
                RuntimeEndpointState::Reachable => Some(true),
                RuntimeEndpointState::Unreachable => Some(false),
                RuntimeEndpointState::NotChecked => None,
            },
        })
    })
    .collect();
    let jobs = state.db.web_jobs().await?;
    let rotation = &current.app_config.runtime.rotation;
    Ok(Json(RuntimeDetails {
        sampled_at_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
            * 1000.0,
        status: snapshot.status.as_str().into(),
        engine: if snapshot
            .active_config
            .as_ref()
            .is_some_and(|config| config.protocol == "hysteria2")
        {
            "sing-box".into()
        } else {
            current.app_config.runtime.engine.clone()
        },
        process_id: snapshot
            .session
            .as_ref()
            .filter(|_| snapshot.pid_running)
            .and_then(|session| session.process_id),
        started_at: snapshot
            .session
            .as_ref()
            .filter(|_| snapshot.pid_running)
            .and_then(|session| session.started_at.clone()),
        inbounds,
        engines: engines
            .into_iter()
            .map(|engine| RuntimeEngine {
                name: engine.name.into(),
                available: engine.available,
                version: engine.version,
            })
            .collect(),
        daemon_running,
        rotation_enabled: daemon
            .as_ref()
            .map(|value| value.rotation_enabled)
            .unwrap_or(rotation.enabled),
        rotation_interval_secs: daemon
            .as_ref()
            .map(|value| value.interval_secs)
            .unwrap_or(rotation.interval_secs),
        running_jobs: jobs.iter().filter(|job| job.state == "running").count(),
        queued_jobs: jobs.iter().filter(|job| job.state == "queued").count(),
    }))
}

fn listener_host(host: &str, local: Option<&str>) -> String {
    use std::net::IpAddr;
    match host {
        "0.0.0.0" | "::" => local
            .and_then(|value| value.parse::<IpAddr>().ok())
            .filter(|ip| !ip.is_unspecified() && !ip.is_loopback())
            .filter(|ip| ip.is_ipv4() == (host == "0.0.0.0"))
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| xrat_support::net::connect_host_for_bind_host(host)),
        _ => host.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::listener_host;

    #[test]
    fn wildcard_listeners_use_lan_addresses_with_safe_fallbacks() {
        assert_eq!(
            listener_host("0.0.0.0", Some("192.168.1.168")),
            "192.168.1.168"
        );
        assert_eq!(listener_host("0.0.0.0", None), "127.0.0.1");
        assert_eq!(listener_host("0.0.0.0", Some("0.0.0.0")), "127.0.0.1");
        assert_eq!(listener_host("::", Some("fd00::10")), "fd00::10");
        assert_eq!(listener_host("::", Some("192.168.1.168")), "::1");
        assert_eq!(
            listener_host("127.0.0.1", Some("192.168.1.168")),
            "127.0.0.1"
        );
        assert_eq!(
            listener_host("192.168.1.20", Some("192.168.1.168")),
            "192.168.1.20"
        );
    }
}
