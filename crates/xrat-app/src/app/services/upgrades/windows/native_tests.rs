use super::*;
use crate::app::services::machine_service::{MachineProfile, api_task, scm};
use std::os::windows::fs::OpenOptionsExt;
use windows_sys::Win32::{
    Storage::FileSystem::FILE_SHARE_READ,
    System::Services::{SERVICE_RUNNING, SERVICE_STOPPED},
};

pub(crate) async fn exercise_machine_upgrade(binary: &Path) {
    let profile = MachineProfile::open().unwrap();
    let target = coordination::machine_target()
        .unwrap()
        .canonicalize()
        .unwrap();
    for (running, restart, failure) in [
        (true, true, None),
        (false, true, None),
        (true, false, None),
        (true, true, Some("tamper")),
        (true, true, Some("locked")),
        (true, true, Some("migration")),
    ] {
        if running {
            scm::start().unwrap();
        } else {
            scm::stop().unwrap();
        }
        let task_state = api_task::inspect(profile.operator_sid()).unwrap();
        let original = digest(&target).unwrap();
        let work = Work::new(target.parent().unwrap(), true, ".xrat-upgrade-").unwrap();
        let next = work.path().join("next.exe");
        let helper = work.path().join("updater.exe");
        copy_file(binary, &next).unwrap();
        copy_file(binary, &helper).unwrap();
        let manifest = work.path().join("transaction.json");
        let transaction = Transaction {
            machine: true,
            was_running: false,
            daemon_endpoint: None,
            api_enabled: false,
            api_running: false,
            coordination_started: false,
            binary_installed: false,
            target: target.clone(),
            config: profile.config_path(),
            parent_pid: std::process::id(),
            parent_creation: 0,
            digest: digest(&next).unwrap(),
            version: super::super::current_version().into(),
            state: State::Staged,
            message: "native machine upgrade fixture".into(),
            restart_daemon: restart,
        };
        save(&manifest, &transaction).unwrap();
        if failure == Some("tamper") {
            fs::OpenOptions::new()
                .append(true)
                .open(&next)
                .unwrap()
                .write_all(b"tamper")
                .unwrap();
        }
        let locked = (failure == Some("locked")).then(|| {
            fs::OpenOptions::new()
                .read(true)
                .share_mode(FILE_SHARE_READ)
                .open(&target)
                .unwrap()
        });
        let original_config = fs::read(profile.config_path()).unwrap();
        if failure == Some("migration") {
            fs::write(profile.config_path(), "invalid = [").unwrap();
        }
        let mut child = std::process::Command::new(helper)
            .args(["upgrade", "--apply-transaction"])
            .arg(&manifest)
            .spawn()
            .unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(90);
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if std::time::Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("machine upgrade helper timed out");
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        };
        drop(locked);
        fs::write(profile.config_path(), &original_config).unwrap();
        let result: Transaction = read_json(&manifest).unwrap();
        assert_eq!(status.success(), failure.is_none(), "{}", result.message);
        if failure == Some("migration") {
            assert!(matches!(result.state, State::Failed));
            assert!(result.binary_installed);
            assert!(work.path().join("backup.exe").exists());
            assert!(result.message.contains("migrations may have run"));
            api_task::resume(profile.operator_sid(), task_state.0, task_state.1).unwrap();
            scm::start().unwrap();
            continue;
        }
        assert!(
            matches!(
                (&result.state, failure),
                (State::Completed, None)
                    | (State::Failed, Some("tamper"))
                    | (State::RolledBack, Some("locked"))
            ),
            "{result:?}"
        );
        assert_eq!(
            scm::status().unwrap().unwrap().dwCurrentState,
            if running && (restart || failure.is_some()) {
                SERVICE_RUNNING
            } else {
                SERVICE_STOPPED
            }
        );
        assert_eq!(
            api_task::inspect(profile.operator_sid()).unwrap(),
            (task_state.0, task_state.1 && (restart || failure.is_some()))
        );
        assert_eq!(digest(&target).unwrap(), original);
        coordination::open_directory(target.parent().unwrap())
            .unwrap()
            .verify_tree()
            .unwrap();
    }
}
