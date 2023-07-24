use thiserror::Error;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use edgedb_errors::display::display_error_verbose;

#[derive(Debug, Error)]
pub enum PageError {
    #[error(transparent)]
    EdgeDBQueryError(#[from] edgedb_errors::Error),
    #[error(transparent)]
    JinjaError(#[from] minijinja::Error),
    #[error("Permission denied")]
    PermissionDenied(String),
}

impl IntoResponse for PageError {
    fn into_response(self) -> Response {
        tracing::debug!("To convert PageError: {:?}", self);
        let (status, message) = match self {
            Self::EdgeDBQueryError(ref e) => {
                tracing::error!("EdgeDB error: {}", display_error_verbose(e));
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            Self::JinjaError(ref e) => {
                tracing::error!("Jinja error: {:#}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            Self::PermissionDenied(e) => (StatusCode::FORBIDDEN, e.to_string()),
        };
        (status, message).into_response()
    }
}
