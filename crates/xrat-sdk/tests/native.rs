#![cfg(target_os = "linux")]

mod common;
mod native_support;
#[path = "native_support/validators.rs"]
mod validators;

use native_support::{engine::Engine, proxy::Proxy};
use std::time::Duration;
use xrat_sdk::{
    config::parse_link,
    engines::xray,
    prober::{self, AcceptedHttpStatuses, FailureKind, ProbeEngineKind},
};

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
