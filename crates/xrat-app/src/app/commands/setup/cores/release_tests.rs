use super::*;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use xrat_support::http::{
    Client, HttpClient, HttpError, HttpErrorKind, HttpOptions, HttpRequest, HttpResponse,
    ResponseBody,
};

struct MetadataTransport(Mutex<VecDeque<(String, HttpResponse)>>);
#[async_trait::async_trait]
impl HttpClient for MetadataTransport {
    async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, HttpError> {
        let (url, response) = self
            .0
            .lock()
            .unwrap()
            .pop_front()
            .expect("unexpected request");
        assert_eq!(request.url, url);
        Ok(response)
    }
}

fn response(prerelease: bool, next: Option<&str>) -> HttpResponse {
    let mut response = HttpResponse::from_bytes(xrat_support::http::StatusCode::OK, serde_json::to_vec(&serde_json::json!([{
        "tag_name": "v26.7.28", "prerelease": prerelease, "created_at": "2026-07-28T00:00:00Z",
        "assets": [{"name": asset_name(CoreKind::Xray, &Version::new(26,7,28)).unwrap(),
            "browser_download_url": "https://example.invalid/xray.zip", "digest": format!("sha256:{}", "a".repeat(64))}]
    }])).unwrap());
    if let Some(url) = next {
        response.headers.insert(
            "link",
            format!("<{url}>; rel=\"next\", <https://example.invalid/last>; rel=\"last\"")
                .parse()
                .unwrap(),
        );
    }
    response
}

#[tokio::test]
async fn prerelease_lookup_uses_small_pages_and_follows_next_link() {
    let first = release_api_url(CoreKind::Xray, None, true);
    let second = format!("{first}&page=2");
    let transport = Arc::new(MetadataTransport(Mutex::new(VecDeque::from([
        (first, response(false, Some(&second))),
        (second, response(true, None)),
    ]))));
    let client = Client::with_transport(transport.clone(), HttpOptions::default());
    let release = fetch_release(&client, CoreKind::Xray, None, true)
        .await
        .unwrap();
    assert_eq!(release.version, Version::new(26, 7, 28));
    assert!(transport.0.lock().unwrap().is_empty());
}

#[tokio::test]
async fn prerelease_lookup_stops_after_finding_a_published_prerelease() {
    let first = release_api_url(CoreKind::Xray, None, true);
    let transport = Arc::new(MetadataTransport(Mutex::new(VecDeque::from([(
        first,
        response(true, Some("https://example.invalid/unneeded")),
    )]))));
    let client = Client::with_transport(transport, HttpOptions::default());
    assert!(
        fetch_release(&client, CoreKind::Xray, None, true)
            .await
            .is_ok()
    );
}

struct TimeoutBody;
#[async_trait::async_trait]
impl ResponseBody for TimeoutBody {
    async fn next_chunk(&mut self) -> Result<Option<Vec<u8>>, HttpError> {
        Err(HttpError::new(
            HttpErrorKind::Timeout,
            "error decoding response body",
        ))
    }
}

#[tokio::test]
async fn body_timeout_reports_download_timeout_and_pinned_version_workaround() {
    let url = release_api_url(CoreKind::Xray, None, true);
    let mut response = response(true, None);
    response.body = Box::new(TimeoutBody);
    let transport = Arc::new(MetadataTransport(Mutex::new(VecDeque::from([(
        url, response,
    )]))));
    let client = Client::with_transport(transport, HttpOptions::default());
    let error = fetch_release(&client, CoreKind::Xray, None, true)
        .await
        .unwrap_err();
    assert!(error.contains("metadata download timed out"), "{error}");
    assert!(error.contains("xrat install xray --version"));
}

#[tokio::test]
async fn no_prerelease_returns_a_clear_error_after_the_last_page() {
    let url = release_api_url(CoreKind::Xray, None, true);
    let transport = Arc::new(MetadataTransport(Mutex::new(VecDeque::from([(
        url,
        response(false, None),
    )]))));
    let client = Client::with_transport(transport, HttpOptions::default());
    assert!(
        fetch_release(&client, CoreKind::Xray, None, true)
            .await
            .unwrap_err()
            .contains("no published prerelease")
    );
}
