use std::path::Path;
use xrat_support::process::Command;

use crate::app::AppError;
use crate::app::commands::output;
use crate::app::commands::progress::CliProgress;
use crate::cli::UpgradeArgs;

use super::{install_binary, run_post_upgrade_migrations_with_spawner};

pub(crate) async fn upgrade(
    args: &UpgradeArgs,
    target: &Path,
    config_path: &Path,
) -> crate::app::Result<()> {
    upgrade_with_spawner(
        args,
        target,
        config_path,
        std::sync::Arc::new(xrat_support::process::SystemProcessSpawner),
    )
    .await
}

pub(crate) async fn upgrade_with_spawner(
    args: &UpgradeArgs,
    target: &Path,
    config_path: &Path,
    spawner: std::sync::Arc<dyn xrat_support::process::ProcessSpawner>,
) -> crate::app::Result<()> {
    let color = output::color_enabled();
    let source_dir = &args.path;
    if !source_dir.join("Cargo.toml").is_file() {
        return Err(AppError::InvalidArgument(format!(
            "no Cargo.toml in {}; pass --path <dir> pointing at the xrat checkout",
            source_dir.display()
        )));
    }

    println!(
        "{}",
        output::notice(
            format!(
                "building from {} (cargo build --release)",
                source_dir.display()
            ),
            color
        )
    );
    let status = Command::with_spawner("cargo", spawner.clone())
        .args(["build", "--release"])
        .current_dir(source_dir)
        .status()
        .map_err(|error| AppError::InvalidArgument(format!("cannot run cargo: {error}")))?;
    if !status.success() {
        return Err(AppError::InvalidArgument(format!(
            "cargo build --release failed ({status})"
        )));
    }

    let built = source_dir.join("target").join("release").join("xrat");
    if !built.is_file() {
        return Err(AppError::InvalidArgument(format!(
            "build succeeded but {} is missing",
            built.display()
        )));
    }

    let progress = CliProgress::spinner(true, format!("installing to {}", target.display()));
    let result = install_binary(&built, target);
    progress.finish_and_clear();
    result?;

    let progress = CliProgress::spinner(true, "applying database migrations");
    let result = run_post_upgrade_migrations_with_spawner(target, config_path, spawner.clone());
    progress.finish_and_clear();
    result?;

    println!("{}", output::success("upgraded xrat from source", color));

    Ok(())
}

#[cfg(all(test, unix))]
mod process_tests {
    use super::*;
    use async_trait::async_trait;
    use std::io;
    use std::os::unix::process::ExitStatusExt;
    use std::process::{ExitStatus, Output};
    use std::sync::{Arc, Mutex};
    use xrat_support::process::{Child, CommandSpec, ProcessSpawner};

    struct UpgradeSpawner {
        calls: Mutex<Vec<(std::ffi::OsString, Vec<std::ffi::OsString>)>>,
        fail_build: bool,
    }
    #[async_trait]
    impl ProcessSpawner for UpgradeSpawner {
        fn spawn(&self, _: &CommandSpec) -> io::Result<Child> {
            panic!("upgrade uses status")
        }
        fn run(&self, spec: &CommandSpec, capture: bool) -> io::Result<Output> {
            assert!(!capture);
            self.calls
                .lock()
                .unwrap()
                .push((spec.program.clone(), spec.args.clone()));
            if spec.program == "cargo" {
                assert!(spec.current_dir.is_some());
            }
            Ok(Output {
                status: ExitStatus::from_raw(if self.fail_build { 7 << 8 } else { 0 }),
                stdout: Vec::new(),
                stderr: Vec::new(),
            })
        }
        async fn output_async(&self, _: &CommandSpec) -> io::Result<Output> {
            panic!("upgrade uses status")
        }
    }
    #[tokio::test]
    async fn injected_source_upgrade_carries_spawner_into_post_install_migrations() {
        for fail_build in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let source = root.path().join("source");
            std::fs::create_dir_all(source.join("target/release")).unwrap();
            std::fs::write(source.join("Cargo.toml"), "fixture").unwrap();
            std::fs::write(source.join("target/release/xrat"), "new fixture").unwrap();
            let target = root.path().join("xrat");
            std::fs::write(&target, "old fixture").unwrap();
            let config = root.path().join("config.toml");
            let args = UpgradeArgs {
                source: true,
                path: source,
                ..Default::default()
            };
            let spawner = Arc::new(UpgradeSpawner {
                calls: Mutex::new(Vec::new()),
                fail_build,
            });
            let result = upgrade_with_spawner(&args, &target, &config, spawner.clone()).await;
            let calls = spawner.calls.lock().unwrap();
            if fail_build {
                assert!(
                    result
                        .unwrap_err()
                        .to_string()
                        .contains("cargo build --release failed")
                );
                assert_eq!(calls.len(), 1);
                assert_eq!(std::fs::read_to_string(&target).unwrap(), "old fixture");
            } else {
                result.unwrap();
                assert_eq!(calls.len(), 2);
                assert_eq!(calls[1].0, target.as_os_str());
                assert_eq!(
                    calls[1].1,
                    vec![
                        std::ffi::OsString::from("--config"),
                        config.into_os_string(),
                        "db".into(),
                        "migrate".into()
                    ]
                );
                assert_eq!(std::fs::read_to_string(&target).unwrap(), "new fixture");
            }
        }
    }
}
