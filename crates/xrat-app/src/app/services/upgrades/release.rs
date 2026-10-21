use std::io::Write;
use std::path::Path;
use std::time::Duration;

use crate::app::AppError;
use crate::app::commands::output;
use crate::app::commands::progress::CliProgress;
use crate::app::services::upgrades::UpgradeOptions as UpgradeArgs;

use super::{REPO, current_version, same_version};
#[cfg(not(windows))]
use super::{
    install_binary, restart_daemon_if_active, run_post_upgrade_migrations,
    run_post_upgrade_tun_setup_with_spawner, should_restart_daemon, should_restore_tun,
};

pub(crate) async fn upgrade(
    args: &UpgradeArgs,
    target: &Path,
    config_path: &Path,
) -> crate::app::Result<super::UpgradeOutcome> {
    let color = output::color_enabled();

    let version = match &args.version {
        Some(tag) => tag.clone(),
        None => {
            let progress = CliProgress::spinner(true, "checking latest release");
            let result = fetch_latest_tag(args.timeout_secs).await;
            progress.finish_and_clear();
            result?
        }
    };

    if !args.force && same_version(&version, current_version()) {
        println!(
            "{}",
            output::success(
                format!("already using latest version ({})", current_version()),
                color
            )
        );
        return Ok(super::UpgradeOutcome::Current);
    }

    let arch = detect_arch()?;

    println!(
        "{}",
        output::notice(
            format!("upgrading {} -> {version}", current_version()),
            color
        )
    );

    let extension = if arch.contains("windows") {
        "zip"
    } else {
        "tar.gz"
    };
    let filename = format!("xrat-{version}-{arch}.{extension}");
    let base_url = format!("https://github.com/{REPO}/releases/download/{version}");

    let work = tempfile::tempdir()?;
    let client = http_client(args.timeout_secs)?;

    download_with_progress(
        &client,
        &format!("{base_url}/{filename}"),
        &work.path().join(&filename),
        &filename,
    )
    .await?;

    let progress = CliProgress::spinner(true, "verifying checksum");
    let result = async {
        download(
            &client,
            &format!("{base_url}/SHASUMS256.txt"),
            &work.path().join("SHASUMS256.txt"),
        )
        .await?;
        verify_checksum(work.path(), &filename)
    }
    .await;
    progress.finish_and_clear();
    result?;

    #[cfg(not(windows))]
    let restore_tun = should_restore_tun(target, config_path, args);

    #[cfg(windows)]
    {
        extract_binary(work.path(), &filename)?;
        let transaction = super::windows::stage(
            &work.path().join("xrat.exe"),
            target,
            config_path,
            &version,
            args,
        )
        .await?;
        Ok(super::UpgradeOutcome::Staged(transaction))
    }

    #[cfg(not(windows))]
    {
        let progress = CliProgress::spinner(true, format!("installing to {}", target.display()));
        let result = (|| {
            extract_binary(work.path(), &filename)?;
            install_binary(
                &work
                    .path()
                    .join(if cfg!(windows) { "xrat.exe" } else { "xrat" }),
                target,
            )
        })();
        progress.finish_and_clear();
        result?;

        let progress = CliProgress::spinner(true, "applying database migrations");
        let result = run_post_upgrade_migrations(target, config_path);
        progress.finish_and_clear();
        result?;

        if restore_tun {
            let spawner = std::sync::Arc::new(xrat_support::process::SystemProcessSpawner);
            let tun_ok =
                run_post_upgrade_tun_setup_with_spawner(target, config_path, spawner.clone());
            if tun_ok && should_restart_daemon(config_path, args) {
                restart_daemon_if_active(target, config_path, spawner).await;
            }
        }

        crate::app::services::release_notes::cache_release(&client, &version).await;

        println!(
            "{}",
            output::success(
                format!("upgraded xrat {} -> {version}", current_version()),
                color
            )
        );

        Ok(super::UpgradeOutcome::Completed)
    }
}

fn detect_arch() -> crate::app::Result<&'static str> {
    detect_arch_with_platform(&xrat_support::platform::HostPlatformDetector)
}

