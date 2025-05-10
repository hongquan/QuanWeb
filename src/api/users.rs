use axum::response::Result as AxumResult;
use axum::Json;
use axum::extract::State;
use gel_tokio::Client as EdgeClient;

use crate::stores;
use crate::models::users::MiniUser;

use super::errors::ApiError;

pub async fn list_users(State(db): State<EdgeClient>) -> AxumResult<Json<Vec<MiniUser>>> {
    let users = stores::user::list_mini_users(&db).await.map_err(ApiError::GelQueryError)?;
    Ok(Json(users))
}
