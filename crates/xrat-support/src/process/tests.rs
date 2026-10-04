use super::*;
use std::os::unix::process::ExitStatusExt;
use std::sync::Mutex;

struct FakeSpawner(Mutex<Vec<(OsString, Vec<OsString>)>>);
#[async_trait]
impl ProcessSpawner for FakeSpawner {
    fn spawn(&self, _spec: &CommandSpec) -> io::Result<Child> {
        Err(io::Error::new(io::ErrorKind::NotFound, "missing fixture"))
    }
    fn run(&self, spec: &CommandSpec, capture: bool) -> io::Result<Output> {
        self.0
            .lock()
            .unwrap()
            .push((spec.program.clone(), spec.args.clone()));
        assert_eq!(
            spec.env,
            [(OsString::from("FIXTURE"), OsString::from("value"))]
        );
        assert_eq!(spec.current_dir.as_deref(), Some(Path::new("/fixture")));
        Ok(Output {
            status: ExitStatus::from_raw(7 << 8),
            stdout: if capture {
                b"output".to_vec()
            } else {
                Vec::new()
            },
            stderr: b"diagnostic".to_vec(),
        })
    }
    async fn output_async(&self, spec: &CommandSpec) -> io::Result<Output> {
        self.run(spec, true)
    }
}
#[tokio::test]
async fn fake_command_captures_spec_and_nonzero_status_without_running_binary() {
    let spawner = Arc::new(FakeSpawner(Mutex::new(Vec::new())));
    let mut command = Command::with_spawner("not-installed", spawner.clone());
    command
        .args(["argument", "with spaces"])
        .env("FIXTURE", "value")
        .current_dir("/fixture");
    let output = command.output_async().await.unwrap();
    assert_eq!(output.status.code(), Some(7));
    assert_eq!(output.stdout, b"output");
    assert_eq!(output.stderr, b"diagnostic");
    assert_eq!(
        spawner.0.lock().unwrap()[0].1,
        [OsString::from("argument"), OsString::from("with spaces")]
    );
    assert_eq!(
        command.spawn().err().unwrap().kind(),
        io::ErrorKind::NotFound
    );
}
#[test]
fn production_capture_drains_both_pipes_and_preserves_exit_status() {
    let output = Command::new("sh")
        .args([
            "-c",
            "head -c 131072 /dev/zero; head -c 131072 /dev/zero >&2; exit 7",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(7));
    assert_eq!(output.stdout.len(), 131072);
    assert_eq!(output.stderr.len(), 131072);
}
#[test]
fn production_child_can_be_inspected_terminated_and_reaped() {
    let mut child = Command::new("sleep").arg("30").spawn().unwrap();
    assert!(child.id() > 0);
    assert!(child.try_wait().unwrap().is_none());
    child.kill().unwrap();
    assert!(!child.wait().unwrap().success());
}

#[tokio::test]
async fn production_async_capture_drains_both_pipes() {
    let output = Command::new("sh")
        .args([
            "-c",
            "head -c 131072 /dev/zero; head -c 131072 /dev/zero >&2; exit 7",
        ])
        .output_async()
        .await
        .unwrap();
    assert_eq!(output.status.code(), Some(7));
    assert_eq!((output.stdout.len(), output.stderr.len()), (131072, 131072));
}

#[tokio::test]
async fn cancelling_async_output_with_kill_on_drop_terminates_the_local_child() {
    let root = tempfile::tempdir().unwrap();
    let pid_path = root.path().join("pid");
    let child_path = pid_path.clone();
    let task = tokio::spawn(async move {
        Command::new("sh")
            .args(["-c", "echo $$ > \"$1\"; exec sleep 30", "fixture"])
            .arg(child_path)
            .kill_on_drop(true)
            .output_async()
            .await
    });
    let pid = tokio::time::timeout(std::time::Duration::from_secs(2), async {
        loop {
            if let Ok(value) = std::fs::read_to_string(&pid_path)
                && let Ok(pid) = value.trim().parse::<i64>()
            {
                break pid;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    let signals = crate::signals::SystemProcessSignals::default();
    use crate::signals::ProcessSignals;
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        while signals.is_running(pid) {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap_or_else(|_| {
        panic!(
            "cancelled child {pid} remains: {:?}",
            std::fs::read_to_string(format!("/proc/{pid}/status"))
        )
    });
}

#[tokio::test]
async fn cancelling_async_output_without_kill_on_drop_preserves_child_and_reaps_its_exit() {
    let root = tempfile::tempdir().unwrap();
    let pid_path = root.path().join("pid");
    let complete_path = root.path().join("completed");
    let child_pid_path = pid_path.clone();
    let child_complete_path = complete_path.clone();
    let task = tokio::spawn(async move {
        Command::new("sh")
            .args([
                "-c",
                "echo $$ > \"$1\"; sleep 0.2; printf completed > \"$2\"",
                "fixture",
            ])
            .arg(child_pid_path)
            .arg(child_complete_path)
            .output_async()
            .await
    });
    let pid = tokio::time::timeout(std::time::Duration::from_secs(2), async {
        loop {
            if let Ok(value) = std::fs::read_to_string(&pid_path)
                && let Ok(pid) = value.trim().parse::<i64>()
            {
                break pid;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    use crate::signals::ProcessSignals;
    let signals = crate::signals::SystemProcessSignals::default();
    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        while !complete_path.exists() || signals.is_running(pid) {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("detached output must preserve the child and reap its eventual exit");
    assert_eq!(std::fs::read_to_string(complete_path).unwrap(), "completed");
}
