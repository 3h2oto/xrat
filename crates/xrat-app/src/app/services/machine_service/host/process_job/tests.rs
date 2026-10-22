use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const MODE: &str = "XRAT_SERVICE_JOB_TEST_MODE";
const RECEIPT: &str = "XRAT_SERVICE_JOB_TEST_RECEIPT";

struct ChildProcess(OwnedHandle);
impl Drop for ChildProcess {
    fn drop(&mut self) {
        unsafe {
            windows_sys::Win32::System::Threading::TerminateProcess(self.0.as_raw_handle(), 1)
        };
    }
}

fn child_guard(pid: u32, creation: u64) -> ChildProcess {
    use windows_sys::Win32::{
        Foundation::FILETIME,
        System::Threading::{
            GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE,
        },
    };
    let raw = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_TERMINATE,
            0,
            pid,
        )
    };
    assert!(!raw.is_null());
    let handle = unsafe { OwnedHandle::from_raw_handle(raw) };
    let (mut created, mut exited, mut kernel, mut user) = (
        FILETIME::default(),
        FILETIME::default(),
        FILETIME::default(),
        FILETIME::default(),
    );
    assert_ne!(
        unsafe {
            GetProcessTimes(
                handle.as_raw_handle(),
                &mut created,
                &mut exited,
                &mut kernel,
                &mut user,
            )
        },
        0
    );
    assert_eq!(
        u64::from(created.dwHighDateTime) << 32 | u64::from(created.dwLowDateTime),
        creation
    );
    ChildProcess(handle)
}

fn helper_name() -> String {
    format!("{}::subprocess", module_path!().split_once("::").unwrap().1)
}

struct Parent(Child);
impl Drop for Parent {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn abrupt_service_exit_terminates_detached_children() {
    let directory = tempfile::tempdir().unwrap();
    let receipt = directory.path().join("child.json");
    let mut parent = Parent(
        Command::new(std::env::current_exe().unwrap())
            .args(["--ignored", "--exact", &helper_name(), "--nocapture"])
            .env(MODE, "parent")
            .env(RECEIPT, &receipt)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(15);
    let (pid, creation): (u32, u64) = loop {
        if let Ok(bytes) = std::fs::read(&receipt)
            && let Ok(identity) = serde_json::from_slice(&bytes)
        {
            break identity;
        }
        assert!(
            parent.0.try_wait().unwrap().is_none(),
            "job parent exited before creating its child"
        );
        assert!(Instant::now() < deadline, "job child did not become ready");
        std::thread::sleep(Duration::from_millis(20));
    };
    let _child_cleanup = child_guard(pid, creation);
    assert_eq!(
        xrat_support::windows::wait_process_exit(pid, creation, Duration::from_millis(100))
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::TimedOut
    );
    parent.0.kill().unwrap();
    parent.0.wait().unwrap();
    xrat_support::windows::wait_process_exit(pid, creation, Duration::from_secs(5))
        .expect("closing the service process must terminate its detached child");
}

#[test]
#[ignore = "subprocess fixture for the service job lifetime test"]
fn subprocess() {
    match std::env::var(MODE).as_deref() {
        Ok("parent") => {
            super::contain_service_process().unwrap();
            super::contain_service_process().unwrap();
            let mut command = xrat_support::process::Command::new(std::env::current_exe().unwrap());
            command
                .args(["--ignored", "--exact", &helper_name(), "--nocapture"])
                .env(MODE, "child")
                .detached(true)
                .stdin(xrat_support::process::Stdio::Null)
                .stdout(xrat_support::process::Stdio::Null)
                .stderr(xrat_support::process::Stdio::Null);
            xrat_support::process::StartupChild::new(command.spawn().unwrap()).detach();
        }
        Ok("child") => {
            let pid = std::process::id();
            let creation = xrat_support::windows::process_creation_time(pid).unwrap();
            let path = PathBuf::from(std::env::var_os(RECEIPT).unwrap());
            std::fs::write(path, serde_json::to_vec(&(pid, creation)).unwrap()).unwrap();
        }
        _ => return,
    }
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}
