use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::http::StatusCode;
use axum::{response::IntoResponse, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error(transparent)]
    PathRejection(#[from] PathRejection),
    #[error(transparent)]
    JsonExtractorRejection(#[from] JsonRejection),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        tracing::info!("To convert ApiError: {:?}", self);
        let (status, message) = match self {
            Self::PathRejection(path_rejection) => {
                (StatusCode::NOT_FOUND, path_rejection.body_text())
            }
            Self::JsonExtractorRejection(json_rejection) => {
                (json_rejection.status(), json_rejection.body_text())
            }
        };
        let payload = json!({
            "detail": message,
            "origin": "with_rejection",
        });
        (status, Json(payload)).into_response()
    }
}
