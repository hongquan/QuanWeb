
use axum::http::StatusCode;
use axum::{Json, debug_handler};
use axum_garde::WithValidation;
use axum_login::extractors::AuthContext;
use edgedb_errors::display::display_error_verbose;
use uuid::Uuid;
use serde_json::{json, Value};

use super::store::EdgeDbStore;
use super::structs::LoginReqData;
use crate::db::get_edgedb_client;
use crate::models;
use crate::retrievers;

pub type Auth = AuthContext<Uuid, models::User, EdgeDbStore<models::User>, models::Role>;

#[debug_handler]
pub async fn login(
    mut auth: Auth,
    // WithRejection(Json(value), _): WithRejection<Json<Value>, ApiError>,
    WithValidation(valid_data): WithValidation<Json<LoginReqData>>,
) -> Result<Json<Value>, StatusCode> {
    tracing::info!("Request data: {:?}", valid_data);
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
