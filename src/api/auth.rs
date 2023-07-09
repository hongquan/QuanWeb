
use axum::http::StatusCode;
use axum::{debug_handler, Json, response::Result as AxumResult};
use axum::extract::State;
use axum_extra::extract::WithRejection;
use axum_login::RequireAuthorizationLayer;
use djangohashers::check_password;
use garde::Validate;
use serde_json::Value;
use uuid::Uuid;
use edgedb_tokio::Client;

use crate::auth::structs::LoginReqData;
use crate::models::{User, Role};
use crate::stores;
use crate::types::ApiErrorShape;
use super::errors::ApiError;
use crate::auth::Auth;

#[allow(dead_code)]
pub type RequireAuth = RequireAuthorizationLayer<Uuid, User, Role>;

#[debug_handler]
pub async fn login(
    mut auth: Auth,
    State(db): State<Client>,
    WithRejection(Json(value), _): WithRejection<Json<Value>, ApiError>,
) -> AxumResult<Json<User>> {
    let valid_data: LoginReqData = serde_json::from_value(value).map_err(ApiError::JsonExtractionError)?;
    valid_data.validate(&()).map_err(ApiError::ValidationError)?;
    tracing::info!("Validated request data: {:?}", valid_data);
    let user = stores::get_user_by_email(&valid_data.email, &db)
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
    Ok(Json(user))
}


pub async fn logout(mut auth: Auth) {
    auth.logout().await;
}
