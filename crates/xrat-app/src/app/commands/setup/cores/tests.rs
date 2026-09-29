use super::install::*;
use super::prelude::*;
use super::release::*;

use flate2::Compression;
use flate2::write::GzEncoder;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

async fn serve_once(response: &'static [u8]) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = [0_u8; 1024];
        let _ = socket.read(&mut request).await;
        socket.write_all(response).await.unwrap();
    });
    format!("http://{address}/core")
}

fn test_release(url: String) -> CoreRelease {
    CoreRelease {
        version: Version::new(5, 52, 0),
        tag: "v5.52.0".to_string(),
        asset: ReleaseAsset {
            name: "v2ray.zip".to_string(),
            url,
            sha256: "0".repeat(64),
        },
    }
}

#[tokio::test]
async fn streams_downloads_with_known_and_unknown_lengths() {
    let client = reqwest::Client::new();
    let responses: [(&[u8], &[u8]); 2] = [
        (
            b"HTTP/1.1 200 OK\r\nContent-Length: 11\r\nConnection: close\r\n\r\nhello world",
            b"hello world",
        ),
        (
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n5\r\nhello\r\n6\r\n world\r\n0\r\n\r\n",
            b"hello world",
        ),
    ];

    for (response, expected) in responses {
        let release = test_release(serve_once(response).await);
        let bytes = download_archive(&client, CoreKind::V2Ray, &release, false)
            .await
            .unwrap();
        assert_eq!(bytes, expected);
    }
}

#[test]
fn parses_versions_from_supported_core_output() {
    assert_eq!(
        parse_version("Xray 26.3.27 (Xray, Penetrates Everything.)"),
        Some(Version::new(26, 3, 27))
    );
    assert_eq!(
        parse_version("sing-box version 1.13.18"),
        Some(Version::new(1, 13, 18))
    );
    assert_eq!(
        parse_version("V2Ray 5.52.0 (V2Fly)"),
        Some(Version::new(5, 52, 0))
    );
}

#[test]
fn pins_the_managed_sing_box_release() {
    assert_eq!(
        default_version(CoreKind::SingBox),
        Some(Version::new(1, 13, 21))
    );
    assert_eq!(default_version(CoreKind::Xray), None);
    assert_eq!(default_version(CoreKind::V2Ray), None);
    assert_eq!(
        release_api_url(
            CoreKind::SingBox,
            default_version(CoreKind::SingBox).as_ref(),
            false,
        ),
        "https://api.github.com/repos/SagerNet/sing-box/releases/tags/v1.13.21"
    );
}

#[test]
fn builds_latest_and_pinned_release_api_urls() {
    assert_eq!(
        release_api_url(CoreKind::Xray, None, false),
        "https://api.github.com/repos/XTLS/Xray-core/releases/latest"
    );
    assert_eq!(
        release_api_url(CoreKind::SingBox, Some(&Version::new(1, 13, 2)), false,),
        "https://api.github.com/repos/SagerNet/sing-box/releases/tags/v1.13.2"
    );
    assert_eq!(
        release_api_url(CoreKind::V2Ray, Some(&Version::new(5, 52, 0)), false),
        "https://api.github.com/repos/v2fly/v2ray-core/releases/tags/v5.52.0"
    );
    assert_eq!(
        release_api_url(CoreKind::Xray, None, true),
        "https://api.github.com/repos/XTLS/Xray-core/releases?per_page=100"
    );
}

#[test]
fn selects_newest_published_prerelease_by_creation_time() {
    let releases = [
        ("v26.3.27", false, "2026-03-27T00:00:00Z"),
        ("v26.6.1", true, "2026-06-01T00:00:00Z"),
        ("v26.7.28", true, "2026-07-28T00:00:00Z"),
    ]
    .into_iter()
    .map(|(tag, prerelease, created_at)| GithubRelease {
        tag_name: tag.to_string(),
        prerelease,
        created_at: created_at.to_string(),
        assets: Vec::new(),
    })
    .collect();

    let selected = newest_prerelease(releases).unwrap();

    assert_eq!(selected.tag_name, "v26.7.28");
}

