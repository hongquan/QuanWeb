use std::cmp::max;
use uuid::Uuid;

use axum::extract::{OriginalUri, Path, State};
use axum::http::StatusCode;
use axum::{response::Result as AxumResult, Json};
use axum_extra::extract::{Query, WithRejection};
use edgedb_errors::display::display_error_verbose;

use super::auth::Auth;
use super::errors::ApiError;
use super::paging::gen_pagination_links;
use super::structs::{ObjectListResponse, Paging};
use crate::consts::DEFAULT_PAGE_SIZE;
use crate::models::{MinimalObject, RawBlogPost, User};
use crate::retrievers::get_all_posts_count;
use crate::types::SharedState;

pub async fn root() -> &'static str {
    "API root"
}

pub async fn show_me(auth: Auth) -> AxumResult<Json<User>> {
    tracing::info!("Current user: {:?}", auth.current_user);
    let user = auth.current_user.ok_or(StatusCode::UNAUTHORIZED)?;
    Ok(Json(user))
}

pub async fn list_posts(
    paging: Query<Paging>,
    OriginalUri(original_uri): OriginalUri,
    State(state): State<SharedState>,
) -> Result<Json<ObjectListResponse<RawBlogPost>>, StatusCode> {
    tracing::info!("Paging: {:?}", paging);
    let page = max(1, paging.0.page.unwrap_or(1));
    let per_page = max(0, paging.0.per_page.unwrap_or(DEFAULT_PAGE_SIZE));
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
        categories: {
            id,
            title,
            slug,
        },
    }
    ORDER BY .created_at DESC EMPTY FIRST OFFSET <optional int64>$0 LIMIT <optional int64>$1";
    tracing::debug!("To query: {}", q);
    let posts: Vec<RawBlogPost> = db_conn.query(q, &(offset, limit)).await.map_err(|e| {
        tracing::error!("Error querying EdgeDB: {}", display_error_verbose(&e));
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let count = get_all_posts_count(&db_conn).await.map_err(|e| {
        tracing::error!("Error querying EdgeDB: {}", display_error_verbose(&e));
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let links = gen_pagination_links(&paging.0, count, original_uri);
    let resp = ObjectListResponse::new(posts)
        .with_count(count)
        .with_pagination_links(links);
    Ok(Json(resp))
}

pub async fn get_post(
    WithRejection(Path(post_id), _): WithRejection<Path<Uuid>, ApiError>,
    State(state): State<SharedState>,
) -> AxumResult<Json<RawBlogPost>> {
    let db_conn = &state.db;
    let q = "
    SELECT BlogPost {
        id,
        title,
        is_published,
        published_at,
        created_at,
        updated_at,
        categories: {
            id,
            title,
            slug,
        },
    }
    FILTER .id = <uuid>$0";
    tracing::debug!("To query: {}", q);
    let post: Option<RawBlogPost> = db_conn.query_single(q, &(post_id,)).await.map_err(|e| {
        tracing::error!("Error querying EdgeDB: {}", display_error_verbose(&e));
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(post.ok_or(StatusCode::NOT_FOUND)?))
}

pub async fn delete_post(
    Path(post_id): Path<Uuid>,
    auth: Auth,
    State(state): State<SharedState>,
) -> AxumResult<Json<Option<MinimalObject>>> {
    tracing::info!("Current user: {:?}", auth.current_user);
    auth.current_user.ok_or(StatusCode::FORBIDDEN)?;
    let q = "DELETE BlogPost FILTER .id = <uuid>$0";
    tracing::debug!("To query: {}", q);
    let db_conn = &state.db;
    let deleted_post: Option<MinimalObject> =
        db_conn.query_single(q, &(post_id,)).await.map_err(|e| {
            tracing::error!("Error querying EdgeDB: {}", display_error_verbose(&e));
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(deleted_post))
}
