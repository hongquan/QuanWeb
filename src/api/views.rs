use std::cmp::max;

use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::Query;
use edgedb_errors::display::display_error_verbose;

use crate::db::get_edgedb_client;
use crate::models::{BlogPost, RawBlogPost, User};
use super::structs::Paging;
use super::auth::Auth;

pub async fn root() -> &'static str {
    "API root"
}

pub async fn show_me(auth: Auth) -> axum::response::Result<Json<User>> {
    tracing::info!("Current user: {:?}", auth.current_user);
    let user = auth.current_user.ok_or(StatusCode::UNAUTHORIZED)?;
    Ok(Json(user))
}

pub async fn list_posts(paging: Query<Paging>) -> Result<Json<Vec<BlogPost>>, StatusCode> {
    tracing::info!("Paging: {:?}", paging);
    let page = max(1, paging.0.page.unwrap_or(1));
    let per_page = max(0, paging.0.per_page.unwrap_or(10));
    let offset: i64 = ((page - 1) * per_page).try_into().unwrap_or(0);
    let limit = per_page as i64;
    let db_conn = get_edgedb_client().await.map_err(|e| {
        eprintln!("Error connecting to EdgeDB: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
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
    Ok(Json(posts.into_iter().collect()))
}
