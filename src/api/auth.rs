use std::collections::HashMap;

use axum::http::StatusCode;
use axum::{debug_handler, Json, response::Result as AxumResult};
use axum_extra::extract::WithRejection;
use axum_login::extractors::AuthContext;
use djangohashers::check_password;
use garde::Validate;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::auth::{store::EdgeDbStore, structs::LoginReqData};
use crate::db::get_edgedb_client;
use crate::models;
use crate::retrievers;
use crate::types::ApiErrorShape;
use super::errors::ApiError;

pub type Auth = AuthContext<Uuid, models::User, EdgeDbStore<models::User>, models::Role>;

fn flatten_garde_errors(errors: garde::Errors) -> HashMap<String, String> {
    errors
        .flatten()
        .into_iter()
        .map(|(k, v)| (k, v.message.to_string()))
        .collect()
}

#[debug_handler]
pub async fn login(
    mut auth: Auth,
    WithRejection(Json(value), _): WithRejection<Json<Value>, ApiError>,
) -> AxumResult<Json<Value>> {
    let valid_data: LoginReqData = serde_json::from_value(value).map_err(ApiError::JsonExtractionError)?;
    if let Err(e) = valid_data.validate(&()) {
        tracing::info!("Data validation failed: {}", e);
        let resp: ApiErrorShape = flatten_garde_errors(e).into();
        return Err((StatusCode::UNPROCESSABLE_ENTITY, Json(resp)).into());
    }
    tracing::info!("Validated request data: {:?}", valid_data);
    let client = get_edgedb_client().await.map_err(ApiError::EdgeDBQueryError)?;
    let user = retrievers::get_user_by_email(&valid_data.email, &client)
        .await
        .map_err(ApiError::EdgeDBQueryError)?
        .ok_or_else(|| {
            tracing::info!("User not found");
            let resp: ApiErrorShape = "User not found".to_string().into();
            (StatusCode::UNAUTHORIZED, Json(resp))
        })?;
    let passwd_check = check_password(&valid_data.password.expose_secret(), &user.password)
        .map_err(|e| {
            tracing::error!("Error checking password: {:?}", e);
            ApiError::LoginError("Wrong password".into())
        })?;
    tracing::info!("Password check: {:?}", passwd_check);
    passwd_check.then_some(()).ok_or(ApiError::LoginError("Wrong password".into()))?;
    tracing::info!("Logging in user: {:?}", user);
    auth.login(&user).await.map_err(|e| {
        tracing::error!("Error logging in user: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let resp = json!({
        "success": true,
        "email": user.email,
    });
    Ok(Json(resp))
}
