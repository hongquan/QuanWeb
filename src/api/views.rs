use std::cmp::max;

use axum::extract::{Host, OriginalUri, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::Query;
use edgedb_errors::display::display_error_verbose;
use url::Url;

use super::auth::Auth;
use super::paging::gen_pagination_links;
use super::structs::{ObjectListResponse, Paging};
use crate::models::{BlogPost, RawBlogPost, User};
use crate::retrievers::get_all_posts_count;
use crate::types::SharedState;

pub async fn root() -> &'static str {
    "API root"
}

pub async fn show_me(auth: Auth) -> axum::response::Result<Json<User>> {
    tracing::info!("Current user: {:?}", auth.current_user);
    let user = auth.current_user.ok_or(StatusCode::UNAUTHORIZED)?;
    Ok(Json(user))
}

pub async fn list_posts(
    paging: Query<Paging>,
    Host(hostname): Host,
    OriginalUri(original_uri): OriginalUri,
    State(state): State<SharedState>,
) -> Result<Json<ObjectListResponse<BlogPost>>, StatusCode> {
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
    let count = get_all_posts_count(&db_conn).await.map_err(|e| {
        tracing::error!("Error querying EdgeDB: {}", display_error_verbose(&e));
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let orig_url = format!("http://{hostname}{original_uri}");
    let base_url = Url::parse(orig_url.as_str()).map_err(|e| {
        tracing::error!("Error parsing URL: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let links = gen_pagination_links(&paging.0, count, base_url);
    let resp = ObjectListResponse::new(posts)
        .with_count(count)
        .with_pagination_links(links);
    Ok(Json(resp))
}
