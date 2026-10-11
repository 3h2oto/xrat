use super::*;
use std::process::{Child, Command, Stdio};

#[test]
fn inspects_loopback_without_treating_it_as_owned_tun() {
    let info = inspect("lo0").unwrap().unwrap();
    assert!(info.ifindex > 0);
    assert!(!info.is_tun);
    assert!(wait_for_release("lo0", info.ifindex).is_err());
    assert_eq!(inspect("xrat-missing").unwrap(), None);
}

struct Engine(Child);
impl Drop for Engine {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn dns_snapshot() -> Vec<u8> {
    let output = Command::new("/usr/sbin/scutil")
        .arg("--dns")
        .output()
        .unwrap();
    assert!(output.status.success());
    output.stdout
}

#[test]
#[ignore = "requires root and XRAT_MACOS_SINGBOX on a disposable native macOS runner"]
fn native_utun_release_preserves_dns_after_graceful_and_forced_exit() {
    let binary = std::env::var_os("XRAT_MACOS_SINGBOX").expect("set XRAT_MACOS_SINGBOX");
    let uid = Command::new("/usr/bin/id").arg("-u").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&uid.stdout).trim(), "0");
    let interface = "utun99";
    assert!(
        inspect(interface).unwrap().is_none(),
        "refusing existing interface"
    );
    let addresses = vec!["172.31.255.1/30".to_string()];
    super::super::macos_routes::preflight(&addresses, None).unwrap();
    let baseline_dns = dns_snapshot();
    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join("config.json");
    std::fs::write(
        &config,
        serde_json::to_vec(&serde_json::json!({
            "log": {"level": "debug"},
            "inbounds": [{"type":"tun", "tag":"tun-in", "interface_name":interface,
                "address":addresses, "auto_route":true, "stack":"system"}],
            "outbounds": [{"type":"direct", "tag":"direct"}],
            "route": {"auto_detect_interface":true, "final":"direct"}
        }))
        .unwrap(),
    )
    .unwrap();
    for forced in [false, true] {
        let log =
            std::fs::File::create(directory.path().join(format!("engine-{forced}.log"))).unwrap();
        let mut engine = Engine(
            Command::new(&binary)
                .args(["run", "-c"])
                .arg(&config)
                .stdin(Stdio::null())
                .stdout(log.try_clone().unwrap())
                .stderr(log)
                .spawn()
                .unwrap(),
        );
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let info = loop {
            assert!(
                engine.0.try_wait().unwrap().is_none(),
                "engine exited during startup"
            );
            if let Some(info) = inspect(interface).unwrap()
                && super::super::macos_routes::ready(&addresses, interface).unwrap()
            {
                break info;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "utun routes were not ready"
            );
            std::thread::sleep(std::time::Duration::from_millis(20));
        };
        assert!(wait_for_release(interface, info.ifindex + 1).is_err());
        if forced {
            engine.0.kill().unwrap();
        } else {
            assert!(
                Command::new("/bin/kill")
                    .args(["-TERM", &engine.0.id().to_string()])
                    .status()
                    .unwrap()
                    .success()
            );
        }
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while engine.0.try_wait().unwrap().is_none() {
            assert!(std::time::Instant::now() < deadline, "engine did not stop");
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        wait_for_release(interface, info.ifindex).unwrap();
        assert!(inspect(interface).unwrap().is_none());
        assert_eq!(dns_snapshot(), baseline_dns);
        super::super::macos_routes::preflight(&addresses, None).unwrap();
    }
}
