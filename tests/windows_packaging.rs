#[path = "windows_packaging/fixture.rs"]
mod fixture;
#[path = "windows_packaging/upgrades.rs"]
mod upgrades;

use fixture::Fixture;
use std::{thread, time::Duration};

#[test]
#[ignore = "requires a packaged native Windows ZIP in XRAT_WINDOWS_ARCHIVE"]
fn packaged_daemon_serves_browser_assets_and_authenticates() {
    let fixture = Fixture::new();
    fixture.xrat(&["init"]);
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let password = format!(
        "XRAT-CI-{}",
        fixture
            .directory
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
    );
    let hash = fixture.password_hash(&password);
    std::fs::write(
        &fixture.config,
        format!(
            r#"
[server]
enabled = true
host = "127.0.0.1"
port = {port}
[server.web]
enabled = true
admin_username = "ci-admin"
admin_password_hash = '{hash}'
public_url = "http://localhost:{port}"
"#
        ),
    )
    .unwrap();
    fixture.xrat(&["daemon", "start"]);
    fixture.xrat(&["daemon", "status"]);
    let client = reqwest::blocking::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();
    let base = format!("http://localhost:{port}");
    let html = (0..50)
        .find_map(|_| {
            if let Ok(response) = client.get(format!("{base}/")).send()
                && response.status().is_success()
            {
                return Some(response.text().unwrap());
            }
            thread::sleep(Duration::from_millis(100));
            None
        })
        .expect("packaged HTTP server did not become ready");
    assert!(
        html.contains("id=\"main\""),
        "packaged HTML missing root element"
    );
    for (asset, mime) in [
        ("boot.js", "javascript"),
        ("xrat_web.js", "javascript"),
        ("xrat_web_bg.wasm", "application/wasm"),
        ("app.css", "text/css"),
    ] {
        let response = client
            .get(format!("{base}/{asset}"))
            .send()
            .unwrap()
            .error_for_status()
            .unwrap();
        assert!(
            response.headers()[reqwest::header::CONTENT_TYPE]
                .to_str()
                .unwrap()
                .contains(mime)
        );
        let bytes = response.bytes().unwrap();
        assert!(bytes.len() >= 8, "missing served asset {asset}");
        if asset.ends_with(".wasm") {
            assert_eq!(&bytes[..4], b"\0asm");
        }
    }
    let response = client
        .post(format!("{base}/api/v1/login"))
        .header("Origin", &base)
        .header("Content-Type", "application/json")
        .body(
            serde_json::to_vec(&serde_json::json!({"username":"ci-admin","password":password}))
                .unwrap(),
        )
        .send()
        .unwrap()
        .error_for_status()
        .unwrap();
    let cookies = response
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .map(|header| header.to_str().unwrap().split(';').next().unwrap())
        .collect::<Vec<_>>()
        .join("; ");
    assert!(!cookies.is_empty(), "login did not establish a session");
    let response = client
        .get(format!("{base}/api/v1/me"))
        .header(reqwest::header::COOKIE, cookies)
        .send()
        .unwrap()
        .error_for_status()
        .unwrap();
    let me: serde_json::Value = serde_json::from_slice(&response.bytes().unwrap()).unwrap();
    assert_eq!(me["user"]["username"], "ci-admin");
    fixture.xrat(&["daemon", "restart"]);
    fixture.xrat(&["daemon", "status"]);
    fixture.xrat(&["daemon", "stop"]);
    fixture.xrat(&["daemon", "stop"]);
}

#[test]
#[ignore = "requires a packaged native Windows ZIP in XRAT_WINDOWS_ARCHIVE"]
fn staged_upgrade_replaces_executable_and_rejects_tampering() {
    let fixture = Fixture::new();
    fixture.xrat(&["init"]);
    upgrades::exercise(&fixture);
}
