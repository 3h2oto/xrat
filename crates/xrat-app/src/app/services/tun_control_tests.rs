use super::*;

#[test]
fn rollback_restores_tun_without_overwriting_other_settings() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("config.toml");
    let mut config = crate::app::config::AppConfig::default();
    let previous = config.runtime.tun.clone();
    config.runtime.tun.whitelist = vec!["example".into()];
    let expected = config.runtime.tun.clone();
    std::fs::write(
        &path,
        "# keep this comment\n[runtime.tun]\nwhitelist = [\"example\"]\n",
    )
    .unwrap();
    restore_tun_settings_on_disk(&path, &previous, &expected).unwrap();
    let restored: crate::app::config::AppConfig =
        toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(restored.runtime.tun, previous);
    assert_eq!(restored.dns, config.dns);
    assert!(
        std::fs::read_to_string(&path)
            .unwrap()
            .contains("# keep this comment")
    );
}

#[test]
fn rollback_refuses_newer_tun_settings_and_reports_missing_file() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("config.toml");
    let mut config = crate::app::config::AppConfig::default();
    let previous = config.runtime.tun.clone();
    config.runtime.tun.whitelist = vec!["concurrent-edit".into()];
    let contents = "[runtime.tun]\nwhitelist = [\"concurrent-edit\"]\n";
    std::fs::write(&path, contents).unwrap();
    assert!(
        restore_tun_settings_on_disk(&path, &previous, &previous)
            .unwrap_err()
            .to_string()
            .contains("changed concurrently")
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), contents);
    std::fs::remove_file(&path).unwrap();
    assert!(restore_tun_settings_on_disk(&path, &previous, &previous).is_err());
}
