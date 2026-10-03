use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
pub use reqwest::header::{HeaderMap, HeaderValue};
pub use reqwest::{Method, StatusCode};

mod reqwest_adapter;
pub use reqwest_adapter::{ReqwestBlockingHttpClient, ReqwestHttpClient};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpErrorKind {
    Timeout,
    Connect,
    Tls,
    Auth,
    Redirect,
    Request,
    Body,
    Status,
    Other,
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct HttpError {
    pub kind: HttpErrorKind,
    pub message: String,
}
impl HttpError {
    pub fn new(kind: HttpErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
    pub fn is_timeout(&self) -> bool {
        self.kind == HttpErrorKind::Timeout
    }
    pub fn is_connect(&self) -> bool {
        self.kind == HttpErrorKind::Connect
    }
    pub fn is_redirect(&self) -> bool {
        self.kind == HttpErrorKind::Redirect
    }
    pub fn is_request(&self) -> bool {
        self.kind == HttpErrorKind::Request
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RedirectPolicy {
    #[default]
    Default,
    None,
    Limited(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HttpOptions {
    pub timeout: Option<Duration>,
    pub proxy: Option<String>,
    pub redirect: RedirectPolicy,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: Method,
    pub url: String,
    pub headers: HeaderMap,
    pub body: Vec<u8>,
    pub options: HttpOptions,
}

#[async_trait]
pub trait ResponseBody: Send {
    async fn next_chunk(&mut self) -> Result<Option<Vec<u8>>, HttpError>;
}

pub struct HttpResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub content_length: Option<u64>,
    pub remote_addr: Option<SocketAddr>,
    pub body: Box<dyn ResponseBody>,
}
struct BufferedBody(Option<Vec<u8>>);
#[async_trait]
impl ResponseBody for BufferedBody {
    async fn next_chunk(&mut self) -> Result<Option<Vec<u8>>, HttpError> {
        Ok(self.0.take())
    }
}
impl HttpResponse {
    pub fn from_bytes(status: StatusCode, body: Vec<u8>) -> Self {
        Self {
            status,
            headers: HeaderMap::new(),
            content_length: Some(body.len() as u64),
            remote_addr: None,
            body: Box::new(BufferedBody(Some(body))),
        }
    }

    pub fn status(&self) -> StatusCode {
        self.status
    }
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }
    pub fn content_length(&self) -> Option<u64> {
        self.content_length
    }
    pub fn remote_addr(&self) -> Option<SocketAddr> {
        self.remote_addr
    }
    pub async fn chunk(&mut self) -> Result<Option<Vec<u8>>, HttpError> {
        self.body.next_chunk().await
    }
    pub async fn bytes(mut self) -> Result<Vec<u8>, HttpError> {
        let mut bytes = Vec::new();
        while let Some(chunk) = self.chunk().await? {
            bytes.extend(chunk);
        }
        Ok(bytes)
    }
    pub async fn text(self) -> Result<String, HttpError> {
        Ok(String::from_utf8_lossy(&self.bytes().await?)
            .trim_start_matches('\u{feff}')
            .to_string())
    }
    pub fn error_for_status(self) -> Result<Self, HttpError> {
        if self.status.is_client_error() || self.status.is_server_error() {
            Err(HttpError::new(
                HttpErrorKind::Status,
                format!("HTTP status {}", self.status),
            ))
        } else {
            Ok(self)
        }
    }
}

#[async_trait]
pub trait HttpClient: Send + Sync {
    async fn execute(&self, request: HttpRequest) -> Result<HttpResponse, HttpError>;
}

#[derive(Clone)]
pub struct Client {
    transport: Arc<dyn HttpClient>,
    options: HttpOptions,
}
impl std::fmt::Debug for Client {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("Client").finish_non_exhaustive()
    }
}
impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
impl Client {
    pub fn new() -> Self {
        Self::with_transport(
            Arc::new(ReqwestHttpClient::default()),
            HttpOptions::default(),
        )
    }
    pub fn with_transport(transport: Arc<dyn HttpClient>, options: HttpOptions) -> Self {
        Self { transport, options }
    }
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.options.timeout = Some(timeout);
        self
    }
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }
    pub fn get(&self, url: impl AsRef<str>) -> RequestBuilder {
        self.request(Method::GET, url)
    }
    pub fn head(&self, url: impl AsRef<str>) -> RequestBuilder {
        self.request(Method::HEAD, url)
    }
    pub fn post(&self, url: impl AsRef<str>) -> RequestBuilder {
        self.request(Method::POST, url)
    }
    fn request(&self, method: Method, url: impl AsRef<str>) -> RequestBuilder {
        RequestBuilder {
            client: self.clone(),
            request: HttpRequest {
                method,
                url: url.as_ref().into(),
                headers: HeaderMap::new(),
                body: Vec::new(),
                options: self.options.clone(),
            },
            error: None,
        }
    }
}
#[derive(Default)]
pub struct ClientBuilder {
    options: HttpOptions,
}
impl ClientBuilder {
    pub fn timeout(mut self, value: Duration) -> Self {
        self.options.timeout = Some(value);
        self
    }
    pub fn proxy(mut self, value: Proxy) -> Self {
        self.options.proxy = Some(value.0);
        self
    }
    pub fn redirect(mut self, value: RedirectPolicy) -> Self {
        self.options.redirect = value;
        self
    }
    pub fn user_agent(mut self, value: impl Into<String>) -> Self {
        self.options.user_agent = Some(value.into());
        self
    }
    pub fn build(self) -> Result<Client, HttpError> {
        let transport = Arc::new(ReqwestHttpClient::new(&self.options)?);
        Ok(Client::with_transport(transport, self.options))
    }
}
pub struct Proxy(String);
impl Proxy {
    pub fn all(value: &str) -> Result<Self, HttpError> {
        reqwest_adapter::validate_proxy(value)?;
        Ok(Self(value.into()))
    }
}
pub struct RequestBuilder {
    client: Client,
    request: HttpRequest,
    error: Option<HttpError>,
}
impl RequestBuilder {
    pub fn bearer_auth(mut self, value: &str) -> Self {
        match HeaderValue::from_str(&format!("Bearer {value}")) {
            Ok(value) => {
                self.request.headers.insert("authorization", value);
            }
            Err(error) => {
                self.error = Some(HttpError::new(HttpErrorKind::Request, error.to_string()));
            }
        }
        self
    }
    pub fn body(mut self, value: Vec<u8>) -> Self {
        self.request.body = value;
        self
    }
    pub async fn send(self) -> Result<HttpResponse, HttpError> {
        if let Some(error) = self.error {
            return Err(error);
        }
        self.client.transport.execute(self.request).await
    }
}
pub async fn get(url: &str) -> Result<HttpResponse, HttpError> {
    Client::new().get(url).send().await
}

pub struct BlockingResponse {
    pub status: StatusCode,
    pub bytes: Vec<u8>,
}
impl BlockingResponse {
    pub fn error_for_status(self) -> Result<Self, HttpError> {
        if self.status.is_client_error() || self.status.is_server_error() {
            Err(HttpError::new(
                HttpErrorKind::Status,
                format!("HTTP status {}", self.status),
            ))
        } else {
            Ok(self)
        }
    }
    pub fn bytes(self) -> Result<Vec<u8>, HttpError> {
        Ok(self.bytes)
    }
}
pub trait BlockingHttpClient: Send + Sync {
    fn get(&self, url: &str) -> Result<BlockingResponse, HttpError>;
}
pub mod blocking {
    use super::*;
    pub fn get(url: &str) -> Result<BlockingResponse, HttpError> {
        ReqwestBlockingHttpClient.get(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::Mutex;
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
}
