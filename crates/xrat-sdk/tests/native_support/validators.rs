#[cfg(not(windows))]
use std::path::PathBuf;
use std::process::Command;
use xrat_sdk::engines::{singbox, xray};

#[test]
#[ignore = "requires pinned Xray and sing-box binaries"]
fn generated_configs_pass_pinned_native_validators() {
    #[cfg(not(windows))]
    let xray_binary = PathBuf::from(std::env::var("XRAT_SDK_XRAY_BINARY").expect("Xray binary"));
    #[cfg(not(windows))]
    let singbox_binary =
        PathBuf::from(std::env::var("XRAT_SDK_SINGBOX_BINARY").expect("sing-box binary"));
    #[cfg(windows)]
    let (xray_binary, singbox_binary) = {
        let engines = crate::windows_engines::paths();
        (engines.xray.clone(), engines.singbox.clone())
    };
    let version = Command::new(&xray_binary).arg("version").output().unwrap();
    assert!(
        String::from_utf8_lossy(&version.stdout).contains(if cfg!(windows) {
            "26.9.30"
        } else {
            "26.3.27"
        })
    );
    let version = Command::new(&singbox_binary)
        .arg("version")
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&version.stdout).contains("1.13.21"));
    let directory = tempfile::tempdir().unwrap();
    for node in crate::common::nodes::nodes() {
        let xray = xray::generate_runtime_config(&node, 1080, Some(8080)).unwrap();
        let singbox = singbox::generate_singbox_runtime_config(
            &node,
            vec![singbox::SingboxInbound::socks(
                "socks-in",
                "127.0.0.1",
                1080,
                None,
            )],
            None,
            None,
        )
        .unwrap();
        for (binary, args, value) in [
            (
                &xray_binary,
                vec!["run", "-test", "-c"],
                serde_json::to_vec(&xray).unwrap(),
            ),
            (
                &singbox_binary,
                vec!["check", "-c"],
                serde_json::to_vec(&singbox).unwrap(),
            ),
        ] {
            let file = directory.path().join("config.json");
            std::fs::write(&file, value).unwrap();
            let output = Command::new(binary).args(args).arg(&file).output().unwrap();
            assert!(
                output.status.success(),
                "{} {}: {} {}",
                binary.display(),
                node.protocol,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
