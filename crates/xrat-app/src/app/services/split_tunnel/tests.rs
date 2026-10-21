use super::*;

#[test]
fn wrapper_resolution_does_not_route_auxiliary_commands() {
    let dir = tempfile::tempdir().unwrap();
    let wrapper = dir.path().join("browser");
    std::fs::write(
        &wrapper,
        "#!/bin/sh\n/usr/bin/pidof $APP\nexec $APP \"$@\"\n",
    )
    .unwrap();
    assert_eq!(extract_target_from_wrapper_script(&wrapper), None);
}

#[test]
#[cfg(unix)]
fn canonical_executable_paths_are_kept_for_symlink_rules() {
    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("actual-binary");
    std::fs::write(&binary, "binary fixture").unwrap();
    let canonical_binary = std::fs::canonicalize(&binary).unwrap();
    let alias = dir.path().join("alias");
    std::os::unix::fs::symlink(&binary, &alias).unwrap();
    let tun = TunSettings {
        split_mode: TunSplitMode::Whitelist,
        whitelist: vec![alias.display().to_string()],
        ..Default::default()
    };
    let compiled = compile_split_rules_with_dirs(&tun, &[]);
    assert!(
        compiled
            .singbox_process_paths
            .contains(&canonical_binary.display().to_string())
    );
    assert!(
        compiled
            .xray_processes
            .contains(&canonical_binary.display().to_string())
    );
}

#[test]
#[cfg(not(target_os = "macos"))]
fn desktop_id_validation_compilation_and_discovery_use_the_same_identity() {
    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("actual-browser");
    std::fs::write(&binary, "binary fixture").unwrap();
    std::fs::write(
        dir.path().join("demo.desktop"),
        format!(
            "[Desktop Entry]\nType=Application\nName=Demo\nExec={}\n",
            binary.display()
        ),
    )
    .unwrap();
    let dirs = vec![dir.path().to_path_buf()];
    let normalized = normalize_app_specifier_with_dirs("demo", &dirs).unwrap();
    let tun = TunSettings {
        split_mode: TunSplitMode::Whitelist,
        whitelist: vec!["demo".into()],
        ..Default::default()
    };
    let compiled = compile_split_rules_with_dirs(&tun, &dirs);
    assert_eq!(compiled.xray_processes, normalized);
    let discovered = discover_desktop_apps_in_dirs(&dirs);
    assert_eq!(discovered[0].rule_entry, "demo.desktop");
    let selected = TunSettings {
        whitelist: vec![discovered[0].rule_entry.clone()],
        ..tun
    };
    assert_eq!(compile_split_rules_with_dirs(&selected, &dirs), compiled);
}

#[test]
#[cfg(target_os = "linux")]
fn running_desktop_app_keeps_its_desktop_specifier() {
    let directory = tempfile::tempdir().unwrap();
    let executable = std::env::current_exe().unwrap();
    std::fs::write(
        directory.path().join("running-fixture.desktop"),
        format!(
            "[Desktop Entry]\nType=Application\nName=Running Fixture\nExec={}\n",
            executable.display()
        ),
    )
    .unwrap();
    let apps = discover_apps_with_dirs(
        true,
        true,
        Some("Running Fixture"),
        &[directory.path().to_path_buf()],
    );
    assert_eq!(apps.len(), 1);
    assert_eq!(apps[0].source, AppSource::RunningAndDesktop);
    assert_eq!(apps[0].rule_entry, "running-fixture.desktop");
}

#[test]
fn extracts_executable_from_complex_desktop_exec_lines() {
    assert_eq!(
        extract_executable_from_exec_line("/usr/lib/firefox/firefox %u"),
        Some("/usr/lib/firefox/firefox".to_string())
    );
    assert_eq!(
        extract_executable_from_exec_line(
            "env BAMF_DESKTOP_FILE_HINT=/var/lib/snapd/desktop/applications/telegram.desktop /opt/telegram/Telegram -- %u"
        ),
        Some("/opt/telegram/Telegram".to_string())
    );
    assert_eq!(
        extract_executable_from_exec_line(
            "/usr/bin/flatpak run --branch=stable --arch=x86_64 --command=telegram-desktop org.telegram.desktop -- %u"
        ),
        Some("telegram-desktop".to_string())
    );
    assert_eq!(
        extract_executable_from_exec_line("\"/opt/Google Chrome/chrome\" --new-window %U"),
        Some("/opt/Google Chrome/chrome".to_string())
    );
}

