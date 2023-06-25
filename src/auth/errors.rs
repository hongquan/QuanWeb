use thiserror::Error;
use axum::{extract::rejection::JsonRejection, response::IntoResponse, Json};
use serde_json::json;


#[derive(Debug, Error)]
pub enum ApiError {
    #[error(transparent)]
    JsonExtractorRejection(#[from] JsonRejection),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ApiError::JsonExtractorRejection(json_rejection) => {
                (json_rejection.status(), json_rejection.body_text())
            }
        };
        let payload = json!({
            "message": message,
            "origin": "with_rejection",
        });
        (status, Json(payload)).into_response()
    }
}
