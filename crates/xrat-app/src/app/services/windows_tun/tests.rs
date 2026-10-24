use super::package::*;
use std::io::{Cursor, Write};

fn archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, contents) in entries {
        writer
            .start_file(*name, zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(contents).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

#[test]
fn untrusted_packages_and_unsupported_architectures_are_rejected() {
    assert!(ARCHIVE_URL.starts_with("https://www.wintun.net/builds/"));
    for arch in ["x86_64", "aarch64", "x86", "riscv64"] {
        assert!(prepare(b"untrusted", arch).is_err());
    }
    assert!(prepare(&vec![0; ARCHIVE_LIMIT + 1], "x86_64").is_err());
}

#[test]
fn only_the_exact_architecture_and_license_members_are_read() {
    let bytes = archive(&[
        ("wintun/bin/amd64/wintun.dll", b"x64"),
        ("wintun/bin/arm64/wintun.dll", b"arm64"),
        ("wintun/LICENSE.txt", b"license"),
        ("../../untrusted.dll", b"ignored"),
    ]);
    let x64 = prepare_fixture(&bytes, "amd64", b"x64", b"license").unwrap();
    assert_eq!(x64.dll, b"x64");
    assert_eq!(x64.license, b"license");
    assert_eq!(
        prepare_fixture(&bytes, "arm64", b"arm64", b"license")
            .unwrap()
            .dll,
        b"arm64"
    );
    assert!(prepare_fixture(&bytes, "arm64", b"x64", b"license").is_err());
    assert!(prepare_fixture(&bytes, "amd64", b"x64", b"changed license").is_err());
}

#[test]
fn incomplete_empty_and_oversized_members_are_rejected() {
    for dll in [vec![], vec![0; 1024 * 1024 + 1]] {
        let bytes = archive(&[
            ("wintun/bin/amd64/wintun.dll", &dll),
            ("wintun/LICENSE.txt", b"license"),
        ]);
        assert!(prepare_fixture(&bytes, "amd64", &dll, b"license").is_err());
    }
    let bytes = archive(&[
        ("other/wintun.dll", b"x64"),
        ("wintun/LICENSE.txt", b"license"),
    ]);
    assert!(prepare_fixture(&bytes, "amd64", b"x64", b"license").is_err());
    let bytes = archive(&[("wintun/bin/amd64/wintun.dll", b"x64")]);
    assert!(prepare_fixture(&bytes, "amd64", b"x64", b"license").is_err());
}

#[test]
#[ignore = "requires XRAT_WINTUN_ARCHIVE pointing to the official 0.14.1 ZIP"]
fn official_package_matches_both_architectures_and_license() {
    let bytes =
        std::fs::read(std::env::var_os("XRAT_WINTUN_ARCHIVE").expect("archive path")).unwrap();
    assert_eq!(prepare(&bytes, "x86_64").unwrap().dll.len(), 427552);
    assert_eq!(prepare(&bytes, "aarch64").unwrap().dll.len(), 222488);
    #[cfg(windows)]
    for arch in ["x86_64", "aarch64"] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("wintun.dll");
        std::fs::write(&path, prepare(&bytes, arch).unwrap().dll).unwrap();
        super::verify_signature(&path).unwrap();
    }
    let mut tampered = bytes;
    tampered[100] ^= 1;
    assert!(prepare(&tampered, "x86_64").is_err());
}
