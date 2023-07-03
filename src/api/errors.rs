use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::http::StatusCode;
use axum::{response::IntoResponse, Json};
use thiserror::Error;
use edgedb_errors::display::display_error_verbose;
use edgedb_errors::kinds as EdErrKind;

use crate::types::ApiErrorShape;

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum ApiError {
    #[error(transparent)]
    PathRejection(#[from] PathRejection),
    #[error(transparent)]
    JsonRejection(#[from] JsonRejection),
    #[error(transparent)]
    JsonExtractionError(#[from] serde_json::Error),
    #[error(transparent)]
    EdgeDBQueryError(#[from] edgedb_errors::Error),
    #[error("{0} not found")]
    ObjectNotFound(String),
    #[error("Error logging in")]
    LoginError(String),
    #[error("Not enough data")]
    NotEnoughData,
    #[error("Other error: {0}")]
    Other(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        tracing::debug!("To convert ApiError: {:?}", self);
        let (status, message) = match self {
            Self::PathRejection(path_rejection) => {
                (StatusCode::NOT_FOUND, path_rejection.body_text())
            }
            Self::JsonRejection(json_rejection) => {
                (json_rejection.status(), json_rejection.body_text())
            }
            Self::JsonExtractionError(ref e) => {
                if e.is_data() {
                    tracing::error!("Unexpected JSON shape: {}", e);
                    (StatusCode::UNPROCESSABLE_ENTITY, self.to_string())
                } else {
                    tracing::error!("Failed to parse as JSON: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
                }
            },
            Self::EdgeDBQueryError(ref e) => {
                tracing::error!("EdgeDB error: {}", display_error_verbose(e));
                if e.is::<EdErrKind::ConstraintViolationError>() {
                    let detail = e.details().unwrap_or_default();
                    (StatusCode::UNPROCESSABLE_ENTITY, detail.to_string())
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
                }
            }
            Self::ObjectNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            Self::LoginError(e) => (StatusCode::UNAUTHORIZED, e.to_string()),
            Self::NotEnoughData => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            Self::Other(message) => (StatusCode::INTERNAL_SERVER_ERROR, message)
        };
        let payload = ApiErrorShape::from(message);
        (status, Json(payload)).into_response()
    }
}