#[test]
fn rejects_missing_or_invalid_release_digest() {
    assert!(parse_sha256(None).is_err());
    assert!(parse_sha256(Some("sha256:nope")).is_err());
    assert!(parse_sha256(Some("sha512:abcd")).is_err());
}

#[test]
fn rejects_checksum_mismatch() {
    let asset = ReleaseAsset {
        name: "core.zip".to_string(),
        url: "https://example.invalid/core.zip".to_string(),
        sha256: "0".repeat(64),
    };
    assert!(verify_checksum(&asset, b"different").is_err());
}

#[test]
fn release_metadata_requires_the_platform_asset_digest() {
    let version = Version::new(1, 13, 18);
    let name = asset_name(CoreKind::SingBox, &version).unwrap();
    let payload = GithubRelease {
        tag_name: "v1.13.18".to_string(),
        prerelease: false,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        assets: vec![GithubAsset {
            name,
            browser_download_url: "https://example.invalid/sing-box.tar.gz".to_string(),
            digest: None,
        }],
    };
    assert!(release_from_payload(CoreKind::SingBox, payload).is_err());
}

#[test]
fn detects_outdated_installed_core() {
    let probe = CoreProbe {
        kind: CoreKind::V2Ray,
        path: Some(PathBuf::from("/usr/bin/v2ray")),
        version: Some(Version::new(5, 48, 0)),
        managed: false,
        latest: Ok(CoreRelease {
            version: Version::new(5, 52, 0),
            tag: "v5.52.0".to_string(),
            asset: ReleaseAsset {
                name: "v2ray.zip".to_string(),
                url: "https://example.invalid/v2ray.zip".to_string(),
                sha256: "0".repeat(64),
            },
        }),
    };
    assert!(probe.outdated());
    assert!(probe.detail().contains("latest v5.52.0; external"));
}

#[test]
fn selects_supported_platform_assets() {
    let version = Version::new(1, 13, 18);
    let cases = [
        (CoreKind::Xray, "linux", "x86_64", "Xray-linux-64.zip"),
        (
            CoreKind::Xray,
            "linux",
            "aarch64",
            "Xray-linux-arm64-v8a.zip",
        ),
        (CoreKind::Xray, "macos", "x86_64", "Xray-macos-64.zip"),
        (
            CoreKind::Xray,
            "macos",
            "aarch64",
            "Xray-macos-arm64-v8a.zip",
        ),
        (CoreKind::V2Ray, "linux", "x86_64", "v2ray-linux-64.zip"),
        (
            CoreKind::V2Ray,
            "linux",
            "aarch64",
            "v2ray-linux-arm64-v8a.zip",
        ),
        (CoreKind::V2Ray, "macos", "x86_64", "v2ray-macos-64.zip"),
        (
            CoreKind::V2Ray,
            "macos",
            "aarch64",
            "v2ray-macos-arm64-v8a.zip",
        ),
        (
            CoreKind::SingBox,
            "linux",
            "x86_64",
            "sing-box-1.13.18-linux-amd64.tar.gz",
        ),
        (
            CoreKind::SingBox,
            "linux",
            "aarch64",
            "sing-box-1.13.18-linux-arm64.tar.gz",
        ),
        (
            CoreKind::SingBox,
            "macos",
            "x86_64",
            "sing-box-1.13.18-darwin-amd64.tar.gz",
        ),
        (
            CoreKind::SingBox,
            "macos",
            "aarch64",
            "sing-box-1.13.18-darwin-arm64.tar.gz",
        ),
    ];
    for (kind, os, arch, expected) in cases {
        assert_eq!(asset_name_for(kind, &version, os, arch).unwrap(), expected);
    }
    assert!(asset_name_for(CoreKind::Xray, &version, "windows", "x86_64").is_err());
}

