use std::collections::HashMap;

use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::http::StatusCode;
use axum::{response::IntoResponse, Json};
use gel_errors::display::display_error_verbose;
use gel_errors::kinds as EdErrKind;
use indexmap::IndexMap;
use serde_json::value::Value;
use thiserror::Error;
use validify::ValidationError as VE;

use crate::types::ApiErrorShape;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error(transparent)]
    PathRejection(#[from] PathRejection),
    #[error(transparent)]
    JsonRejection(#[from] JsonRejection),
    #[error(transparent)]
    JsonExtractionError(#[from] serde_json::Error),
    #[error(transparent)]
    GelQueryError(#[from] gel_errors::Error),
    #[error("{0} not found")]
    ObjectNotFound(String),
    #[error("Please login")]
    Unauthorized,
    #[error("Error logging in")]
    LoginError(String),
    #[error("Not enough data")]
    NotEnoughData,
    #[error(transparent)]
    ValidationErrors(#[from] validify::ValidationErrors),
    #[error("Other error: {0}")]
    Other(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        tracing::debug!("To convert ApiError: {:#?}", self);
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
            }
            Self::GelQueryError(ref e) => {
                tracing::error!("Gel error: {}", display_error_verbose(e));
                if e.is::<EdErrKind::ConstraintViolationError>() {
                    let detail = e.details().unwrap_or_default();
                    (StatusCode::UNPROCESSABLE_ENTITY, detail.to_string())
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
                }
            }
            Self::ObjectNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            Self::LoginError(e) => (StatusCode::UNAUTHORIZED, e.to_string()),
            Self::NotEnoughData => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            Self::ValidationErrors(e) => {
                let resp: ApiErrorShape = flatten_validation_errors(e).into();
                return (StatusCode::UNPROCESSABLE_ENTITY, Json(resp)).into_response();
            }
            Self::Other(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        };
        let payload = ApiErrorShape::from(message);
        (status, Json(payload)).into_response()
    }
}

pub fn flatten_validation_errors(
    errors: validify::ValidationErrors,
) -> IndexMap<&'static str, String> {
    let mut hm: IndexMap<&str, String> = IndexMap::new();
    let schema_errors: Vec<String> = errors
        .schema_errors()
        .iter()
        .filter_map(|e| e.message())
        .collect();
    schema_errors
        .first()
        .and_then(|f| hm.insert("_schema_", f.to_string()));
    let field_errors = errors.field_errors();
    let field_errors = field_errors.into_iter().filter_map(|e| match e {
        VE::Field {
            field: Some(field),
            code,
            params,
            message,
            location: _location,
        } => Some((
            field,
            message
                .or_else(|| deduce_message(code, &params))
                .unwrap_or("Please check again".into()),
        )),
        _ => None,
    });
    hm.extend(field_errors);
    hm
}

pub fn deduce_message(code: &str, params: &HashMap<&str, Value>) -> Option<String> {
    if code == "url" {
        return Some("Must be a valid URL".into());
    }
    params.get("min").and_then(|cond| {
        params
            .get("value")
            .map(|input_value| {
                if input_value.is_string() {
                    format!("Must be at least {cond} characters long")
                } else {
                    format!("Must be at least {cond} elements")
                }
            })
            .or(Some(format!("Must be at least {cond}")))
    })
}
