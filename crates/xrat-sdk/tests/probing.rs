use std::{path::Path, time::Duration};
use xrat_sdk::{
    config::parse_link,
    engines::xray::XrayGenOptions,
    prober::{self, AcceptedHttpStatuses, FailureKind, ProbeEngineKind},
};

#[tokio::test]
async fn tcp_probe_reports_local_success_and_refusal() {
    // Windows can take a couple of seconds to surface a refused connection:
    // network filter drivers (TUN/VPN adapters, security suites) delay the RST
    // that the TCP stack sends for a closed port, and the probe only learns
    // about the refusal once the OS reports it. The deadline therefore has to
    // be generous enough for the operating system to deliver that answer.
    const PROBE_TIMEOUT: Duration = Duration::from_secs(10);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    assert!(
        prober::tcp_check("127.0.0.1", port, PROBE_TIMEOUT)
            .await
            .success
    );
    // Release the port so the very same endpoint stops listening and refuses
    // connections. Port 0 must not be used as the "closed" target: it is not a
    // valid destination port, and the OS error for it differs per platform
    // (Linux/macOS report ECONNREFUSED, Windows reports WSAEADDRNOTAVAIL), so
    // the probe would not report a refusal there.
    drop(listener);
    let result = prober::tcp_check("127.0.0.1", port, PROBE_TIMEOUT).await;
    assert!(!result.success);
    assert_eq!(result.failure_kind, Some(FailureKind::Refused));
    assert_eq!(result.failure_reason.as_deref(), Some("Connection refused"));
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
