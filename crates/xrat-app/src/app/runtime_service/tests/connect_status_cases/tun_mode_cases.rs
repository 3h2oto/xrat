use super::*;
use crate::app::services::tun::{apply_with_ports, capture_state};

async fn context_and_ports() -> (
    AppContext,
    RuntimeProcessPorts,
    Arc<Mutex<State>>,
    ConfigRecord,
) {
    let mut context = test_context().await;
    context.runtime_paths.xray_path = context.runtime_paths.root_dir.join("fake-xray");
    std::fs::write(&context.runtime_paths.xray_path, "test binary").unwrap();
    context.app_config.paths.xray = Some(context.runtime_paths.xray_path.clone());
    save_config(&context);
    let config = import_single_config(&context).await;
    let (ports, state) = ports(false, false);
    state.lock().unwrap().track_tun = true;
    (context, ports, state, config)
}

fn save_config(context: &AppContext) {
    std::fs::write(&context.runtime_paths.config_path, format!(
        "[runtime]\nengine = {:?}\nreplace_active_session = {}\n[runtime.socks]\nport = {}\n[runtime.http]\nport = {}\n[runtime.shadowsocks]\nport = {}\n[runtime.tun]\nenabled = {}\n[paths]\nxray = {:?}\nsing_box = {:?}\n",
        context.app_config.runtime.engine,
        context.app_config.runtime.replace_active_session,
        context.app_config.runtime.socks.port,
        context.app_config.runtime.http.port,
        context.app_config.runtime.shadowsocks.port,
        context.app_config.runtime.tun.enabled,
        context.runtime_paths.xray_path.display().to_string(),
        context.runtime_paths.sing_box_path.display().to_string(),
    )).unwrap();
}

async fn connect(context: &AppContext, ports: &RuntimeProcessPorts, config: &ConfigRecord) {
    RuntimeService::with_process_ports(context, ports.clone())
        .connect(ConnectRequest {
            config_id: config.id,
        })
        .await
        .unwrap();
}

