use std::cmp::max;

use axum::extract::{OriginalUri, Path, State};
use axum::{http::StatusCode, response::Result as AxumResult, Json};
use axum_extra::extract::{Query, WithRejection};
use serde_json::{Map as JMap, Value};
use uuid::Uuid;

use super::auth::Auth;
use super::errors::ApiError;
use super::paging::gen_pagination_links;
use super::structs::{BlogCategoryCreateData, BlogCategoryPatchData, ObjectListResponse, Paging};
use crate::consts::DEFAULT_PAGE_SIZE;
use crate::models::{BlogCategory, MinimalObject, User};
use crate::retrievers;
use crate::types::SharedState;

pub use super::posts::{create_post, delete_post, get_post, list_posts, update_post_partial};

pub async fn root() -> &'static str {
    "API root"
}

pub async fn show_me(auth: Auth) -> AxumResult<Json<User>> {
    tracing::info!("Current user: {:?}", auth.current_user);
    let user = auth.current_user.ok_or(StatusCode::UNAUTHORIZED)?;
    Ok(Json(user))
}

pub async fn list_categories(
    paging: Query<Paging>,
    OriginalUri(original_uri): OriginalUri,
    State(state): State<SharedState>,
) -> AxumResult<Json<ObjectListResponse<BlogCategory>>> {
    let page = max(1, paging.0.page.unwrap_or(1));
    let per_page = max(0, paging.0.per_page.unwrap_or(DEFAULT_PAGE_SIZE)) as u16;
    let offset: i64 = ((page - 1) * per_page).try_into().unwrap_or(0);
    let limit = per_page as i64;
    let db_conn = &state.db;
    let categories = retrievers::get_blog_categories(Some(offset), Some(limit), db_conn)
        .await
        .map_err(ApiError::EdgeDBQueryError)?;
    let count = retrievers::get_all_categories_count(db_conn)
        .await
        .map_err(ApiError::EdgeDBQueryError)?;
    let links = gen_pagination_links(&paging.0, count, original_uri);
    let resp = ObjectListResponse::new(categories)
        .with_count(count)
        .with_pagination_links(links);
    Ok(Json(resp))
}

pub async fn get_category(
    WithRejection(Path(category_id), _): WithRejection<Path<Uuid>, ApiError>,
    State(state): State<SharedState>,
) -> AxumResult<Json<BlogCategory>> {
    let db_conn = &state.db;
    let category = retrievers::get_blog_category(category_id, db_conn)
        .await
        .map_err(ApiError::EdgeDBQueryError)?
        .ok_or(ApiError::ObjectNotFound("BlogCategory".into()))?;
    Ok(Json(category))
}

pub async fn delete_category(
    Path(category_id): Path<Uuid>,
    auth: Auth,
    State(state): State<SharedState>,
) -> AxumResult<Json<MinimalObject>> {
    tracing::info!("Current user: {:?}", auth.current_user);
    auth.current_user.ok_or(StatusCode::FORBIDDEN)?;
    let q = "DELETE BlogCategory FILTER .id = <uuid>$0";
    tracing::debug!("To query: {}", q);
    let db_conn = &state.db;
    let deleted_cat: MinimalObject = db_conn
        .query_single(q, &(category_id,))
        .await
        .map_err(ApiError::EdgeDBQueryError)?
        .ok_or(ApiError::ObjectNotFound("BlogCategory".into()))?;
    Ok(Json(deleted_cat))
}

pub async fn update_category_partial(
    WithRejection(Path(category_id), _): WithRejection<Path<Uuid>, ApiError>,
    auth: Auth,
    State(state): State<SharedState>,
    WithRejection(Json(value), _): WithRejection<Json<Value>, ApiError>,
) -> AxumResult<Json<BlogCategory>> {
    auth.current_user.ok_or_else(|| {
        tracing::debug!("Not logged in!");
        StatusCode::FORBIDDEN
    })?;
    // Collect list of submitted fields
    let jdata: JMap<String, Value> =
        serde_json::from_value(value.clone()).map_err(ApiError::JsonExtractionError)?;
    let db_conn = &state.db;
    // User submit no field to update
    if jdata.is_empty() {
        let post = retrievers::get_blog_category(category_id, db_conn)
            .await
            .map_err(ApiError::EdgeDBQueryError)?;
        let post = post.ok_or(ApiError::ObjectNotFound("BlogCategory".into()))?;
        return Ok(Json(post));
    };
    // Check that data has invalid fields
    let patch_data: BlogCategoryPatchData =
        serde_json::from_value(value).map_err(ApiError::JsonExtractionError)?;
    let submitted_fields: Vec<&String> = jdata.keys().collect();
    let set_clause = patch_data.gen_set_clause(&submitted_fields);
    let args = patch_data.make_edgedb_object(category_id, &submitted_fields);
    let q = format!(
        "SELECT (
            UPDATE BlogCategory
            FILTER .id = <uuid>$id
            SET {{
                {set_clause}
            }}
        ) {{
            id,
            title,
            slug,
        }}"
    );
    tracing::debug!("To query: {}", q);
    let cat = db_conn.query_single(&q, &args).await
        .map_err(ApiError::EdgeDBQueryError)?
        .ok_or(ApiError::ObjectNotFound("BlogCategory".into()))?;
    Ok(Json(cat))
}

pub async fn create_category(
    auth: Auth,
    State(state): State<SharedState>,
    WithRejection(Json(value), _): WithRejection<Json<Value>, ApiError>,
) -> AxumResult<(StatusCode, Json<BlogCategory>)> {
    auth.current_user.ok_or(StatusCode::FORBIDDEN)?;
    // Collect list of submitted fields
    let jdata: JMap<String, Value> =
        serde_json::from_value(value.clone()).map_err(ApiError::JsonExtractionError)?;
    let db_conn = &state.db;
    // User submitted no field to create BlogPost
    (!jdata.is_empty())
        .then_some(())
        .ok_or(ApiError::NotEnoughData)?;
    // Check that data has valid fields
    let post_data: BlogCategoryCreateData =
        serde_json::from_value(value).map_err(ApiError::JsonExtractionError)?;
    tracing::debug!("Post data: {:?}", post_data);
    let set_clause = post_data.gen_set_clause();
    let args = post_data.make_edgedb_object();
    let q = format!(
        "
    SELECT (
        INSERT BlogCategory {{
            {set_clause}
        }}
    ) {{
        id,
        title,
        slug,
    }}"
    );
    tracing::debug!("To query: {}", q);
    let created_cat: BlogCategory = db_conn
        .query_single(&q, &args)
        .await
        .map_err(ApiError::EdgeDBQueryError)?
        .ok_or(ApiError::Other("Failed to create BlogCategory".into()))?;
    Ok((StatusCode::CREATED, Json(created_cat)))
}
