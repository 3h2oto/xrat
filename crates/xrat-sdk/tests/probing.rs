use std::{path::Path, time::Duration};
use xrat_sdk::{
    config::parse_link,
    engines::xray::XrayGenOptions,
    prober::{self, AcceptedHttpStatuses, FailureKind, ProbeEngineKind},
};

#[tokio::test]
async fn tcp_probe_reports_local_success_and_refusal() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    assert!(
        prober::tcp_check("127.0.0.1", port, Duration::from_secs(1))
            .await
            .success
    );
    drop(listener);
    let result = prober::tcp_check("127.0.0.1", 0, Duration::from_secs(1)).await;
    assert_eq!(result.failure_kind, Some(FailureKind::Refused));
}

#[tokio::test]
async fn proxy_probes_report_missing_binary_without_panicking() {
    let node = parse_link("http://127.0.0.1:443").unwrap().unwrap();
    for engine in [ProbeEngineKind::Xray, ProbeEngineKind::Singbox] {
        let options = XrayGenOptions::default();
        let result = prober::real_delay_check(
            &node,
            "http://127.0.0.1/",
            engine,
            Path::new("/definitely-not-installed/sdk-engine"),
            Duration::from_secs(1),
            Duration::from_secs(1),
            &options,
            &AcceptedHttpStatuses::default(),
            false,
        )
        .await;
        assert_eq!(result.failure_kind, Some(FailureKind::Process));
        let result = prober::download_speed_check(
            &node,
            "http://127.0.0.1/",
            engine,
            Path::new("/definitely-not-installed/sdk-engine"),
            Duration::from_secs(1),
            Duration::from_secs(1),
            &options,
        )
        .await;
        assert_eq!(result.failure_kind, Some(FailureKind::Process));
        let result = prober::upload_speed_check(
            &node,
            "http://127.0.0.1/",
            engine,
            Path::new("/definitely-not-installed/sdk-engine"),
            Duration::from_secs(1),
            Duration::from_secs(1),
            1024,
            &options,
        )
        .await;
        assert_eq!(result.failure_kind, Some(FailureKind::Process));
    }
}
