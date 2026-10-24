use super::*;

#[test]
#[ignore = "requires an elevated native Windows runner"]
fn native_machine_profile_preserves_registration_and_configuration() {
    let parent = tempfile::tempdir().unwrap();
    let path = parent.path().join("machine");
    let operator = xrat_support::windows::current_user_sid().unwrap();
    let profile = MachineProfile::provision_at(path.clone(), &operator).unwrap();
    std::fs::write(profile.config_path(), "# preserved\n").unwrap();
    drop(profile);
    let profile = MachineProfile::provision_at(path.clone(), &operator).unwrap();
    assert_eq!(profile.operator_sid(), operator);
    assert_eq!(
        std::fs::read_to_string(profile.config_path()).unwrap(),
        "# preserved\n"
    );
    drop(profile);
    assert!(MachineProfile::provision_at(path.clone(), "S-1-5-21-99-98-97-1001").is_err());
    assert_eq!(
        read_registration(&path.join("operator.json"))
            .unwrap()
            .operator_sid,
        operator
    );
}

#[test]
#[ignore = "requires an elevated native Windows runner"]
fn native_machine_profile_rejects_unprotected_existing_root() {
    let parent = tempfile::tempdir().unwrap();
    let path = parent.path().join("machine");
    std::fs::create_dir(&path).unwrap();
    assert!(MachineProfile::provision_at(path, "S-1-5-21-1-2-3-1001").is_err());
}

#[test]
#[ignore = "requires an elevated native Windows runner"]
fn native_machine_lifecycle_lease_excludes_other_hosts_and_provisioning() {
    let parent = tempfile::tempdir().unwrap();
    let path = parent.path().join("machine");
    let operator = xrat_support::windows::current_user_sid().unwrap();
    let profile = MachineProfile::provision_at(path.clone(), &operator).unwrap();
    let lease = profile.lifecycle_lease().unwrap();
    let second = MachineProfile::provision_at(path, &operator).unwrap();
    assert!(second.lifecycle_lease().is_err());
    profile.root.verify_tree().unwrap();
    drop(lease);
    let next = second.lifecycle_lease().unwrap();
    assert!(profile.lifecycle_lease().is_err());
    drop(next);
    assert!(profile.lifecycle_lease().is_ok());
}

#[test]
#[ignore = "requires an elevated native Windows runner"]
fn native_machine_core_activation_preserves_acl_and_locked_executable() {
    use std::os::windows::fs::OpenOptionsExt;
    let parent = tempfile::tempdir().unwrap();
    let operator = xrat_support::windows::current_user_sid().unwrap();
    let profile = MachineProfile::provision_at(parent.path().join("machine"), &operator).unwrap();
    let _lease = profile.lifecycle_lease().unwrap();
    let cores = profile.core_directory().unwrap();
    let target = cores.path().join("xray.exe");
    let mut old = cores.create_file("xray.exe").unwrap();
    old.write_all(b"old").unwrap();
    drop(old);
    let locked = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&target)
        .unwrap();
    let mut next = cores.create_file("next.exe").unwrap();
    next.write_all(b"new").unwrap();
    drop(next);
    let staged = tempfile::TempPath::try_from_path(cores.path().join("next.exe")).unwrap();
    assert!(staged.persist(&target).is_err());
    drop(locked);
    assert_eq!(std::fs::read(&target).unwrap(), b"old");
    let mut next = cores.create_file("next.exe").unwrap();
    next.write_all(b"new").unwrap();
    drop(next);
    tempfile::TempPath::try_from_path(cores.path().join("next.exe"))
        .unwrap()
        .persist(&target)
        .unwrap();
    assert_eq!(std::fs::read(&target).unwrap(), b"new");
    cores.verify_tree().unwrap();
}