#[test]
fn replaces_managed_directory_after_staging() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("xray");
    let staged = root.path().join("staged");
    fs::create_dir(&target).unwrap();
    fs::create_dir(&staged).unwrap();
    fs::write(target.join("xray"), "old").unwrap();
    fs::write(staged.join("xray"), "new").unwrap();

    replace_directory(&staged, &target).unwrap();

    assert_eq!(fs::read_to_string(target.join("xray")).unwrap(), "new");
    assert!(!staged.exists());
}

#[test]
fn restores_existing_directory_when_activation_fails() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("xray");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("xray"), "old").unwrap();

    assert!(replace_directory(&root.path().join("missing"), &target).is_err());

    assert_eq!(fs::read_to_string(target.join("xray")).unwrap(), "old");
}

#[cfg(unix)]
#[test]
fn leaves_unmanaged_cli_entry_untouched() {
    let root = tempfile::tempdir().unwrap();
    let managed_root = root.path().join("cores");
    let binary = managed_root.join("xray").join("xray");
    let bin_dir = root.path().join("bin");
    fs::create_dir_all(binary.parent().unwrap()).unwrap();
    fs::create_dir_all(&bin_dir).unwrap();
    fs::write(&binary, "managed").unwrap();
    fs::write(bin_dir.join("xray"), "external").unwrap();

    let warning = ensure_cli_link_in(CoreKind::Xray, &binary, &managed_root, &bin_dir)
        .unwrap()
        .expect("collision should warn");

    assert!(warning.contains("left existing"));
    assert_eq!(
        fs::read_to_string(bin_dir.join("xray")).unwrap(),
        "external"
    );
}

#[cfg(unix)]
#[test]
fn creates_cli_link_to_managed_binary() {
    let root = tempfile::tempdir().unwrap();
    let managed_root = root.path().join("cores");
    let binary = managed_root.join("xray").join("xray");
    let bin_dir = root.path().join("bin");
    fs::create_dir_all(binary.parent().unwrap()).unwrap();
    fs::write(&binary, "managed").unwrap();

    ensure_cli_link_in(CoreKind::Xray, &binary, &managed_root, &bin_dir).unwrap();

    assert_eq!(fs::read_link(bin_dir.join("xray")).unwrap(), binary);
}

#[test]
fn extracts_only_expected_xray_files() {
    let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default();
    for (name, value) in [
        ("xray", "binary"),
        ("geoip.dat", "ip"),
        ("geosite.dat", "site"),
        ("../escape", "bad"),
    ] {
        archive.start_file(name, options).unwrap();
        archive.write_all(value.as_bytes()).unwrap();
    }
    let bytes = archive.finish().unwrap().into_inner();
    let root = tempfile::tempdir().unwrap();

    extract_zip(CoreKind::Xray, &bytes, root.path()).unwrap();

    assert_eq!(
        fs::read_to_string(root.path().join("xray")).unwrap(),
        "binary"
    );
    assert!(!root.path().parent().unwrap().join("escape").exists());
}

#[test]
fn extracts_nested_sing_box_binary() {
    let encoder = GzEncoder::new(Vec::new(), Compression::default());
    let mut archive = tar::Builder::new(encoder);
    let contents = b"sing-box-binary";
    let mut header = tar::Header::new_gnu();
    header.set_size(contents.len() as u64);
    header.set_mode(0o755);
    header.set_cksum();
    archive
        .append_data(
            &mut header,
            "sing-box-1.0.0-linux-amd64/sing-box",
            &contents[..],
        )
        .unwrap();
    let encoder = archive.into_inner().unwrap();
    let bytes = encoder.finish().unwrap();
    let root = tempfile::tempdir().unwrap();

    extract_sing_box(&bytes, root.path()).unwrap();

    assert_eq!(fs::read(root.path().join("sing-box")).unwrap(), contents);
}
