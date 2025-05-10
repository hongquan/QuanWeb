use std::num::NonZeroU16;

use axum::extract::{OriginalUri, Path, Query, State};
use axum::{http::StatusCode, response::Result as AxumResult, Json};
use axum_extra::extract::WithRejection;
use gel_tokio::Client as EdgeClient;
use serde_json::{Map as JMap, Value};
use tracing::debug;
use uuid::Uuid;
use validify::Validify;

use super::errors::ApiError;
use super::paging::gen_pagination_links;
use super::structs::{
    BlogPostCreateData, BlogPostPatchData, NPaging, ObjectListResponse, OtherQuery,
};
use crate::auth::AuthSession;
use crate::consts::DEFAULT_PAGE_SIZE;
use crate::models::{DetailedBlogPost, MediumBlogPost, MinimalObject};
use crate::stores;
use crate::types::EdgeSelectable;
use crate::utils::split_search_query;

pub async fn list_posts(
    Query(paging): Query<NPaging>,
    Query(mut other_query): Query<OtherQuery>,
    OriginalUri(original_uri): OriginalUri,
    State(db): State<EdgeClient>,
) -> AxumResult<Json<ObjectListResponse<MediumBlogPost>>> {
    let NPaging { page, per_page } = paging;
    let page = page.unwrap_or(NonZeroU16::MIN);
    let per_page = per_page.unwrap_or(DEFAULT_PAGE_SIZE);
    let offset = ((page.get() - 1) * per_page as u16) as i64;
    let limit = per_page as i64;
    other_query.validify().map_err(ApiError::ValidationErrors)?;
    let search_tokens = split_search_query(other_query.q.as_deref());
    let lower_search_tokens: Option<Vec<String>> =
        search_tokens.map(|v| v.into_iter().map(|s| s.to_lowercase()).collect());
    let count = stores::blog::count_search_result_posts(lower_search_tokens.as_ref(), &db)
        .await
        .map_err(ApiError::GelQueryError)?;
    let posts =
        stores::blog::get_blogposts(lower_search_tokens.as_ref(), Some(offset), Some(limit), &db)
            .await
            .map_err(ApiError::GelQueryError)?;
    let total_pages =
        NonZeroU16::new((count as f64 / per_page as f64).ceil() as u16).unwrap_or(NonZeroU16::MIN);
    let links = gen_pagination_links(&paging, count, original_uri);
    let resp = ObjectListResponse {
        count,
        total_pages,
        links,
        objects: posts,
    };
    Ok(Json(resp))
}

pub async fn get_post(
    WithRejection(Path(post_id), _): WithRejection<Path<Uuid>, ApiError>,
    State(db): State<EdgeClient>,
) -> AxumResult<Json<DetailedBlogPost>> {
    let post = stores::blog::get_post(post_id, &db)
        .await
        .map_err(ApiError::GelQueryError)?
        .ok_or(ApiError::ObjectNotFound("BlogPost".into()))?;
    Ok(Json(post))
}

pub async fn delete_post(
    Path(post_id): Path<Uuid>,
    auth_session: AuthSession,
    State(db): State<EdgeClient>,
) -> AxumResult<StatusCode> {
    auth_session.user.ok_or(ApiError::Unauthorized)?;
    let q = "DELETE BlogPost FILTER .id = <uuid>$0";
    tracing::debug!("To query: {}", q);
    let _deleted_post: MinimalObject = db
        .query_single(q, &(post_id,))
        .await
        .map_err(ApiError::GelQueryError)?
        .ok_or(ApiError::ObjectNotFound("BlogPost".into()))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_post_partial(
    WithRejection(Path(post_id), _): WithRejection<Path<Uuid>, ApiError>,
    auth_session: AuthSession,
    State(db): State<EdgeClient>,
    WithRejection(Json(value), _): WithRejection<Json<Value>, ApiError>,
) -> AxumResult<Json<DetailedBlogPost>> {
    auth_session.user.ok_or(ApiError::Unauthorized)?;
    // Collect list of submitted fields
    let jdata: JMap<String, Value> =
        serde_json::from_value(value.clone()).map_err(ApiError::JsonExtractionError)?;
    // User submit no field to update
    if jdata.is_empty() {
        let post = stores::blog::get_post(post_id, &db)
            .await
            .map_err(ApiError::GelQueryError)?;
        let post = post.ok_or(ApiError::ObjectNotFound("BlogPost".into()))?;
        return Ok(Json(post));
    };
    // Check that data has invalid fields
    let patch_data: BlogPostPatchData =
        serde_json::from_value(value).map_err(ApiError::JsonExtractionError)?;
    let submitted_fields: Vec<&String> = jdata.keys().collect();
    let set_clause = patch_data.gen_set_clause(&submitted_fields);
    let fields = DetailedBlogPost::fields_as_shape();
    let args = patch_data.make_edgedb_args(post_id, &submitted_fields);
    let q = format!(
        "SELECT (
            UPDATE BlogPost
            FILTER .id = <uuid>$id
            SET {{
                {set_clause}
            }}
        ) {fields}"
    );
    debug!("To query: {q}");
    debug!("Query with params: {args:#?}");
    let updated_post: Option<DetailedBlogPost> = db
        .query_single(&q, &args)
        .await
        .map_err(ApiError::GelQueryError)?;
    let updated_post = updated_post.ok_or(ApiError::ObjectNotFound("BlogPost".into()))?;
    Ok(Json(updated_post))
}

pub async fn create_post(
    auth_session: AuthSession,
    State(db): State<EdgeClient>,
    WithRejection(Json(value), _): WithRejection<Json<Value>, ApiError>,
) -> AxumResult<(StatusCode, Json<DetailedBlogPost>)> {
    auth_session.user.ok_or(ApiError::Unauthorized)?;
    // Collect list of submitted fields
    let jdata: JMap<String, Value> =
        serde_json::from_value(value.clone()).map_err(ApiError::JsonExtractionError)?;
    // User submitted no field to create BlogPost
    (!jdata.is_empty())
        .then_some(())
        .ok_or(ApiError::NotEnoughData)?;
    // Check that data has valid fields
    let mut post_data: BlogPostCreateData =
        serde_json::from_value(value).map_err(ApiError::JsonExtractionError)?;
    post_data.validify().map_err(ApiError::ValidationErrors)?;
    tracing::debug!("Post data: {:?}", post_data);
    let submitted_fields: Vec<&String> = jdata.keys().collect();
    let set_clause = post_data.gen_set_clause(&submitted_fields);
    let fields = DetailedBlogPost::fields_as_shape();
    let args = post_data.make_edgedb_args(&submitted_fields);
    let q = format!(
        "
    SELECT (
        INSERT BlogPost {{
            {set_clause}
        }}
    ) {fields}"
    );
    tracing::debug!("To query: {}", q);
    tracing::debug!("Query with params: {:?}", args);
    let created_post: DetailedBlogPost = db
        .query_single(&q, &args)
        .await
        .map_err(ApiError::GelQueryError)?
        .ok_or(ApiError::Other("Failed to create BlogPost".into()))?;
    Ok((StatusCode::CREATED, Json(created_post)))
}
