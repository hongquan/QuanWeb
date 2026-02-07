use std::num::NonZeroU16;

use axum::extract::{OriginalUri, Path, Query, State};
use axum::response::Html;
use axum::{Json, http::StatusCode, response::Result as AxumResult};
use axum_extra::extract::WithRejection;
use gel_tokio::Client as EdgeClient;
use serde_json::{Map as JMap, Value};
use slugrs::slugify;
use uuid::Uuid;
use validify::Validify;

use super::errors::ApiError;
pub use super::minors::{
    create_book, create_book_author, create_presentation, delete_book, delete_book_author,
    delete_presentation, get_book, get_book_author, get_presentation, list_book_authors,
    list_books, list_presentations, update_book_author_partial, update_book_partial,
    update_presentation_partial,
};
use super::paging::gen_pagination_links;
pub use super::posts::{create_post, delete_post, get_post, list_posts, update_post_partial};
use super::structs::{BlogCategoryCreateData, BlogCategoryPatchData, CategoryListQuery, ObjectListResponse};
pub use super::users::list_users;
use crate::auth::AuthSession;
use crate::consts::DEFAULT_PAGE_SIZE;
use crate::models::{BlogCategory, MinimalObject, User};
use crate::stores;
use crate::types::{AppState, EdgeSelectable};
use crate::utils::markdown::{markdown_to_html, markdown_to_html_document};

pub async fn root() -> &'static str {
    "API root"
}

pub async fn show_me(auth_session: AuthSession) -> AxumResult<Json<User>> {
    tracing::info!("Current user: {:?}", auth_session.user);
    let user = auth_session.user.ok_or(ApiError::Unauthorized)?;
    Ok(Json(user))
}

pub async fn list_categories(
    Query(query): Query<CategoryListQuery>,
    OriginalUri(original_uri): OriginalUri,
    State(db): State<EdgeClient>,
) -> AxumResult<Json<ObjectListResponse<BlogCategory>>> {
    let CategoryListQuery { page, per_page, sort } = query;
    let page = page.unwrap_or(NonZeroU16::MIN);
    let per_page = per_page.unwrap_or(DEFAULT_PAGE_SIZE);
    let offset = ((page.get() - 1) * per_page as u16) as i64;
    let limit = per_page as i64;
    let sort_by_featured = matches!(sort, Some(super::structs::CategorySort::FeaturedOrder));
    let categories = stores::blog::get_blog_categories(Some(offset), Some(limit), sort_by_featured, &db)
        .await
        .map_err(ApiError::GelQueryError)?;
    let count = stores::blog::get_all_categories_count(&db)
        .await
        .map_err(ApiError::GelQueryError)?;
    tracing::debug!("All categories count: {}", count);
    let total_pages =
        NonZeroU16::new((count as f64 / per_page as f64).ceil() as u16).unwrap_or(NonZeroU16::MIN);
    tracing::debug!("Total pages: {}", total_pages);
    // Create a paging struct for link generation
    let paging = super::structs::NPaging { page: Some(page), per_page: Some(per_page) };
    let links = gen_pagination_links(&paging, count, original_uri);
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
        .map_err(ApiError::GelQueryError)?
        .ok_or(ApiError::ObjectNotFound("BlogCategory".into()))?;
    Ok(Json(category))
}

