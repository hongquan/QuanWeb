use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::http::StatusCode;
use axum::{response::IntoResponse, Json};
use serde_json::json;
use thiserror::Error;

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
                tracing::error!("EdgeDB error: {}", e.initial_message().unwrap_or_default());
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            Self::Other(message) => (StatusCode::INTERNAL_SERVER_ERROR, message)
        };
        let payload = json!({
            "detail": message,
        });
        (status, Json(payload)).into_response()
    }
}
