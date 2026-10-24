use super::*;

#[test]
fn unattended_setup_installs_recommended_missing_cores() {
    assert!(unattended_dependency_change(cores::CoreKind::Xray, true));
    assert!(unattended_dependency_change(cores::CoreKind::SingBox, true));
    assert!(!unattended_dependency_change(cores::CoreKind::V2Ray, true));
}

#[test]
fn unattended_setup_updates_any_installed_core() {
    for kind in cores::CORE_KINDS {
        assert!(unattended_dependency_change(kind, false));
    }
}

#[cfg(unix)]
#[test]
fn tui_shim_is_a_shell_script_named_xratui() {
    let path = steps::tui_shim_path().expect("executable directory resolves");
    assert_eq!(
        path.file_name().and_then(|name| name.to_str()),
        Some("xratui")
    );

    let script = steps::tui_shim_script(std::path::Path::new("/opt/xrat/bin/xrat"));
    assert!(script.starts_with("#!/usr/bin/env sh\n"));
    assert!(script.contains("tui \"$@\""));
    assert!(script.contains("/opt/xrat/bin/xrat"));

    let probed = steps::probe_tui_shim();
    assert!(
        matches!(probed.status, StepStatus::AlreadyDone | StepStatus::Missing),
        "unix setup must report the shim as present or absent, got {:?}",
        probed.status
    );
}

#[cfg(windows)]
#[test]
fn tui_shim_step_is_skipped_and_writes_nothing_on_windows() {
    let shim = steps::exe_dir().map(|dir| dir.join("xratui"));
    let existed = shim.as_ref().is_some_and(|path| path.exists());

    let probed = steps::probe_tui_shim();
    assert_eq!(probed.name, steps::STEP_TUI_SHIM);
    assert_eq!(probed.status, StepStatus::Skipped);

    let applied = steps::apply_tui_shim();
    assert_eq!(applied.name, steps::STEP_TUI_SHIM);
    assert_eq!(applied.status, StepStatus::Skipped);

    if !existed {
        assert!(
            shim.as_ref().is_none_or(|path| !path.exists()),
            "the skipped windows step must not write a shim"
        );
    }
}
