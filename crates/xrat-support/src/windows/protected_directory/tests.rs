use super::*;
use std::io::Write;
mod restricted;

#[test]
#[ignore = "requires an elevated native Windows runner"]
fn native_machine_directory_is_protected_and_reopenable() {
    let parent = tempfile::tempdir().unwrap();
    let path = parent.path().join("protected");
    let directory = ProtectedDirectory::create(&path).unwrap();
    assert_eq!(directory.path(), path);
    let reopened = ProtectedDirectory::create(&path).unwrap();
    directory
        .create_file("data")
        .unwrap()
        .write_all(b"retained")
        .unwrap();
    directory.verify_tree().unwrap();
    drop(reopened);
    drop(directory);
    let reopened = ProtectedDirectory::open(&path).unwrap();
    assert_eq!(
        std::fs::read_to_string(reopened.path().join("data")).unwrap(),
        "retained"
    );
}

#[test]
#[ignore = "requires an elevated native Windows runner"]
fn native_machine_directory_rejects_preexisting_user_directory() {
    let parent = tempfile::tempdir().unwrap();
    let path = parent.path().join("unprotected");
    std::fs::create_dir(&path).unwrap();
    assert_eq!(
        ProtectedDirectory::create(&path).err().unwrap().kind(),
        io::ErrorKind::PermissionDenied
    );
}

#[test]
#[ignore = "requires an elevated native Windows runner"]
fn native_machine_directory_rejects_reparse_points() {
    let parent = tempfile::tempdir().unwrap();
    let target = parent.path().join("target");
    let directory = ProtectedDirectory::create(&target).unwrap();
    let link = parent.path().join("link");
    std::os::windows::fs::symlink_dir(&target, &link).unwrap();
    assert_eq!(
        ProtectedDirectory::open(&link).err().unwrap().kind(),
        io::ErrorKind::PermissionDenied
    );
    drop(directory);
}

#[test]
#[ignore = "requires an elevated native Windows runner"]
fn native_machine_directory_denies_ordinary_user_read_and_write() {
    let parent = tempfile::tempdir().unwrap();
    let path = parent.path().join("protected");
    let directory = ProtectedDirectory::create(&path).unwrap();
    let data = path.join("config.toml");
    directory
        .create_file("config.toml")
        .unwrap()
        .write_all(b"protected configuration")
        .unwrap();
    directory.verify_tree().unwrap();
    let ordinary = restricted::OrdinaryUser::enter().unwrap();
    assert!(std::fs::read(&data).is_err());
    assert!(std::fs::write(&data, "tampered").is_err());
    assert!(std::fs::write(path.join("new.exe"), "untrusted").is_err());
    assert!(ProtectedDirectory::open(&path).is_err());
    use std::os::windows::fs::OpenOptionsExt;
    assert!(
        std::fs::OpenOptions::new()
            .access_mode(windows_sys::Win32::Storage::FileSystem::WRITE_DAC)
            .open(&data)
            .is_err()
    );
    drop(ordinary);
    assert_eq!(
        std::fs::read_to_string(&data).unwrap(),
        "protected configuration"
    );
    drop(directory);
}

#[test]
#[ignore = "requires an elevated native Windows runner"]
fn native_machine_directory_rejects_user_owned_descendants() {
    let parent = tempfile::tempdir().unwrap();
    let path = parent.path().join("protected");
    let directory = ProtectedDirectory::create(&path).unwrap();
    let user = crate::windows::current_user_sid().unwrap();
    let descriptor =
        SecurityDescriptor::from_sddl(&format!("O:{user}D:P(A;;FA;;;SY)(A;;FA;;;BA)")).unwrap();
    let attributes = descriptor.attributes();
    let name = path
        .join("untrusted.exe")
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let handle = unsafe {
        CreateFileW(
            name.as_ptr(),
            FILE_GENERIC_WRITE,
            0,
            &attributes,
            CREATE_NEW,
            FILE_ATTRIBUTE_NORMAL,
            ptr::null_mut(),
        )
    };
    assert_ne!(handle, INVALID_HANDLE_VALUE);
    drop(unsafe { std::fs::File::from_raw_handle(handle) });
    assert_eq!(
        directory.verify_tree().unwrap_err().kind(),
        io::ErrorKind::PermissionDenied
    );
}
