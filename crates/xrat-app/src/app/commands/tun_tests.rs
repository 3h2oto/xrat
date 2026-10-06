use super::tun::save_tun_enabled;

#[test]
fn tun_switches_preserve_config_and_are_idempotent() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("config.toml");
    std::fs::write(
        &path,
        "# keep this comment\n[runtime.socks]\nport = 18200\n",
    )
    .unwrap();
    let outcome = save_tun_enabled(&path, Some(true)).unwrap();
    assert!(outcome.config.runtime.tun.enabled);
    assert_eq!(outcome.config.runtime.socks.port, 18200);
    assert!(
        std::fs::read_to_string(&path)
            .unwrap()
            .contains("# keep this comment")
    );
    assert!(
        save_tun_enabled(&path, Some(true))
            .unwrap()
            .changed_paths
            .is_empty()
    );
    assert!(
        !save_tun_enabled(&path, None)
            .unwrap()
            .config
            .runtime
            .tun
            .enabled
    );
    assert!(
        save_tun_enabled(&path, Some(false))
            .unwrap()
            .changed_paths
            .is_empty()
    );
    assert!(
        save_tun_enabled(&path, None)
            .unwrap()
            .config
            .runtime
            .tun
            .enabled
    );
}

#[test]
fn tun_switch_rejects_invalid_config_without_writing() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("config.toml");
    let contents = "[runtime]\nengine = \"v2ray\"\n";
    std::fs::write(&path, contents).unwrap();
    assert!(save_tun_enabled(&path, Some(true)).is_err());
    assert_eq!(std::fs::read_to_string(path).unwrap(), contents);
}
