
use axum::http::StatusCode;
use axum::{debug_handler, Json, response::Result as AxumResult};
use axum_extra::extract::WithRejection;
use axum_login::{extractors::AuthContext, RequireAuthorizationLayer};
use djangohashers::check_password;
use garde::Validate;
use serde_json::Value;
use uuid::Uuid;

use crate::auth::{store::EdgeDbStore, structs::LoginReqData};
use crate::db::get_edgedb_client;
use crate::models::{User, Role};
use crate::stores;
use crate::types::ApiErrorShape;
use super::errors::ApiError;

pub type Auth = AuthContext<Uuid, User, EdgeDbStore<User>, Role>;
#[allow(dead_code)]
pub type RequireAuth = RequireAuthorizationLayer<Uuid, User, Role>;

#[debug_handler]
pub async fn login(
    mut auth: Auth,
    WithRejection(Json(value), _): WithRejection<Json<Value>, ApiError>,
) -> AxumResult<Json<User>> {
    let valid_data: LoginReqData = serde_json::from_value(value).map_err(ApiError::JsonExtractionError)?;
    valid_data.validate(&()).map_err(ApiError::ValidationError)?;
    tracing::info!("Validated request data: {:?}", valid_data);
    let client = get_edgedb_client().await.map_err(ApiError::EdgeDBQueryError)?;
    let user = stores::get_user_by_email(&valid_data.email, &client)
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
