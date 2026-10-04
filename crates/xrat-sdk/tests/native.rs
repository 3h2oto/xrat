#![cfg(target_os = "linux")]

mod common;
mod native_support;

use native_support::{engine::Engine, proxy::Proxy};
use std::{path::PathBuf, process::Command, time::Duration};
use xrat_sdk::{
    config::parse_link,
    engines::{singbox, xray},
    prober::{self, AcceptedHttpStatuses, FailureKind, ProbeEngineKind},
};

#[test]
#[ignore = "requires pinned Xray and sing-box binaries"]
fn generated_configs_pass_pinned_native_validators() {
    let xray_binary = PathBuf::from(std::env::var("XRAT_SDK_XRAY_BINARY").expect("Xray binary"));
    let singbox_binary =
        PathBuf::from(std::env::var("XRAT_SDK_SINGBOX_BINARY").expect("sing-box binary"));
    let version = Command::new(&xray_binary).arg("version").output().unwrap();
    assert!(String::from_utf8_lossy(&version.stdout).contains("26.3.27"));
    let version = Command::new(&singbox_binary)
        .arg("version")
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&version.stdout).contains("1.13.21"));
    let directory = tempfile::tempdir().unwrap();
    for node in common::nodes::nodes() {
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

#[tokio::test]
#[ignore = "requires native engines and local sockets"]
async fn proxy_probes_cover_success_failure_timeout_and_cleanup() {
    for kind in [ProbeEngineKind::Xray, ProbeEngineKind::Singbox] {
        let engine = Engine::new(kind);
        for (status, stall, expected) in [
            (200, false, None),
            (403, false, Some(FailureKind::Proxy)),
            (200, true, Some(FailureKind::Timeout)),
        ] {
            let proxy = Proxy::new(status, stall).await;
            let node = parse_link(&format!("http://127.0.0.1:{}", proxy.port))
                .unwrap()
                .unwrap();
            let result = prober::real_delay_check(
                &node,
                "http://127.0.0.1/",
                engine.kind,
                &engine.binary,
                Duration::from_secs(5),
                Duration::from_millis(250),
                &xray::XrayGenOptions::default(),
                &AcceptedHttpStatuses::default(),
                false,
            )
            .await;
            assert_eq!(
                result.success,
                expected.is_none(),
                "{:?}",
                result.failure_reason
            );
            if let Some(expected) = expected {
                assert_eq!(result.failure_kind, Some(expected));
            }
            engine.assert_stopped().await;
        }
        let proxy = Proxy::new(200, false).await;
        let node = parse_link(&format!("http://127.0.0.1:{}", proxy.port))
            .unwrap()
            .unwrap();
        let result = prober::download_speed_check(
            &node,
            "http://127.0.0.1/",
            engine.kind,
            &engine.binary,
            Duration::from_secs(5),
            Duration::from_secs(2),
            &xray::XrayGenOptions::default(),
        )
        .await;
        assert!(result.success, "{:?}", result.failure_reason);
        engine.assert_stopped().await;
        let proxy = Proxy::new(200, false).await;
        let node = parse_link(&format!("http://127.0.0.1:{}", proxy.port))
            .unwrap()
            .unwrap();
        let result = prober::upload_speed_check(
            &node,
            "http://127.0.0.1/",
            engine.kind,
            &engine.binary,
            Duration::from_secs(5),
            Duration::from_secs(2),
            1024,
            &xray::XrayGenOptions::default(),
        )
        .await;
        assert!(result.success, "{:?}", result.failure_reason);
        engine.assert_stopped().await;
        let mut proxy = Proxy::new(200, true).await;
        let node = parse_link(&format!("http://127.0.0.1:{}", proxy.port))
            .unwrap()
            .unwrap();
        let binary = engine.binary.clone();
        let task = tokio::spawn(async move {
            prober::real_delay_check(
                &node,
                "http://127.0.0.1/",
                kind,
                &binary,
                Duration::from_secs(5),
                Duration::from_secs(60),
                &xray::XrayGenOptions::default(),
                &AcceptedHttpStatuses::default(),
                false,
            )
            .await
        });
        tokio::time::timeout(Duration::from_secs(5), &mut proxy.request)
            .await
            .unwrap()
            .unwrap();
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        engine.assert_stopped().await;
        for (command, expected) in [
            ("/bin/false", FailureKind::Process),
            ("/bin/sleep 60", FailureKind::Process),
        ] {
            engine.replace_command(command);
            let node = parse_link("http://127.0.0.1:443").unwrap().unwrap();
            let result = prober::real_delay_check(
                &node,
                "http://127.0.0.1/",
                kind,
                &engine.binary,
                Duration::from_millis(100),
                Duration::from_secs(1),
                &xray::XrayGenOptions::default(),
                &AcceptedHttpStatuses::default(),
                false,
            )
            .await;
            assert!(!result.success);
            assert_eq!(result.failure_kind, Some(expected));
            engine.assert_stopped().await;
        }
    }
}
