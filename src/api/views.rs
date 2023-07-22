use std::cmp::max;
use std::num::NonZeroU16;

use axum::extract::{OriginalUri, Path, Query, State};
use axum::response::Html;
use axum::{http::StatusCode, response::Result as AxumResult, Json};
use axum_extra::extract::WithRejection;
use edgedb_tokio::Client as EdgeClient;
use garde::Validate;
use serde_json::{Map as JMap, Value};
use uuid::Uuid;

use super::errors::ApiError;
pub use super::minors::{
    create_presentation, delete_presentation, get_presentation, list_presentations,
    update_presentation_partial,
    list_book_authors, get_book_author, update_book_author_partial, delete_book_author,
    create_book_author,
};
use super::paging::gen_pagination_links;
pub use super::posts::{create_post, delete_post, get_post, list_posts, update_post_partial};
use super::structs::{BlogCategoryCreateData, BlogCategoryPatchData, ObjectListResponse, Paging};
use crate::auth::Auth;
use crate::consts::DEFAULT_PAGE_SIZE;
use crate::models::{BlogCategory, MinimalObject, User};
use crate::stores;
use crate::utils::markdown::markdown_to_html;

pub async fn root() -> &'static str {
    "API root"
}

pub async fn show_me(auth: Auth) -> AxumResult<Json<User>> {
    tracing::info!("Current user: {:?}", auth.current_user);
    let user = auth.current_user.ok_or(ApiError::Unauthorized)?;
    Ok(Json(user))
}

pub async fn list_categories(
    paging: Query<Paging>,
    OriginalUri(original_uri): OriginalUri,
    State(db): State<EdgeClient>,
) -> AxumResult<Json<ObjectListResponse<BlogCategory>>> {
    let page = max(1, paging.0.page.unwrap_or(1));
    let per_page = max(0, paging.0.per_page.unwrap_or(DEFAULT_PAGE_SIZE)) as u16;
    let offset: i64 = ((page - 1) * per_page).try_into().unwrap_or(0);
    let limit = per_page as i64;
    let categories = stores::blog::get_blog_categories(Some(offset), Some(limit), &db)
        .await
        .map_err(ApiError::EdgeDBQueryError)?;
    let count = stores::blog::get_all_categories_count(&db)
        .await
        .map_err(ApiError::EdgeDBQueryError)?;
    tracing::debug!("All categories count: {}", count);
    let total_pages =
        NonZeroU16::new((count as f64 / per_page as f64).ceil() as u16).unwrap_or(NonZeroU16::MIN);
    tracing::debug!("Total pages: {}", total_pages);
    let links = gen_pagination_links(&paging.0, count, original_uri);
    tracing::debug!("Links: {:?}", links);
    let resp = ObjectListResponse {
        objects: categories,
        count,
        total_pages,
        links,
    };
    Ok(Json(resp))
}

pub async fn get_category(
    WithRejection(Path(category_id), _): WithRejection<Path<Uuid>, ApiError>,
    State(db): State<EdgeClient>,
) -> AxumResult<Json<BlogCategory>> {
    let category = stores::blog::get_category(category_id, &db)
        .await
        .map_err(ApiError::EdgeDBQueryError)?
        .ok_or(ApiError::ObjectNotFound("BlogCategory".into()))?;
    Ok(Json(category))
}

pub async fn delete_category(
    Path(category_id): Path<Uuid>,
    auth: Auth,
    State(db): State<EdgeClient>,
) -> AxumResult<StatusCode> {
    tracing::info!("Current user: {:?}", auth.current_user);
    auth.current_user.ok_or(StatusCode::FORBIDDEN)?;
    let q = "DELETE BlogCategory FILTER .id = <uuid>$0";
    tracing::debug!("To query: {}", q);
    let _deleted_cat: MinimalObject = db
        .query_single(q, &(category_id,))
        .await
        .map_err(ApiError::EdgeDBQueryError)?
        .ok_or(ApiError::ObjectNotFound("BlogCategory".into()))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_category_partial(
    WithRejection(Path(category_id), _): WithRejection<Path<Uuid>, ApiError>,
    auth: Auth,
    State(db): State<EdgeClient>,
    WithRejection(Json(value), _): WithRejection<Json<Value>, ApiError>,
) -> AxumResult<Json<BlogCategory>> {
    auth.current_user.ok_or_else(|| {
        tracing::debug!("Not logged in!");
        StatusCode::FORBIDDEN
    })?;
    // Collect list of submitted fields
    let jdata: JMap<String, Value> =
        serde_json::from_value(value.clone()).map_err(ApiError::JsonExtractionError)?;
    // User submit no field to update
    if jdata.is_empty() {
        let post = stores::blog::get_category(category_id, &db)
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
    tracing::debug!("With args: {:#?}", args);
    let cat = db
        .query_single(&q, &args)
        .await
        .map_err(ApiError::EdgeDBQueryError)?
        .ok_or(ApiError::ObjectNotFound("BlogCategory".into()))?;
    Ok(Json(cat))
}

pub async fn create_category(
    auth: Auth,
    State(db): State<EdgeClient>,
    WithRejection(Json(value), _): WithRejection<Json<Value>, ApiError>,
) -> AxumResult<(StatusCode, Json<BlogCategory>)> {
    auth.current_user.ok_or(StatusCode::FORBIDDEN)?;
    // Collect list of submitted fields
    let jdata: JMap<String, Value> =
        serde_json::from_value(value.clone()).map_err(ApiError::JsonExtractionError)?;
    // User submitted no field to create BlogPost
    (!jdata.is_empty())
        .then_some(())
        .ok_or(ApiError::NotEnoughData)?;
    // Check that data has valid fields
    let post_data: BlogCategoryCreateData =
        serde_json::from_value(value).map_err(ApiError::JsonExtractionError)?;
    tracing::debug!("Post data: {:?}", post_data);
    post_data.validate(&()).map_err(ApiError::ValidationError)?;
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
    let created_cat: BlogCategory = db
        .query_single(&q, &args)
        .await
        .map_err(ApiError::EdgeDBQueryError)?
        .ok_or(ApiError::Other("Failed to create BlogCategory".into()))?;
    Ok((StatusCode::CREATED, Json(created_cat)))
}

pub async fn convert_to_html(body: String) -> AxumResult<Html<String>> {
    let html = markdown_to_html(&body);
    Ok(Html(html))
}
