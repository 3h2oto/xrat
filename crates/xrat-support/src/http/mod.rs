use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
pub use reqwest::header::{HeaderMap, HeaderValue};
pub use reqwest::{Method, StatusCode};

mod reqwest_adapter;
pub use reqwest_adapter::{ReqwestBlockingHttpClient, ReqwestHttpClient};

pub mod blocking;

#[cfg(test)]
mod tests;

mod client;
pub use client::BlockingHttpClient;
pub use client::BlockingResponse;
pub use client::Client;
pub use client::ClientBuilder;
pub use client::HttpClient;
pub use client::HttpError;
pub use client::HttpErrorKind;
pub use client::HttpOptions;
pub use client::HttpRequest;
pub use client::HttpResponse;
pub use client::Proxy;
pub use client::RedirectPolicy;
pub use client::RequestBuilder;
pub use client::ResponseBody;
pub use client::get;
