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
