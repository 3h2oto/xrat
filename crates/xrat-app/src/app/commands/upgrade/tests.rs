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