#[tokio::test]
async fn live_tun_enable_disable_reuses_config_and_is_idempotent_for_both_engines() {
    for singbox in [false, true] {
        let (mut context, ports, state, config) = context_and_ports().await;
        if singbox {
            context.app_config.runtime.engine = "sing-box".into();
            context.runtime_paths.sing_box_path = context.runtime_paths.xray_path.clone();
            context.app_config.paths.sing_box = Some(context.runtime_paths.sing_box_path.clone());
            save_config(&context);
        }
        connect(&context, &ports, &config).await;
        let enabled = apply_with_ports(&mut context, Some(true), ports.clone(), true)
            .await
            .unwrap();
        assert!(enabled.enabled && enabled.active);
        assert_eq!(
            enabled.active_config_ref.as_deref(),
            Some(config.r#ref.as_str())
        );
        assert_eq!(state.lock().unwrap().spawned, 2);
        assert!(context.app_config.runtime.tun.enabled);
        let session = enabled.session_id;
        let repeated = apply_with_ports(&mut context, Some(true), ports.clone(), true)
            .await
            .unwrap();
        assert_eq!(session, repeated.session_id);
        assert_eq!(state.lock().unwrap().spawned, 2);
        let disabled = apply_with_ports(&mut context, Some(false), ports.clone(), true)
            .await
            .unwrap();
        assert!(!disabled.enabled && !disabled.active);
        assert_eq!(disabled.active_config_ref, Some(config.r#ref.clone()));
        assert!(state.lock().unwrap().tun_interface.is_none());
        assert_eq!(state.lock().unwrap().spawned, 3);
        let saved: crate::app::config::AppConfig =
            toml::from_str(&std::fs::read_to_string(&context.runtime_paths.config_path).unwrap())
                .unwrap();
        assert!(!saved.runtime.tun.enabled);
        RuntimeService::with_process_ports(&context, ports)
            .disconnect()
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn live_tun_checks_preserve_proxy_session_and_config() {
    for failure in [
        "old-core",
        "unknown-core",
        "engine-capabilities",
        "owner-capabilities",
        "preflight",
    ] {
        let (mut context, mut ports, state, config) = context_and_ports().await;
        connect(&context, &ports, &config).await;
        let original = std::fs::read(&context.runtime_paths.config_path).unwrap();
        let old_session = context
            .db
            .get_running_runtime_session()
            .await
            .unwrap()
            .unwrap()
            .id;
        match failure {
            "old-core" => state.lock().unwrap().version = Some("Xray 26.3.27".into()),
            "unknown-core" => state.lock().unwrap().version = Some("not an Xray banner".into()),
            "engine-capabilities" => state.lock().unwrap().missing_capabilities = true,
            "preflight" => {
                ports.spawner = Arc::new(FakePorts {
                    state: state.clone(),
                    fail_readiness: false,
                    fail_validation: true,
                })
            }
            _ => {}
        }
        let error = apply_with_ports(
            &mut context,
            Some(true),
            ports.clone(),
            failure != "owner-capabilities",
        )
        .await
        .unwrap_err();
        assert!(!error.to_string().is_empty());
        assert_eq!(
            std::fs::read(&context.runtime_paths.config_path).unwrap(),
            original
        );
        assert_eq!(
            context
                .db
                .get_running_runtime_session()
                .await
                .unwrap()
                .unwrap()
                .id,
            old_session
        );
        assert_eq!(state.lock().unwrap().spawned, 1);
        assert!(state.lock().unwrap().running);
        assert!(!context.app_config.runtime.tun.enabled);
        RuntimeService::with_process_ports(&context, ports)
            .disconnect()
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn live_tun_startup_failure_restores_the_previous_mode_in_both_directions() {
    for previous_tun in [false, true] {
        let (mut context, ports, state, config) = context_and_ports().await;
        context.app_config.runtime.tun.enabled = previous_tun;
        save_config(&context);
        connect(&context, &ports, &config).await;
        let original = std::fs::read(&context.runtime_paths.config_path).unwrap();
        state.lock().unwrap().startup_failures = 1;
        let error = apply_with_ports(&mut context, Some(!previous_tun), ports.clone(), true)
            .await
            .unwrap_err()
            .to_string();
        assert!(error.contains("previous runtime was restored"), "{error}");
        assert_eq!(
            std::fs::read(&context.runtime_paths.config_path).unwrap(),
            original
        );
        assert_eq!(context.app_config.runtime.tun.enabled, previous_tun);
        let snapshot = RuntimeService::with_process_ports(&context, ports.clone())
            .status()
            .await
            .unwrap();
        assert!(snapshot.pid_running);
        assert_eq!(snapshot.active_config.unwrap().id, config.id);
        let snapshot = RuntimeService::with_process_ports(&context, ports.clone())
            .status()
            .await
            .unwrap();
        assert_eq!(
            capture_state(&context, &snapshot, &ports).active,
            previous_tun
        );
        RuntimeService::with_process_ports(&context, ports)
            .disconnect()
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn live_tun_save_conflict_restores_connection_and_keeps_external_edit() {
    for previous_tun in [false, true] {
        let (mut context, ports, state, config) = context_and_ports().await;
        context.app_config.runtime.tun.enabled = previous_tun;
        save_config(&context);
        connect(&context, &ports, &config).await;
        state.lock().unwrap().change_config_on_spawn =
            Some(context.runtime_paths.config_path.clone());
        let error = apply_with_ports(&mut context, Some(!previous_tun), ports.clone(), true)
            .await
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("previous mode and connection restored"),
            "{error}"
        );
        let contents = std::fs::read_to_string(&context.runtime_paths.config_path).unwrap();
        assert!(contents.contains("# external edit"));
        let saved: crate::app::config::AppConfig = toml::from_str(&contents).unwrap();
        assert_eq!(saved.runtime.tun.enabled, previous_tun);
        let snapshot = RuntimeService::with_process_ports(&context, ports.clone())
            .status()
            .await
            .unwrap();
        assert!(snapshot.pid_running);
        assert_eq!(
            capture_state(&context, &snapshot, &ports).active,
            previous_tun
        );
        RuntimeService::with_process_ports(&context, ports)
            .disconnect()
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn disconnected_tun_toggle_saves_without_starting_runtime() {
    let (mut context, ports, state, _) = context_and_ports().await;
    let enabled = apply_with_ports(&mut context, None, ports.clone(), false)
        .await
        .unwrap();
    assert!(enabled.enabled && !enabled.active);
    assert!(enabled.active_config_ref.is_none());
    assert_eq!(state.lock().unwrap().spawned, 0);
    let disabled = apply_with_ports(&mut context, None, ports, false)
        .await
        .unwrap();
    assert!(!disabled.enabled && !disabled.active);
}

#[tokio::test]
async fn invalid_tun_config_does_not_write_or_replace() {
    let (mut context, ports, state, config) = context_and_ports().await;
    connect(&context, &ports, &config).await;
    let contents = "[runtime]\nengine = \"v2ray\"\n";
    std::fs::write(&context.runtime_paths.config_path, contents).unwrap();
    assert!(
        apply_with_ports(&mut context, Some(true), ports.clone(), true)
            .await
            .is_err()
    );
    assert_eq!(
        std::fs::read_to_string(&context.runtime_paths.config_path).unwrap(),
        contents
    );
    assert_eq!(state.lock().unwrap().spawned, 1);
    assert!(state.lock().unwrap().running);
    RuntimeService::with_process_ports(&context, ports)
        .disconnect()
        .await
        .unwrap();
}

#[tokio::test]
async fn active_tun_state_requires_matching_session_and_kernel_identity() {
    let (mut context, ports, state, config) = context_and_ports().await;
    context.app_config.runtime.tun.enabled = true;
    connect(&context, &ports, &config).await;
    let snapshot = RuntimeService::with_process_ports(&context, ports.clone())
        .status()
        .await
        .unwrap();
    assert!(capture_state(&context, &snapshot, &ports).active);
    state
        .lock()
        .unwrap()
        .tun_interface
        .as_mut()
        .unwrap()
        .ifindex = 99;
    assert!(!capture_state(&context, &snapshot, &ports).active);
    state
        .lock()
        .unwrap()
        .tun_interface
        .as_mut()
        .unwrap()
        .ifindex = 42;
    let mut record = crate::app::runtime_service::tun_ownership::load_ownership(
        &context.runtime_paths.runtime_dir,
    )
    .unwrap();
    record.session_id += 1;
    crate::app::runtime_service::tun_ownership::save_ownership(
        &context.runtime_paths.runtime_dir,
        &record,
    )
    .unwrap();
    assert!(!capture_state(&context, &snapshot, &ports).active);
    RuntimeService::with_process_ports(&context, ports)
        .disconnect()
        .await
        .unwrap();
}

#[tokio::test]
async fn missing_tun_interface_does_not_change_the_mode_used_for_rollback() {
    let (mut context, ports, state, config) = context_and_ports().await;
    context.app_config.runtime.tun.enabled = true;
    save_config(&context);
    connect(&context, &ports, &config).await;
    {
        let mut state = state.lock().unwrap();
        state.tun_interface = None;
        state.startup_failures = 1;
    }
    let error = apply_with_ports(&mut context, Some(false), ports.clone(), true)
        .await
        .unwrap_err()
        .to_string();
    assert!(error.contains("previous runtime was restored"), "{error}");
    let snapshot = RuntimeService::with_process_ports(&context, ports.clone())
        .status()
        .await
        .unwrap();
    assert!(snapshot.pid_running);
    assert!(capture_state(&context, &snapshot, &ports).active);
    assert!(context.app_config.runtime.tun.enabled);
    RuntimeService::with_process_ports(&context, ports)
        .disconnect()
        .await
        .unwrap();
}
