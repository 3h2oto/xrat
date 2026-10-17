use super::*;

#[test]
fn same_version_ignores_v_prefix() {
    assert!(same_version("v0.2.1", "0.2.1"));
    assert!(same_version("0.2.1", "v0.2.1"));
    assert!(same_version(" v0.2.1 ", "0.2.1"));
    assert!(!same_version("v0.2.1", "0.2.2"));
}

#[test]
fn should_restore_tun_respects_explicit_flags() {
    let dummy_target = Path::new("/dummy/xrat");
    let dummy_config = Path::new("/dummy/config.toml");

    let args = UpgradeArgs {
        no_tun_setup: true,
        ..Default::default()
    };
    assert!(!should_restore_tun(dummy_target, dummy_config, &args));

    let args = UpgradeArgs {
        tun_setup: true,
        ..Default::default()
    };
    assert!(should_restore_tun(dummy_target, dummy_config, &args));
}

#[test]
fn should_restart_daemon_respects_flag_and_config() {
    let root = tempfile::tempdir().unwrap();
    let config_path = root.path().join("config.toml");

    // Default without flag -> true
    let args = UpgradeArgs::default();
    assert!(should_restart_daemon(&config_path, &args));

    // With --no-restart-daemon flag -> false
    let args = UpgradeArgs {
        no_restart_daemon: true,
        ..Default::default()
    };
    assert!(!should_restart_daemon(&config_path, &args));

    // With config restart_daemon_on_upgrade = false
    std::fs::write(
        &config_path,
        "[runtime.tun]\nrestart_daemon_on_upgrade = false\n",
    )
    .unwrap();
    let args = UpgradeArgs::default();
    assert!(!should_restart_daemon(&config_path, &args));
}

#[cfg(unix)]
#[tokio::test]
async fn stale_socket_does_not_start_daemon_after_upgrade() {
    use std::os::unix::net::UnixListener;
    use std::sync::Arc;

    let root = tempfile::tempdir().unwrap();
    let socket_path = root.path().join("daemon.sock");
    let listener = UnixListener::bind(&socket_path).unwrap();
    drop(listener);
    assert!(socket_path.exists());
    let marker = root.path().join("started");
    let target = root.path().join("xrat");
    std::fs::write(
        &target,
        format!("#!/bin/sh\ntouch '{}'\n", marker.display()),
    )
    .unwrap();
    super::updater::set_executable(&target).unwrap();

    super::updater::restart_standalone_if_active(
        &target,
        &root.path().join("config.toml"),
        &socket_path,
        Arc::new(xrat_support::process::SystemProcessSpawner),
    )
    .await;

    assert!(
        !marker.exists(),
        "stale socket must not trigger daemon restart"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn responsive_daemon_is_restarted_after_upgrade() {
    use std::sync::Arc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let root = tempfile::tempdir().unwrap();
    let socket_path = root.path().join("daemon.sock");
    let listener = tokio::net::UnixListener::bind(&socket_path).unwrap();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = Vec::new();
        stream.read_to_end(&mut request).await.unwrap();
        let response = crate::app::daemon::ipc::ping_response();
        stream
            .write_all(&serde_json::to_vec(&response).unwrap())
            .await
            .unwrap();
    });
    let marker = root.path().join("started");
    let target = root.path().join("xrat");
    std::fs::write(
        &target,
        format!("#!/bin/sh\ntouch '{}'\n", marker.display()),
    )
    .unwrap();
    super::updater::set_executable(&target).unwrap();

    super::updater::restart_standalone_if_active(
        &target,
        &root.path().join("config.toml"),
        &socket_path,
        Arc::new(xrat_support::process::SystemProcessSpawner),
    )
    .await;

    server.await.unwrap();
    assert!(marker.exists(), "responsive daemon must be restarted");
}
