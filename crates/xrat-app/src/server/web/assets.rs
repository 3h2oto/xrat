use axum::{
    extract::Request,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};

#[derive(rust_embed::RustEmbed)]
#[folder = "web-assets/"]
struct Assets;

pub async fn serve(request: Request) -> Response {
    if !matches!(request.method().as_str(), "GET" | "HEAD") {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    let path = request.uri().path().trim_start_matches('/');
    if path.starts_with("api/") {
        return StatusCode::NOT_FOUND.into_response();
    }
    let file = if path.is_empty() || !path.contains('.') {
        "index.html"
    } else {
        path
    };
    match Assets::get(file) {
        Some(asset) => (
            [(
                header::CONTENT_TYPE,
                mime_guess::from_path(file)
                    .first_or_octet_stream()
                    .to_string(),
            )],
            asset.data,
        )
            .into_response(),
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Web UI assets unavailable; build with just web-build before compiling XRAT",
        )
            .into_response(),
    }
}
