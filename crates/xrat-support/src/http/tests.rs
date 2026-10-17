use super::*;
use axum::{Router, response::Redirect, routing::get};
use std::collections::VecDeque;
use std::sync::Mutex;
use tokio::net::TcpListener;
struct ScriptedBody(VecDeque<Result<Vec<u8>, HttpError>>);
#[async_trait]
impl ResponseBody for ScriptedBody {
    async fn next_chunk(&mut self) -> Result<Option<Vec<u8>>, HttpError> {
        self.0.pop_front().transpose()
    }
}
struct FakeTransport(Mutex<Vec<HttpRequest>>);
#[async_trait]
impl HttpClient for FakeTransport {
    async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, HttpError> {
        self.0.lock().unwrap().push(request);
        Ok(HttpResponse {
            status: StatusCode::OK,
            headers: HeaderMap::new(),
            content_length: Some(6),
            remote_addr: None,
            body: Box::new(ScriptedBody(VecDeque::from([
                Ok(b"abc".to_vec()),
                Ok(b"def".to_vec()),
            ]))),
        })
    }
}
#[tokio::test]
async fn typed_request_preserves_options_authorization_and_body_chunks() {
    let transport = Arc::new(FakeTransport(Mutex::new(Vec::new())));
    let options = HttpOptions {
        timeout: Some(Duration::from_secs(8)),
        proxy: Some("socks5h://127.0.0.1:1080".into()),
        redirect: RedirectPolicy::Limited(10),
        user_agent: Some("xrat-test".into()),
    };
    let client = Client::with_transport(transport.clone(), options.clone());
    let mut response = client
        .post("https://example.invalid/upload")
        .bearer_auth("token")
        .body(vec![1, 2, 3])
        .send()
        .await
        .unwrap();
    assert_eq!(response.content_length(), Some(6));
    assert_eq!(response.chunk().await.unwrap(), Some(b"abc".to_vec()));
    assert_eq!(response.bytes().await.unwrap(), b"def");
    let requests = transport.0.lock().unwrap();
    assert_eq!(requests[0].options, options);
    assert_eq!(requests[0].method, Method::POST);
    assert_eq!(requests[0].headers["authorization"], "Bearer token");
    assert_eq!(requests[0].body, [1, 2, 3]);
}
#[tokio::test]
async fn interrupted_body_is_reported_after_successful_headers() {
    let response = HttpResponse {
        status: StatusCode::OK,
        headers: HeaderMap::new(),
        content_length: Some(10),
        remote_addr: None,
        body: Box::new(ScriptedBody(VecDeque::from([
            Ok(b"partial".to_vec()),
            Err(HttpError::new(HttpErrorKind::Body, "interrupted")),
        ]))),
    };
    assert_eq!(
        response.bytes().await.unwrap_err().kind,
        HttpErrorKind::Body
    );
}
#[test]
fn status_errors_preserve_redirect_responses() {
    let response = |status| HttpResponse {
        status,
        headers: HeaderMap::new(),
        content_length: None,
        remote_addr: None,
        body: Box::new(ScriptedBody(VecDeque::new())),
    };
    assert!(response(StatusCode::FOUND).error_for_status().is_ok());
    assert!(matches!(
        response(StatusCode::BAD_GATEWAY).error_for_status(),
        Err(HttpError {
            kind: HttpErrorKind::Status,
            ..
        })
    ));
}
#[test]
fn safe_summary_never_copies_untrusted_request_or_response_details() {
    let secret =
        "https://user:password@example.invalid/private-token?key=secret redirected to secret";
    for kind in [
        HttpErrorKind::Timeout,
        HttpErrorKind::Connect,
        HttpErrorKind::Tls,
        HttpErrorKind::Auth,
        HttpErrorKind::Redirect,
        HttpErrorKind::Request,
        HttpErrorKind::Body,
        HttpErrorKind::Status,
        HttpErrorKind::Other,
    ] {
        let error = HttpError::new(kind, secret);
        let summary = error.safe_summary();
        for token in [
            "password",
            "private-token",
            "secret",
            "example.invalid",
            "user:",
        ] {
            assert!(!summary.contains(token), "{kind:?}: {summary}");
        }
    }
    assert_eq!(
        HttpError::new(HttpErrorKind::Connect, format!("dns error: {secret}")).safe_summary(),
        "DNS lookup failed"
    );
    for code in [401, 404, 429, 500, 503] {
        let error = HttpError::new(
            HttpErrorKind::Status,
            format!("HTTP status {code} {secret}"),
        );
        assert!(error.safe_summary().contains(&code.to_string()));
        assert!(!error.safe_summary().contains("secret"));
    }
}

async fn redirect_server(target: &str) -> String {
    let target = target.to_owned();
    let app = Router::new()
        .route(
            "/",
            get(move || async move { Redirect::temporary(&target) }),
        )
        .route("/loop", get(|| async { Redirect::temporary("/loop") }))
        .route(
            "/headers",
            get(|headers: HeaderMap| async move {
                headers.get("x-hwid").unwrap().to_str().unwrap().to_owned()
            }),
        );
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{address}")
}

#[tokio::test]
async fn same_origin_redirect_preserves_custom_headers() {
    let url = redirect_server("/headers").await;
    let client = Client::builder()
        .redirect(RedirectPolicy::SameOrigin)
        .build()
        .unwrap();
    let headers = HeaderMap::from_iter([(
        HeaderName::from_static("x-hwid"),
        HeaderValue::from_static("device-secret"),
    )]);
    let body = client
        .get(&url)
        .headers(headers)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert_eq!(body, "device-secret");
}

#[tokio::test]
async fn same_origin_redirect_blocks_other_origins() {
    let destination = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target = format!(
        "http://{}/private-token?key=secret",
        destination.local_addr().unwrap()
    );
    let url = redirect_server(&target).await;
    let client = Client::builder()
        .redirect(RedirectPolicy::SameOrigin)
        .build()
        .unwrap();
    let error = client.get(&url).send().await.err().unwrap();
    assert!(error.is_redirect());
    assert!(!error.safe_summary().contains("secret"));
    assert!(
        tokio::time::timeout(Duration::from_millis(100), destination.accept())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn same_origin_redirect_keeps_loop_limit() {
    let url = redirect_server("/loop").await;
    let client = Client::builder()
        .redirect(RedirectPolicy::SameOrigin)
        .build()
        .unwrap();
    let error = tokio::time::timeout(Duration::from_secs(5), client.get(&url).send())
        .await
        .unwrap()
        .err()
        .unwrap();
    assert!(error.is_redirect());
}
