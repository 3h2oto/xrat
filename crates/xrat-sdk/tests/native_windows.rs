#![cfg(windows)]

mod common;
#[path = "native_support/proxy.rs"]
mod proxy;
#[path = "native_support/validators.rs"]
mod validators;
#[path = "native_support/windows_engines.rs"]
mod windows_engines;

use std::{path::Path, time::Duration};
use xrat_sdk::{
    config::parse_link,
    engines::xray,
    prober::{self, AcceptedHttpStatuses, FailureKind, ProbeEngineKind},
};

fn assert_stopped(binary: &Path) {
    let output = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command"])
        .arg("@(Get-CimInstance Win32_Process | Where-Object { $_.Name -eq $env:XRAT_TEST_ENGINE }).Count")
        .env("XRAT_TEST_ENGINE", binary.file_name().unwrap())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "0",
        "engine survived probe completion or cancellation"
    );
}

#[tokio::test]
#[ignore = "requires checksum-pinned native Windows engines and local sockets"]
async fn native_windows_proxy_probes_and_cancellation_release_the_engine() {
    let engines = tokio::task::spawn_blocking(windows_engines::paths)
        .await
        .unwrap();
    for (kind, source) in [
        (ProbeEngineKind::Xray, &engines.xray),
        (ProbeEngineKind::Singbox, &engines.singbox),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let binary = directory.path().join(format!(
            "xrat-probe-{}.exe",
            directory.path().file_name().unwrap().to_string_lossy()
        ));
        std::fs::copy(source, &binary).unwrap();
        for (status, stall, expected) in [
            (200, false, None),
            (403, false, Some(FailureKind::Proxy)),
            (200, true, Some(FailureKind::Timeout)),
        ] {
            let proxy = proxy::Proxy::new(status, stall).await;
            let node = parse_link(&format!("http://127.0.0.1:{}", proxy.port))
                .unwrap()
                .unwrap();
            let result = prober::real_delay_check(
                &node,
                "http://127.0.0.1/",
                kind,
                &binary,
                Duration::from_secs(10),
                Duration::from_secs(1),
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
            assert_stopped(&binary);
        }
        let mut proxy = proxy::Proxy::new(200, true).await;
        let node = parse_link(&format!("http://127.0.0.1:{}", proxy.port))
            .unwrap()
            .unwrap();
        let engine = binary.clone();
        let task = tokio::spawn(async move {
            prober::real_delay_check(
                &node,
                "http://127.0.0.1/",
                kind,
                &engine,
                Duration::from_secs(10),
                Duration::from_secs(60),
                &xray::XrayGenOptions::default(),
                &AcceptedHttpStatuses::default(),
                false,
            )
            .await
        });
        tokio::time::timeout(Duration::from_secs(15), &mut proxy.request)
            .await
            .unwrap()
            .unwrap();
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        // Cancellation initiates termination; allow the Windows process to exit.
        tokio::time::sleep(Duration::from_millis(500)).await;
        assert_stopped(&binary);
    }
}
