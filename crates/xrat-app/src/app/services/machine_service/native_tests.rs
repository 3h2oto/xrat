use super::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct InstallationGuard {
    profile: PathBuf,
    binaries: PathBuf,
}

impl Drop for InstallationGuard {
    fn drop(&mut self) {
        if let Err(error) = scm::uninstall() {
            eprintln!("service test cleanup failed; profile preserved: {error}");
            return;
        }
        for path in [&self.profile, &self.binaries] {
            if let Err(error) = std::fs::remove_dir_all(path)
                && error.kind() != std::io::ErrorKind::NotFound
            {
                eprintln!(
                    "service test cleanup failed for {}: {error}",
                    path.display()
                );
            }
        }
    }
}

fn command(binary: &Path, arguments: &[&str]) -> Output {
    let root = tempfile::tempdir().unwrap();
    let invalid = root.path().join("invalid.toml");
    std::fs::write(&invalid, "invalid = [").unwrap();
    let mut command = Command::new(binary);
    if !arguments.contains(&"machine") {
        command.arg("--config").arg(&invalid);
    }
    let output = command
        .args(arguments)
        .env("XRAT_PATH", root.path())
        .output()
        .expect("native service CLI must execute");
    assert_eq!(
        std::fs::read_dir(root.path()).unwrap().count(),
        1,
        "service commands must not initialize a user profile"
    );
    output
}

fn success(binary: &Path, arguments: &[&str]) {
    let output = command(binary, arguments);
    assert!(
        output.status.success(),
        "{arguments:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

async fn ping() {
    let response = request(GatewayRequest {
        protocol_version: GATEWAY_VERSION,
        request: GatewayRequestKind::Ping {},
    })
    .await
    .expect("operator must reach SYSTEM gateway");
    assert!(response.ok, "{}", response.message);
    assert!(
        matches!(response.payload, Some(GatewayPayload::Ping(payload)) if payload.daemon_ready)
    );
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires XRAT_WINDOWS_SERVICE_BINARY and an isolated elevated native Windows host; changes SCM and ProgramData/Program Files"]
async fn native_machine_service_lifecycle_and_restricted_operator() {
    let binary = PathBuf::from(
        std::env::var_os("XRAT_WINDOWS_SERVICE_BINARY").expect("set a built native xrat.exe path"),
    );
    assert!(binary.is_absolute() && binary.is_file());
    assert!(
        scm::status().unwrap().is_none(),
        "refusing to touch a pre-existing service"
    );
    let profile = xrat_support::windows::program_data()
        .unwrap()
        .join("XRAT-Service");
    let binaries = xrat_support::windows::program_files()
        .unwrap()
        .join("XRAT-Service");
    assert!(
        !profile.exists() && !binaries.exists(),
        "test requires a clean isolated machine profile"
    );
    success(&binary, &["daemon", "install", "--dry-run", "--start"]);
    assert!(!profile.exists() && !binaries.exists());
    assert!(scm::status().unwrap().is_none());
    let guard = InstallationGuard { profile, binaries };
    let operator = xrat_support::windows::current_user_sid().unwrap();
    success(
        &binary,
        &["daemon", "install", "--operator-sid", &operator, "--start"],
    );
    ping().await;
    success(&binary, &["--profile", "machine", "daemon", "status"]);
    success(&binary, &["--profile", "machine", "daemon", "restart"]);
    ping().await;
    let config_path = guard.profile.join("config.toml");
    let registration = std::fs::read(guard.profile.join("operator.json")).unwrap();
    let config = b"# retained administrator configuration\n";
    std::fs::write(&config_path, config).unwrap();
    {
        let impersonation =
            crate::app::daemon::ipc::windows::tests::impersonate_owner_without_admin().unwrap();
        assert!(
            std::fs::File::open(&config_path).is_err(),
            "ordinary operator must not read the protected profile directly"
        );
        assert!(scm::open(windows_sys::Win32::System::Services::SERVICE_CHANGE_CONFIG).is_err());
        assert!(
            scm::open(0x0004_0000).is_err(),
            "ordinary operator must not change service ACLs through ownership"
        );
        ping().await;
        scm::stop().expect("ordinary operator must be able to stop the service");
        scm::start().expect("ordinary operator must be able to start the service");
        ping().await;
        drop(impersonation);
    }
    success(
        &binary,
        &["daemon", "install", "--operator-sid", &operator, "--start"],
    );
    assert_eq!(std::fs::read(&config_path).unwrap(), config);
    assert_eq!(
        std::fs::read(guard.profile.join("operator.json")).unwrap(),
        registration
    );
    assert!(
        !command(
            &binary,
            &[
                "daemon",
                "install",
                "--operator-sid",
                "S-1-5-21-111-222-333-9999"
            ]
        )
        .status
        .success()
    );
    ping().await;
    success(&binary, &["daemon", "uninstall", "--dry-run"]);
    ping().await;
    success(&binary, &["daemon", "uninstall"]);
    success(&binary, &["daemon", "uninstall"]);
    assert!(scm::status().unwrap().is_none());
    assert_eq!(std::fs::read(&config_path).unwrap(), config);
    assert_eq!(
        std::fs::read(guard.profile.join("operator.json")).unwrap(),
        registration
    );
}
