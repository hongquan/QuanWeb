use axum::http::StatusCode;
use axum_login::extractors::AuthContext;
use uuid::Uuid;
use edgedb_errors::display::display_error_verbose;

use super::store::EdgeDbStore;
use crate::db::get_edgedb_client;
use crate::models;
use crate::retrievers;

pub type Auth = AuthContext<Uuid, models::User, EdgeDbStore<models::User>, models::Role>;

pub async fn login(mut auth: Auth) -> Result<(), StatusCode> {
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
    Ok(())
}