pub async fn delete_category(
    Path(category_id): Path<Uuid>,
    auth_session: AuthSession,
    State(db): State<EdgeClient>,
) -> AxumResult<StatusCode> {
    auth_session.user.ok_or(ApiError::Unauthorized)?;
    let q = "DELETE BlogCategory FILTER .id = <uuid>$0";
    tracing::debug!("To query: {}", q);
    let _deleted_cat: MinimalObject = db
        .query_single(q, &(category_id,))
        .await
        .map_err(ApiError::GelQueryError)?
        .ok_or(ApiError::ObjectNotFound("BlogCategory".into()))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_category_partial(
    WithRejection(Path(category_id), _): WithRejection<Path<Uuid>, ApiError>,
    auth_session: AuthSession,
    State(db): State<EdgeClient>,
    WithRejection(Json(value), _): WithRejection<Json<Value>, ApiError>,
) -> AxumResult<Json<BlogCategory>> {
    auth_session.user.ok_or(ApiError::Unauthorized)?;
    // Collect list of submitted fields
    let jdata: JMap<String, Value> =
        serde_json::from_value(value.clone()).map_err(ApiError::JsonExtractionError)?;
    // User submit no field to update
    if jdata.is_empty() {
        let post = stores::blog::get_category(category_id, &db)
            .await
            .map_err(ApiError::GelQueryError)?;
        let post = post.ok_or(ApiError::ObjectNotFound("BlogCategory".into()))?;
        return Ok(Json(post));
    };
    // Check that data has invalid fields
    let patch_data: BlogCategoryPatchData =
        serde_json::from_value(value).map_err(ApiError::JsonExtractionError)?;
    let submitted_fields: Vec<&String> = jdata.keys().collect();
    let set_clause = patch_data.gen_set_clause(&submitted_fields);
    let args = patch_data.make_edgedb_args(category_id, &submitted_fields);
    let fields = BlogCategory::fields_as_shape();
    let q = format!(
        "SELECT (
            UPDATE BlogCategory
            FILTER .id = <uuid>$id
            SET {{
                {set_clause}
            }}
        ) {fields}"
    );
    tracing::debug!("To query: {}", q);
    tracing::debug!("With args: {:#?}", args);
    let cat = db
        .query_single(&q, &args)
        .await
        .map_err(ApiError::GelQueryError)?
        .ok_or(ApiError::ObjectNotFound("BlogCategory".into()))?;
    Ok(Json(cat))
}

pub async fn create_category(
    auth_session: AuthSession,
    State(db): State<EdgeClient>,
    WithRejection(Json(value), _): WithRejection<Json<Value>, ApiError>,
) -> AxumResult<(StatusCode, Json<BlogCategory>)> {
    auth_session.user.ok_or(ApiError::Unauthorized)?;
    // Collect list of submitted fields
    let jdata: JMap<String, Value> =
        serde_json::from_value(value.clone()).map_err(ApiError::JsonExtractionError)?;
    // User submitted no field to create BlogPost
    (!jdata.is_empty())
        .then_some(())
        .ok_or(ApiError::NotEnoughData)?;
    // Check that data has valid fields
    let mut post_data: BlogCategoryCreateData =
        serde_json::from_value(value).map_err(ApiError::JsonExtractionError)?;
    tracing::debug!("Post data: {:?}", post_data);
    post_data.validify().map_err(ApiError::ValidationErrors)?;
    let set_clause = post_data.gen_set_clause();
    let fields = BlogCategory::fields_as_shape();
    let args = post_data.make_edgedb_args();
    let q = format!(
        "
    SELECT (
        INSERT BlogCategory {{
            {set_clause}
        }}
    ) {fields}"
    );
    tracing::debug!("To query: {}", q);
    tracing::debug!("With args: {:#?}", args);
    let created_cat: BlogCategory = db
        .query_single(&q, &args)
        .await
        .map_err(ApiError::GelQueryError)?
        .ok_or(ApiError::Other("Failed to create BlogCategory".into()))?;
    Ok((StatusCode::CREATED, Json(created_cat)))
}

pub async fn convert_to_html(body: String) -> AxumResult<Html<String>> {
    let html = markdown_to_html(&body);
    Ok(Html(html))
}

#[axum::debug_handler]
pub async fn convert_to_html_document(
    State(app_state): State<AppState>,
    body: String,
) -> AxumResult<Html<String>> {
    let AppState { jinja, .. } = app_state;
    let html = markdown_to_html_document(&body, jinja).unwrap_or_default();
    Ok(Html(html))
}

pub async fn generate_slug(body: String) -> AxumResult<String> {
    let slug = slugify(body.as_str());
    Ok(slug)
}
