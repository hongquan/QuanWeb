use std::collections::HashMap;

use axum::http::StatusCode;
use axum::{debug_handler, Json};
use axum_extra::extract::WithRejection;
use axum_garde::WithValidation;
use axum_login::extractors::AuthContext;
use edgedb_errors::display::display_error_verbose;
use garde::Validate;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::db::get_edgedb_client;
use crate::models;
use crate::retrievers;
use crate::types::ApiErrorShape;
use crate::auth::{errors::ApiError, store::EdgeDbStore, structs::LoginReqData};

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
) -> axum::response::Result<Json<Value>> {
    tracing::info!("Request data: {:?}", value);
    let valid_data: LoginReqData = serde_json::from_value(value).map_err(|e| {
        tracing::error!("Error deserializing request data: {}", e);
        let resp: ApiErrorShape = e.to_string().into();
        (StatusCode::UNPROCESSABLE_ENTITY, Json(resp))
    })?;
    if let Err(e) = valid_data.validate(&()) {
        tracing::error!("Error validating request data: {}", e);
        let resp: ApiErrorShape = flatten_garde_errors(e).into();
        return Err((StatusCode::UNPROCESSABLE_ENTITY, Json(resp)).into());
    }
    tracing::info!("Validated request data: {:?}", valid_data);
    let client = get_edgedb_client().await.map_err(|e| {
        tracing::error!("Error connecting to EdgeDB: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let user = retrievers::get_first_user(client)
        .await
        .map_err(|e| {
            tracing::error!("{}", display_error_verbose(&e));
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;
    tracing::info!("Logging in user: {:?}", user);
    auth.login(&user).await.unwrap();
    let resp = json!({
        "success": true,
    });
    Ok(Json(resp))
}

#[debug_handler]
pub async fn login_short(
    WithValidation(login_data): WithValidation<WithRejection<Json<LoginReqData>, ApiError>>,
) -> axum::response::Result<Json<Value>> {
    tracing::info!("Request data: {:?}", login_data);
    let inner = login_data.into_inner();
    tracing::info!("Inner data: {:?}", inner);
    let resp = json!({
        "success": true,
    });
    Ok(Json(resp))
}
