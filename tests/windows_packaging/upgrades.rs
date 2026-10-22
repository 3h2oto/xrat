use super::fixture::Fixture;
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Seek},
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

fn digest(path: &Path) -> String {
    format!("{:x}", Sha256::digest(std::fs::read(path).unwrap()))
}

pub fn exercise(fixture: &Fixture) {
    let version = fixture.xrat(&["--version"]).trim().to_string();
    for (running, restart) in [(false, true), (true, true), (true, false)] {
        for tamper in [false, true] {
            exercise_case(fixture, &version, running, restart, tamper);
        }
    }
}

fn exercise_case(fixture: &Fixture, version: &str, running: bool, restart: bool, tamper: bool) {
    let install = tempfile::Builder::new()
        .prefix("upgrade م ")
        .tempdir_in(fixture.directory.path())
        .unwrap();
    let work = install.path().join(".xrat-upgrade-fixture");
    std::fs::create_dir(&work).unwrap();
    let target = install.path().join("xrat.exe");
    let next = work.join("next.exe");
    let helper = work.join("updater.exe");
    for path in [&target, &next, &helper] {
        std::fs::copy(&fixture.executable, path).unwrap();
    }
    let target = target.canonicalize().unwrap();
    let work = work.canonicalize().unwrap();
    let original = digest(&target);
    if running {
        let output = Command::new(&target)
            .args(["daemon", "start"])
            .env("XRAT_PATH", &fixture.profile)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let manifest = work.join("transaction.json");
    let transaction = serde_json::json!({
        "target":target,"config":fixture.config,"parent_pid":std::process::id(),"parent_creation":0,
        "digest":digest(&next),"version":version.strip_prefix("xrat ").unwrap(),
        "state":"staged","message":"native acceptance fixture","restart_daemon":restart
    });
    std::fs::write(&manifest, serde_json::to_vec(&transaction).unwrap()).unwrap();
    std::fs::write(
        install.path().join("xrat-upgrade.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    if tamper {
        std::fs::write(&next, "tampered executable").unwrap();
    }
    let mut log = tempfile::tempfile().unwrap();
    let mut child = Command::new(&helper)
        .args(["upgrade", "--apply-transaction"])
        .arg(&manifest)
        .env("XRAT_PATH", &fixture.profile)
        .stdin(Stdio::null())
        .stdout(log.try_clone().unwrap())
        .stderr(log.try_clone().unwrap())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("upgrade helper timed out");
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    let result: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&manifest).unwrap()).unwrap();
    log.rewind().unwrap();
    let mut diagnostic = String::new();
    log.read_to_string(&mut diagnostic).unwrap();
    if tamper {
        assert!(!status.success());
        assert_eq!(result["state"], "failed", "{diagnostic}");
        assert_eq!(digest(&target), original);
    } else {
        assert!(status.success(), "{result}\n{diagnostic}");
        assert_eq!(result["state"], "completed");
        let output = Command::new(&target)
            .arg("--version")
            .env("XRAT_PATH", &fixture.profile)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), version);
        let output = Command::new(&target)
            .args(["upgrade", "--status"])
            .env("XRAT_PATH", &fixture.profile)
            .output()
            .unwrap();
        assert!(output.status.success());
        let status: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(status["state"], "completed");
        assert!(!work.join("backup.exe").exists());
        assert!(!helper.exists());
    }
    let status = Command::new(&target)
        .args(["status", "--json"])
        .env("XRAT_PATH", &fixture.profile)
        .output()
        .unwrap();
    assert_eq!(
        status.status.success(),
        running && (restart || tamper),
        "{}",
        String::from_utf8_lossy(&status.stderr)
    );
    fixture.xrat(&["daemon", "stop"]);
}
