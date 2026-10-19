use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::server::response::ApiErrorResponse;

pub type ServerResult<T> = Result<T, ServerError>;

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("missing api key")]
    MissingApiKey,
    #[error("invalid api key")]
    InvalidApiKey,
    #[error("forbidden")]
    Forbidden,
    #[error("too many requests")]
    RateLimited,
    #[error("invalid query: {0}")]
    InvalidQuery(String),
    #[error("config not found")]
    NotFound,
    #[error("PAC host is not allowed")]
    PacHostNotAllowed,
    #[error(transparent)]
    Database(#[from] xrat_db::DbError),
    #[error("{0}")]
    Application(#[from] crate::app::AppError),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::MissingApiKey | Self::InvalidApiKey => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::InvalidQuery(_) => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::PacHostNotAllowed => StatusCode::FORBIDDEN,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Application(crate::app::AppError::InvalidArgument(_)) => StatusCode::BAD_REQUEST,
            Self::Application(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let error = if status.is_server_error() {
            tracing::error!(error = %self, "HTTP request failed");
            "internal server error".to_string()
        } else {
            self.to_string()
        };
        let body = Json(ApiErrorResponse { error });

        (status, body).into_response()
    }
}