fn detect_arch_with_platform(
    detector: &dyn xrat_support::platform::PlatformDetector,
) -> crate::app::Result<&'static str> {
    let platform = detector.detect();
    match (platform.os.as_str(), platform.arch.as_str()) {
        ("linux", "x86_64") => Ok("x86_64-unknown-linux-musl"),
        ("linux", "aarch64") => Ok("aarch64-unknown-linux-musl"),
        ("macos", "x86_64") => Ok("x86_64-apple-darwin"),
        ("macos", "aarch64") => Ok("aarch64-apple-darwin"),
        ("windows", "x86_64") => Ok("x86_64-pc-windows-msvc"),
        ("windows", "aarch64") => Ok("aarch64-pc-windows-msvc"),
        (os, arch) => Err(AppError::UnsupportedPlatform(format!(
            "no prebuilt release for {os}/{arch}; use --source to build from source"
        ))),
    }
}

fn http_client(timeout_secs: u64) -> crate::app::Result<xrat_support::http::Client> {
    Ok(xrat_support::http::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .user_agent(concat!("xrat/", env!("CARGO_PKG_VERSION")))
        .build()?)
}

async fn fetch_latest_tag(timeout_secs: u64) -> crate::app::Result<String> {
    crate::app::services::releases::ReleaseService::default()
        .latest_tag(timeout_secs)
        .await
}

async fn download(
    client: &xrat_support::http::Client,
    url: &str,
    destination: &Path,
) -> crate::app::Result<()> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|source| AppError::ReleaseHttp {
            operation: "downloading xrat release checksums from github.com",
            source,
        })?;
    if !response.status().is_success() {
        return Err(AppError::InvalidArgument(format!(
            "download failed for {url}: HTTP {}",
            response.status()
        )));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|source| AppError::ReleaseHttp {
            operation: "reading xrat release checksums from github.com",
            source,
        })?;
    std::fs::write(destination, &bytes)?;
    Ok(())
}

async fn download_with_progress(
    client: &xrat_support::http::Client,
    url: &str,
    destination: &Path,
    label: &str,
) -> crate::app::Result<()> {
    let mut response = client
        .get(url)
        .send()
        .await
        .map_err(|source| AppError::ReleaseHttp {
            operation: "downloading xrat release archive from github.com",
            source,
        })?;
    if !response.status().is_success() {
        return Err(AppError::InvalidArgument(format!(
            "download failed for {url}: HTTP {}",
            response.status()
        )));
    }

    let progress = CliProgress::bytes_bar(
        true,
        response.content_length(),
        format!("downloading {label}"),
    );
    let mut file = std::fs::File::create(destination)?;
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|source| AppError::ReleaseHttp {
            operation: "reading xrat release archive from github.com",
            source,
        })?
    {
        file.write_all(&chunk)?;
        progress.inc(chunk.len() as u64);
    }
    file.flush()?;
    progress.finish_with_message(format!("{label} done"));
    Ok(())
}

fn verify_checksum(work_dir: &Path, filename: &str) -> crate::app::Result<()> {
    super::archive::verify_checksum(
        &work_dir.join(filename),
        &std::fs::read_to_string(work_dir.join("SHASUMS256.txt"))?,
        filename,
    )
}

fn extract_binary(work_dir: &Path, filename: &str) -> crate::app::Result<()> {
    super::archive::extract_binary(
        &work_dir.join(filename),
        work_dir,
        if filename.ends_with(".zip") {
            "xrat.exe"
        } else {
            "xrat"
        },
    )
}

#[cfg(test)]
mod platform_tests {
    use super::*;
    use xrat_support::platform::{Architecture, OperatingSystem, Platform};
    #[test]
    fn release_targets_use_injected_platform_and_reject_unsupported_pairs() {
        for (os, arch, target) in [
            (
                OperatingSystem::Linux,
                Architecture::X86_64,
                "x86_64-unknown-linux-musl",
            ),
            (
                OperatingSystem::Linux,
                Architecture::Aarch64,
                "aarch64-unknown-linux-musl",
            ),
            (
                OperatingSystem::Macos,
                Architecture::X86_64,
                "x86_64-apple-darwin",
            ),
            (
                OperatingSystem::Macos,
                Architecture::Aarch64,
                "aarch64-apple-darwin",
            ),
            (
                OperatingSystem::Windows,
                Architecture::X86_64,
                "x86_64-pc-windows-msvc",
            ),
            (
                OperatingSystem::Windows,
                Architecture::Aarch64,
                "aarch64-pc-windows-msvc",
            ),
        ] {
            assert_eq!(
                detect_arch_with_platform(&Platform { os, arch }).unwrap(),
                target
            );
        }
        assert!(
            detect_arch_with_platform(&Platform {
                os: OperatingSystem::Windows,
                arch: Architecture::Other("i686")
            })
            .is_err()
        );
    }
}
