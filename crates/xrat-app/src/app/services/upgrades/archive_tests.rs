use super::*;

#[test]
fn checksum_requires_one_exact_valid_entry() {
    let directory = tempfile::tempdir().unwrap();
    let archive = directory.path().join("xrat.zip");
    std::fs::write(&archive, b"verified payload").unwrap();
    let hash = format!("{:x}", Sha256::digest(b"verified payload"));
    verify_checksum(&archive, &format!("{hash}  xrat.zip\n"), "xrat.zip").unwrap();
    verify_checksum(&archive, &format!("{hash} *xrat.zip\n"), "xrat.zip").unwrap();
    for sums in [
        format!("{hash} xrat.zip.extra"),
        format!("{hash} xrat.zip\n{hash} xrat.zip"),
        "bad xrat.zip".into(),
        format!("{} xrat.zip", "0".repeat(64)),
    ] {
        assert!(verify_checksum(&archive, &sums, "xrat.zip").is_err());
    }
}

#[test]
fn extracts_only_the_expected_regular_executable_from_zip_or_tar() {
    for zip in [true, false] {
        let directory = tempfile::tempdir().unwrap();
        let archive = directory
            .path()
            .join(if zip { "release.zip" } else { "release.tar.gz" });
        let executable = if zip { "xrat.exe" } else { "xrat" };
        let file = File::create(&archive).unwrap();
        if zip {
            let mut writer = zip::ZipWriter::new(file);
            writer
                .start_file(executable, zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(b"new binary").unwrap();
            writer
                .start_file("../unwanted", zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(b"unwanted").unwrap();
            writer.finish().unwrap();
        } else {
            let mut writer = tar::Builder::new(flate2::write::GzEncoder::new(
                file,
                flate2::Compression::default(),
            ));
            let mut header = tar::Header::new_gnu();
            header.set_size(10);
            header.set_mode(0o755);
            header.set_cksum();
            writer
                .append_data(&mut header, executable, &b"new binary"[..])
                .unwrap();
            writer.into_inner().unwrap().finish().unwrap();
        }
        extract_binary(&archive, directory.path(), executable).unwrap();
        assert_eq!(
            std::fs::read(directory.path().join(executable)).unwrap(),
            b"new binary"
        );
        assert!(!directory.path().parent().unwrap().join("unwanted").exists());
    }
}

#[test]
fn invalid_archive_preserves_existing_executable() {
    let directory = tempfile::tempdir().unwrap();
    let archive = directory.path().join("release.zip");
    std::fs::write(directory.path().join("xrat.exe"), b"old binary").unwrap();
    let mut writer = zip::ZipWriter::new(File::create(&archive).unwrap());
    writer
        .start_file(
            "elsewhere/xrat.exe",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
    writer.write_all(b"wrong member").unwrap();
    writer.finish().unwrap();
    assert!(extract_binary(&archive, directory.path(), "xrat.exe").is_err());
    assert_eq!(
        std::fs::read(directory.path().join("xrat.exe")).unwrap(),
        b"old binary"
    );
}
