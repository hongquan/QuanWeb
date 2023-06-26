use std::cmp::max;

use axum::http::StatusCode;
use axum::Json;
use axum::extract::State;
use axum_extra::extract::Query;
use edgedb_errors::display::display_error_verbose;

use crate::models::{BlogPost, RawBlogPost, User};
use crate::types::SharedState;
use super::structs::{Paging, ObjectListResponse};
use super::auth::Auth;

pub async fn root() -> &'static str {
    "API root"
}

pub async fn show_me(auth: Auth) -> axum::response::Result<Json<User>> {
    tracing::info!("Current user: {:?}", auth.current_user);
    let user = auth.current_user.ok_or(StatusCode::UNAUTHORIZED)?;
    Ok(Json(user))
}

pub async fn list_posts(paging: Query<Paging>, State(state): State<SharedState>) -> Result<Json<ObjectListResponse<BlogPost>>, StatusCode> {
    tracing::info!("Paging: {:?}", paging);
    let page = max(1, paging.0.page.unwrap_or(1));
    let per_page = max(0, paging.0.per_page.unwrap_or(10));
    let offset: i64 = ((page - 1) * per_page).try_into().unwrap_or(0);
    let limit = per_page as i64;
    let db_conn = &state.db;
    let q = "
    SELECT BlogPost {
        id,
        title,
        is_published,
        published_at,
        created_at,
        updated_at,
    }
    ORDER BY .created_at DESC EMPTY FIRST OFFSET <optional int64>$0 LIMIT <optional int64>$1";
    tracing::debug!("To query: {}", q);
    let posts: Vec<RawBlogPost> = db_conn.query(q, &(offset, limit)).await.map_err(|e| {
        tracing::error!("Error querying EdgeDB: {}", display_error_verbose(&e));
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let posts: Vec<BlogPost> = posts.into_iter().collect();
    let resp = ObjectListResponse::new(posts);
    Ok(Json(resp))
}