#[test]
fn parses_desktop_files_and_skips_hidden_or_non_app_entries() {
    let valid = r#"
[Desktop Entry]
Type=Application
Name=Firefox Web Browser
Exec=/usr/lib/firefox/firefox %u
"#;
    assert_eq!(
        parse_desktop_entry_content("firefox.desktop", valid),
        Some((
            "Firefox Web Browser".to_string(),
            "/usr/lib/firefox/firefox".to_string()
        ))
    );

    let hidden = r#"
[Desktop Entry]
Type=Application
Name=Hidden Helper
NoDisplay=true
Exec=/usr/bin/helper
"#;
    assert_eq!(parse_desktop_entry_content("helper.desktop", hidden), None);
}

#[test]
#[cfg(all(unix, not(target_os = "macos")))]
fn resolves_desktop_specifiers_from_custom_directory() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("org.telegram.desktop"),
        "[Desktop Entry]\nType=Application\nName=Telegram Desktop\nExec=/xrat-fixtures/telegram-desktop -- %u\n",
    )
    .unwrap();
    let dirs = vec![dir.path().to_path_buf()];

    let resolved = normalize_app_entry_with_dirs("org.telegram.desktop", true, &dirs).unwrap();
    assert_eq!(
        resolved,
        vec![
            "telegram-desktop".to_string(),
            "/xrat-fixtures/telegram-desktop".to_string()
        ]
    );

    let by_name = normalize_app_entry_with_dirs("desktop:Telegram Desktop", true, &dirs).unwrap();
    assert_eq!(
        by_name,
        vec![
            "telegram-desktop".to_string(),
            "/xrat-fixtures/telegram-desktop".to_string()
        ]
    );
}

#[test]
fn compiles_split_rules_for_xray_and_singbox() {
    let tun = TunSettings {
        split_mode: TunSplitMode::Blacklist,
        blacklist: vec![
            "xrat-fixture-browser-213".to_string(),
            "/xrat-fixtures/curl".to_string(),
            "/opt/discord/".to_string(),
        ],
        ..Default::default()
    };

    let compiled = compile_split_rules_with_dirs(&tun, &[]);
    assert_eq!(compiled.mode, TunSplitMode::Blacklist);
    assert_eq!(
        compiled.xray_processes,
        vec![
            "xrat-fixture-browser-213",
            "/xrat-fixtures/curl",
            "/opt/discord/"
        ]
    );
    assert_eq!(
        compiled.singbox_process_names,
        vec!["xrat-fixture-browser-213"]
    );
    assert_eq!(compiled.singbox_process_paths, vec!["/xrat-fixtures/curl"]);
    assert_eq!(
        compiled.singbox_process_path_regexes,
        vec!["^/opt/discord/.*"]
    );
}

#[test]
#[cfg(unix)]
fn resolves_shebang_wrapper_script_to_real_binary() {
    let dir = tempfile::tempdir().unwrap();
    let wrapper = std::fs::canonicalize(dir.path())
        .unwrap()
        .join("zen-browser");
    std::fs::write(
        &wrapper,
        "#!/bin/bash\n\nexec /opt/zen-browser-bin/zen-bin \"$@\"\n",
    )
    .unwrap();
    assert_eq!(
        extract_target_from_wrapper_script(&wrapper),
        Some("/opt/zen-browser-bin/zen-bin".to_string())
    );

    let tun = TunSettings {
        split_mode: TunSplitMode::Whitelist,
        whitelist: vec![wrapper.display().to_string()],
        ..Default::default()
    };
    let compiled = compile_split_rules_with_dirs(&tun, &[]);
    assert_eq!(
        compiled.xray_processes,
        vec![
            wrapper.display().to_string(),
            "zen-bin".to_string(),
            "/opt/zen-browser-bin/".to_string()
        ]
    );
    assert_eq!(compiled.singbox_process_names, vec!["zen-bin".to_string()]);
    assert_eq!(
        compiled.singbox_process_path_regexes,
        vec!["^/opt/zen-browser-bin/.*".to_string()]
    );
}
