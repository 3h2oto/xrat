use crate::app::{
    self,
    context::AppContext,
    ports::EngineInfo,
    runtime_service::{RuntimeService, RuntimeStatusSnapshot},
};
use std::time::Duration;

pub(super) struct Source {
    pub snapshot: RuntimeStatusSnapshot,
    pub engines: Vec<EngineInfo>,
    pub engine: String,
    pub daemon_running: Option<bool>,
    pub rotation_enabled: bool,
    pub tun_enabled: bool,
    pub rotation_interval_secs: u64,
}

pub(super) async fn load(context: &AppContext) -> app::Result<Source> {
    #[cfg(any(windows, test))]
    if let Some(client) = &context.machine_client {
        let overview = client.overview().await?;
        let engines = app::services::engine_probe::probe_engines(
            context,
            &app::services::engine_probe::ProcessRuntimeEngineProbe,
        )
        .await;
        return Ok(Source {
            tun_enabled: overview.tun.enabled,
            engine: engine(&overview.runtime, overview.tun.engine),
            snapshot: overview.runtime,
            engines,
            daemon_running: Some(overview.daemon_ready),
            rotation_enabled: overview.rotation_enabled,
            rotation_interval_secs: overview.rotation_interval_secs,
        });
    }
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
    let rotation = &current.app_config.runtime.rotation;
    Ok(Source {
        tun_enabled: current.app_config.runtime.tun.enabled,
        engine: engine(&snapshot, current.app_config.runtime.engine.clone()),
        snapshot,
        engines,
        daemon_running,
        rotation_enabled: daemon
            .as_ref()
            .map(|value| value.rotation_enabled)
            .unwrap_or(rotation.enabled),
        rotation_interval_secs: daemon
            .as_ref()
            .map(|value| value.interval_secs)
            .unwrap_or(rotation.interval_secs),
    })
}

fn engine(snapshot: &RuntimeStatusSnapshot, configured: String) -> String {
    if snapshot
        .active_config
        .as_ref()
        .is_some_and(|config| config.protocol == "hysteria2")
    {
        "sing-box".into()
    } else {
        configured
    }
}
