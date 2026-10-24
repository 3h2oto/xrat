use super::*;
use std::{
    sync::mpsc,
    time::{Duration, Instant},
};
use windows_sys::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE,
};

const PARENT: &str = "process::windows_detached::tests::detached_parent_helper";
const LEAF: &str = "process::windows_detached::tests::detached_leaf_helper";

#[test]
fn detached_child_does_not_keep_parent_output_pipes_open() {
    let directory = tempfile::Builder::new()
        .prefix("xrat inherited handles م ")
        .tempdir()
        .unwrap();
    let marker = directory.path().join("child identity.txt");
    let mut parent = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", PARENT, "--nocapture"])
        .arg(format!("marker={}", marker.display()))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = parent.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        if Instant::now() >= deadline {
            let _ = parent.kill();
            let _ = parent.wait();
            panic!("detached parent helper did not exit");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let identity = std::fs::read_to_string(marker).unwrap();
    let (pid, created) = identity.split_once(' ').unwrap();
    let pid = pid.parse::<u32>().unwrap();
    let created = created.parse::<u64>().unwrap();
    let handle = unsafe {
        OpenProcess(
            PROCESS_TERMINATE | PROCESS_QUERY_LIMITED_INFORMATION | 0x0010_0000,
            0,
            pid,
        )
    };
    assert!(!handle.is_null());
    let mut leaf = Process {
        handle: unsafe { OwnedHandle::from_raw_handle(handle) },
        id: pid,
    };
    assert_eq!(crate::windows::process_creation_time(pid).unwrap(), created);
    assert!(leaf.try_wait().unwrap().is_none());
    let (sender, receiver) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        let _ = sender.send(parent.wait_with_output());
    });
    let output = receiver.recv_timeout(Duration::from_secs(2));
    let alive = leaf.try_wait().unwrap().is_none();
    let _ = leaf.kill();
    let _ = leaf.wait();
    let blocked = output.is_err();
    let output = output
        .unwrap_or_else(|_| receiver.recv_timeout(Duration::from_secs(5)).unwrap())
        .unwrap();
    reader.join().unwrap();
    assert!(alive, "detached descendant must outlive its parent");
    assert!(
        !blocked,
        "detached child retained the exiting parent's captured handles"
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "subprocess helper for inherited-handle regression"]
fn detached_parent_helper() {
    let marker = std::env::args()
        .find_map(|arg| arg.strip_prefix("marker=").map(PathBuf::from))
        .unwrap();
    let child = Command::new(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", LEAF])
        .detached(true)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let created = crate::windows::process_creation_time(child.id()).unwrap();
    std::fs::write(marker, format!("{} {created}", child.id())).unwrap();
}

#[test]
#[ignore = "short-lived detached subprocess helper"]
fn detached_leaf_helper() {
    std::thread::sleep(Duration::from_secs(20));
}
