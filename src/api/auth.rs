
use axum::http::StatusCode;
use axum::{debug_handler, Json, response::Result as AxumResult};
use axum::extract::State;
use axum_extra::extract::WithRejection;
use djangohashers::check_password;
use validify::Validate;
use serde_json::Value;
use edgedb_tokio::Client;

use crate::auth::structs::LoginReqData;
use crate::models::User;
use crate::stores;
use crate::types::ApiErrorShape;
use super::errors::ApiError;
use crate::auth::Auth;

#[debug_handler]
pub async fn login(
    mut auth: Auth,
    State(db): State<Client>,
    WithRejection(Json(value), _): WithRejection<Json<Value>, ApiError>,
) -> AxumResult<Json<User>> {
    let login_data: LoginReqData = serde_json::from_value(value).map_err(ApiError::JsonExtractionError)?;
    login_data.validate().map_err(ApiError::ValidationErrors)?;
    tracing::info!("Validated request data: {:?}", login_data);
    let user = stores::user::get_user_by_email(&login_data.email, &db)
        .await
        .map_err(ApiError::EdgeDBQueryError)?
        .ok_or_else(|| {
            tracing::info!("User not found");
            let resp: ApiErrorShape = "User not found".to_string().into();
            (StatusCode::UNAUTHORIZED, Json(resp))
        })?;
    let passwd_check = check_password(&login_data.password.expose_secret(), &user.password)
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
