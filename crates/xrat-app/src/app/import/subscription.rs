use std::collections::BTreeMap;

use crate::app::AppError;
use crate::app::input::source::read_input_async_with_headers;
use xrat_db::{Database, ImportSummary};
use xrat_support::http::{Client, HeaderMap, HeaderName, HeaderValue, RedirectPolicy};
use xrat_support::url::looks_like_url;

pub async fn import_subscription(
    database: &Database,
    input: &str,
    name: Option<&str>,
    headers: &BTreeMap<String, String>,
) -> crate::app::Result<ImportSummary> {
    if !looks_like_url(input) {
        if !headers.is_empty() {
            return Err(AppError::InvalidArgument(
                "--header requires a subscription URL".into(),
            ));
        }
        let (source, nodes) = super::load_nodes_async(input).await?;
        return super::persist_nodes(database, source, &nodes, name, None).await;
    }

    let request_headers = parse_request_headers(headers)?;
    let client = if request_headers.is_empty() {
        Client::new()
    } else {
        Client::builder()
            .redirect(RedirectPolicy::SameOrigin)
            .build()?
    };
    let (source, bytes) = read_input_async_with_headers(input, &client, request_headers).await?;
    let nodes = super::parse_nodes(&bytes)?;
    super::persist_nodes(database, source, &nodes, name, Some(headers)).await
}

fn parse_request_headers(headers: &BTreeMap<String, String>) -> crate::app::Result<HeaderMap> {
    let mut request_headers = HeaderMap::new();
    for (name, value) in headers {
        let name = HeaderName::from_bytes(name.as_bytes())
            .map_err(|_| AppError::InvalidArgument("invalid subscription header name".into()))?;
        let mut value = HeaderValue::from_str(value)
            .map_err(|_| AppError::InvalidArgument("invalid subscription header value".into()))?;
        value.set_sensitive(true);
        request_headers.insert(name, value);
    }
    Ok(request_headers)
}
