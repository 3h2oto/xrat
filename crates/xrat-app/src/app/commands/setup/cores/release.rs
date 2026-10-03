use super::prelude::*;

pub(crate) async fn fetch_latest(
    client: &xrat_support::http::Client,
    kind: CoreKind,
) -> Result<CoreRelease, String> {
    fetch_release(client, kind, None, false).await
}

pub(crate) async fn fetch_release(
    client: &xrat_support::http::Client,
    kind: CoreKind,
    version: Option<&Version>,
    prerelease: bool,
) -> Result<CoreRelease, String> {
    let pinned = default_version(kind);
    let version = if version.is_none() && !prerelease {
        pinned.as_ref()
    } else {
        version
    };
    let selector = if prerelease {
        "latest prerelease".to_string()
    } else if let Some(version) = version {
        format!("v{version}")
    } else {
        "latest stable".to_string()
    };
    tracing::info!(core = kind.name(), %selector, "resolving proxy core release");
    let response = client
        .get(release_api_url(kind, version, prerelease))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err(format!("GitHub returned HTTP {}", response.status()));
    }
    let body = response.text().await.map_err(|error| error.to_string())?;
    let payload = if prerelease {
        let releases: Vec<GithubRelease> =
            serde_json::from_str(&body).map_err(|error| error.to_string())?;
        newest_prerelease(releases)
            .ok_or_else(|| format!("{} has no published prerelease", kind.repository()))?
    } else {
        serde_json::from_str(&body).map_err(|error| error.to_string())?
    };
    let release = release_from_payload(kind, payload)?;
    if let Some(version) = version
        && release.version != *version
    {
        return Err(format!(
            "GitHub returned v{} when v{version} was requested",
            release.version
        ));
    }
    tracing::info!(
        core = kind.name(),
        version = %release.version,
        tag = %release.tag,
        asset = %release.asset.name,
        %selector,
        "proxy core release resolved"
    );
    Ok(release)
}

/// The default release to install when the caller does not pin a version.
/// Only sing-box is pinned; Xray and V2Ray track latest stable.
pub(crate) fn default_version(kind: CoreKind) -> Option<Version> {
    (kind == CoreKind::SingBox).then(|| {
        Version::parse(PINNED_SINGBOX_VERSION)
            .expect("PINNED_SINGBOX_VERSION must be a valid semantic version")
    })
}

pub(crate) fn release_api_url(
    kind: CoreKind,
    version: Option<&Version>,
    prerelease: bool,
) -> String {
    if prerelease {
        return format!(
            "https://api.github.com/repos/{}/releases?per_page=100",
            kind.repository()
        );
    }
    let release = version
        .map(|version| format!("tags/v{version}"))
        .unwrap_or_else(|| "latest".to_string());
    format!(
        "https://api.github.com/repos/{}/releases/{release}",
        kind.repository()
    )
}

pub(crate) fn newest_prerelease(releases: Vec<GithubRelease>) -> Option<GithubRelease> {
    releases
        .into_iter()
        .filter(|release| release.prerelease)
        .max_by(|left, right| left.created_at.cmp(&right.created_at))
}

pub(crate) fn release_from_payload(
    kind: CoreKind,
    payload: GithubRelease,
) -> Result<CoreRelease, String> {
    let version = Version::parse(payload.tag_name.trim_start_matches('v'))
        .map_err(|error| format!("invalid release tag {}: {error}", payload.tag_name))?;
    let expected_name = asset_name(kind, &version)?;
    let asset = payload
        .assets
        .into_iter()
        .find(|asset| asset.name == expected_name)
        .ok_or_else(|| format!("release {} has no {expected_name}", payload.tag_name))?;
    let sha256 = parse_sha256(asset.digest.as_deref())?;
    Ok(CoreRelease {
        version,
        tag: payload.tag_name,
        asset: ReleaseAsset {
            name: asset.name,
            url: asset.browser_download_url,
            sha256,
        },
    })
}

pub(crate) fn asset_name(kind: CoreKind, version: &Version) -> Result<String, String> {
    asset_name_with_platform(kind, version, &platform::HostPlatformDetector)
}

fn asset_name_with_platform(
    kind: CoreKind,
    version: &Version,
    detector: &dyn platform::PlatformDetector,
) -> Result<String, String> {
    let host = detector.detect();
    asset_name_for(kind, version, host.os.as_str(), host.arch.as_str())
}

pub(crate) fn asset_name_for(
    kind: CoreKind,
    version: &Version,
    os: &str,
    arch: &str,
) -> Result<String, String> {
    match kind {
        CoreKind::Xray | CoreKind::V2Ray => {
            let prefix = if kind == CoreKind::Xray {
                "Xray"
            } else {
                "v2ray"
            };
            let os = match os {
                "linux" => "linux",
                "macos" => "macos",
                _ => {
                    return Err(format!(
                        "managed installation is unsupported on {os}/{arch}"
                    ));
                }
            };
            let arch = match arch {
                "x86_64" => "64",
                "aarch64" => "arm64-v8a",
                _ => {
                    return Err(format!(
                        "managed installation is unsupported on {os}/{arch}"
                    ));
                }
            };
            Ok(format!("{prefix}-{os}-{arch}.zip"))
        }
        CoreKind::SingBox => {
            let os = match os {
                "linux" => "linux",
                "macos" => "darwin",
                _ => {
                    return Err(format!(
                        "managed installation is unsupported on {os}/{arch}"
                    ));
                }
            };
            let arch = match arch {
                "x86_64" => "amd64",
                "aarch64" => "arm64",
                _ => {
                    return Err(format!(
                        "managed installation is unsupported on {os}/{arch}"
                    ));
                }
            };
            Ok(format!("sing-box-{version}-{os}-{arch}.tar.gz"))
        }
    }
}

pub(crate) fn parse_sha256(digest: Option<&str>) -> Result<String, String> {
    let digest = digest.ok_or_else(|| "release asset has no published digest".to_string())?;
    let value = digest
        .strip_prefix("sha256:")
        .ok_or_else(|| format!("unsupported release digest {digest:?}"))?;
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("release asset has an invalid SHA-256 digest".to_string());
    }
    Ok(value.to_ascii_lowercase())
}
