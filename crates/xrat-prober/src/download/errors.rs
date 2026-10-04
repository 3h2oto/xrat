use crate::FailureKind;

pub(super) fn classify_request_error(
    error: &xrat_support::http::HttpError,
) -> (FailureKind, String) {
    if error.is_timeout() {
        (FailureKind::Timeout, "Download request timeout".to_string())
    } else if error.kind == xrat_support::http::HttpErrorKind::Tls
        || error.to_string().to_lowercase().contains("tls")
    {
        (FailureKind::Tls, format!("TLS handshake failed: {error}"))
    } else if error.kind == xrat_support::http::HttpErrorKind::Auth
        || error.to_string().to_lowercase().contains("407")
    {
        (
            FailureKind::Auth,
            format!("Proxy authentication failed: {error}"),
        )
    } else if error.is_connect() {
        (
            FailureKind::Proxy,
            format!("Proxy connection failed: {error}"),
        )
    } else if error.is_request() {
        (
            FailureKind::Proxy,
            format!("Download request failed: {error}"),
        )
    } else {
        (
            FailureKind::Unknown,
            format!("Download HTTP error: {error}"),
        )
    }
}
