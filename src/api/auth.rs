use axum::extract::State;
use axum::http::StatusCode;
use axum::{debug_handler, response::Result as AxumResult, Json};
use axum_extra::extract::WithRejection;
use edgedb_tokio::Client;
use serde_json::Value;
use tracing::info;
use validify::Validate;

use super::errors::ApiError;
use crate::auth::backend::Credentials;
use crate::auth::structs::LoginReqData;
use crate::auth::AuthSession;
use crate::models::User;

#[debug_handler]
pub async fn login(
    mut auth_session: AuthSession,
    State(_db): State<Client>,
    WithRejection(Json(value), _): WithRejection<Json<Value>, ApiError>,
) -> AxumResult<Json<User>> {
    let login_data: LoginReqData =
        serde_json::from_value(value).map_err(ApiError::JsonExtractionError)?;
    login_data.validate().map_err(ApiError::ValidationErrors)?;
    info!("Validated request data: {:?}", login_data);
    let cred = Credentials {
        email: login_data.email.clone(),
        password: login_data.password.expose_secret().clone(),
    };
    let user = auth_session
        .authenticate(cred)
        .await
        .map_err(|e| ApiError::LoginError(e.to_string()))?
        .ok_or_else(|| {
            info!("Failed to authenticate");
            ApiError::LoginError("Wrong email or password".into())
        })?;
    // let user = stores::user::get_user_by_email(&login_data.email, &db)
    //     .await
    //     .map_err(ApiError::EdgeDBQueryError)?
    //     .ok_or_else(|| {
    //         tracing::info!("User not found");
    //         let resp: ApiErrorShape = "User not found".to_string().into();
    //         (StatusCode::UNAUTHORIZED, Json(resp))
    //     })?;
    // let passwd_check = check_password(login_data.password.expose_secret(), &user.password)
    //     .map_err(|e| {
    //         tracing::error!("Error checking password: {:?}", e);
    //         ApiError::LoginError("Wrong password".into())
    //     })?;
    // tracing::info!("Password check: {:?}", passwd_check);
    // passwd_check
    //     .then_some(())
    //     .ok_or(ApiError::LoginError("Wrong password".into()))?;
    info!("Logging in user: {:?}", user);
    auth_session.login(&user).await.map_err(|e| {
        tracing::error!("Error logging in user: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(user))
}

pub async fn logout(mut auth_session: AuthSession) -> AxumResult<()> {
    auth_session
        .logout()
        .map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}
