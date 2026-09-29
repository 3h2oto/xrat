pub(crate) use std::fs;
pub(crate) use std::io::{Cursor, Write};
pub(crate) use std::path::{Path, PathBuf};
pub(crate) use std::process::Command;
pub(crate) use std::time::Duration;

pub(crate) use flate2::read::GzDecoder;
pub(crate) use semver::Version;
pub(crate) use serde::Deserialize;
pub(crate) use sha2::{Digest, Sha256};

pub(crate) use crate::app::commands::progress::CliProgress;
pub(crate) use crate::app::config;
pub(crate) use crate::app::context::AppContext;
pub(crate) use xrat_support::platform;

pub(crate) const CORE_KINDS: [CoreKind; 3] = [CoreKind::Xray, CoreKind::SingBox, CoreKind::V2Ray];

/// Managed sing-box installs track the pinned conformance release so the
/// installed binary matches the v1.13.21 validation target. The runtime still
/// accepts any stable 1.13.x binary; this only controls what setup installs.
pub(crate) const PINNED_SINGBOX_VERSION: &str = "1.13.21";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CoreKind {
    Xray,
    SingBox,
    V2Ray,
}

impl CoreKind {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Xray => "xray",
            Self::SingBox => "sing-box",
            Self::V2Ray => "v2ray",
        }
    }

    pub(crate) fn config_key(self) -> &'static str {
        match self {
            Self::Xray => "xray",
            Self::SingBox => "sing_box",
            Self::V2Ray => "v2ray",
        }
    }

    pub(crate) fn repository(self) -> &'static str {
        match self {
            Self::Xray => "XTLS/Xray-core",
            Self::SingBox => "SagerNet/sing-box",
            Self::V2Ray => "v2fly/v2ray-core",
        }
    }

    pub(crate) fn required(self) -> bool {
        self == Self::Xray
    }

    pub(crate) fn unattended_default(self) -> bool {
        matches!(self, Self::Xray | Self::SingBox)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CoreRelease {
    pub(crate) version: Version,
    pub(crate) tag: String,
    pub(crate) asset: ReleaseAsset,
}

#[derive(Clone, Debug)]
pub(crate) struct ReleaseAsset {
    pub(crate) name: String,
    pub(crate) url: String,
    pub(crate) sha256: String,
}

#[derive(Debug)]
pub(crate) struct CoreProbe {
    pub(crate) kind: CoreKind,
    pub(crate) path: Option<PathBuf>,
    pub(crate) version: Option<Version>,
    pub(crate) managed: bool,
    pub(crate) latest: Result<CoreRelease, String>,
}

impl CoreProbe {
    pub(crate) fn missing(&self) -> bool {
        self.path.is_none()
    }

    pub(crate) fn outdated(&self) -> bool {
        matches!((&self.version, &self.latest), (Some(current), Ok(latest)) if current < &latest.version)
    }

    pub(crate) fn detail(&self) -> String {
        let location = self
            .path
            .as_deref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "not installed".to_string());
        let ownership = if self.managed { "managed" } else { "external" };
        let channel = if self.kind == CoreKind::SingBox {
            "pinned"
        } else {
            "latest"
        };
        let current = self
            .version
            .as_ref()
            .map(|version| format!("v{version}"))
            .unwrap_or_else(|| "version unknown".to_string());
        match &self.latest {
            Ok(latest) if self.path.is_some() => {
                format!(
                    "{location} ({current}; {channel} v{}; {ownership})",
                    latest.version
                )
            }
            Ok(latest) => format!("{location} ({channel} v{})", latest.version),
            Err(error) if self.path.is_some() => {
                format!("{location} ({current}; {ownership}; update check failed: {error})")
            }
            Err(error) => format!("{location} (update check failed: {error})"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct InstallResult {
    pub(crate) binary_path: PathBuf,
    pub(crate) version: Version,
    pub(crate) cli_link_warning: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct GithubRelease {
    pub(crate) tag_name: String,
    pub(crate) prerelease: bool,
    pub(crate) created_at: String,
    pub(crate) assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
pub(crate) struct GithubAsset {
    pub(crate) name: String,
    pub(crate) browser_download_url: String,
    pub(crate) digest: Option<String>,
}
