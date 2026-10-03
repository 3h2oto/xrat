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
