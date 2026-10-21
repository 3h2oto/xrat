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
