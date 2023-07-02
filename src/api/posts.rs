use std::cmp::max;

use axum::extract::{OriginalUri, Path, State};
use axum::{http::StatusCode, response::Result as AxumResult, Json};
use axum_extra::extract::{Query, WithRejection};
use serde_json::{Map as JMap, Value};
use uuid::Uuid;

use super::auth::Auth;
use super::errors::ApiError;
use super::paging::gen_pagination_links;
use super::structs::{BlogPostCreateData, BlogPostPatchData, ObjectListResponse, Paging};
use crate::consts::DEFAULT_PAGE_SIZE;
use crate::models::{DetailedBlogPost, MinimalObject, RawBlogPost};
use crate::retrievers::{self, get_all_posts_count};
use crate::types::SharedState;

pub async fn list_posts(
    paging: Query<Paging>,
    OriginalUri(original_uri): OriginalUri,
    State(state): State<SharedState>,
) -> AxumResult<Json<ObjectListResponse<RawBlogPost>>> {
    tracing::info!("Paging: {:?}", paging);
    let page = max(1, paging.0.page.unwrap_or(1));
    let per_page = max(0, paging.0.per_page.unwrap_or(DEFAULT_PAGE_SIZE)) as u16;
    let offset: i64 = ((page - 1) * per_page).try_into().unwrap_or(0);
    let limit = per_page as i64;
    let db_conn = &state.db;
    let posts = retrievers::get_blogposts(Some(offset), Some(limit), db_conn)
        .await
        .map_err(ApiError::EdgeDBQueryError)?;
    let count = get_all_posts_count(&db_conn)
        .await
        .map_err(ApiError::EdgeDBQueryError)?;
    let total_pages = (count as f64 / per_page as f64).ceil() as u16;
    let links = gen_pagination_links(&paging.0, count, original_uri);
    let resp = ObjectListResponse::new(posts)
        .with_count(count)
        .with_total_pages(total_pages)
        .with_pagination_links(links);
    Ok(Json(resp))
}

pub async fn get_post(
    WithRejection(Path(post_id), _): WithRejection<Path<Uuid>, ApiError>,
    State(state): State<SharedState>,
) -> AxumResult<Json<DetailedBlogPost>> {
    let db_conn = &state.db;
    let post = retrievers::get_blogpost(post_id, db_conn)
        .await
        .map_err(ApiError::EdgeDBQueryError)?
        .ok_or(ApiError::ObjectNotFound("BlogPost".into()))?;
    Ok(Json(post))
}

pub async fn delete_post(
    Path(post_id): Path<Uuid>,
    auth: Auth,
    State(state): State<SharedState>,
) -> AxumResult<Json<MinimalObject>> {
    tracing::info!("Current user: {:?}", auth.current_user);
    auth.current_user.ok_or(StatusCode::FORBIDDEN)?;
    let q = "DELETE BlogPost FILTER .id = <uuid>$0";
    tracing::debug!("To query: {}", q);
    let db_conn = &state.db;
    let deleted_post: MinimalObject = db_conn
        .query_single(q, &(post_id,))
        .await
        .map_err(ApiError::EdgeDBQueryError)?
        .ok_or(ApiError::ObjectNotFound("BlogPost".into()))?;
    Ok(Json(deleted_post))
}

pub async fn update_post_partial(
    WithRejection(Path(post_id), _): WithRejection<Path<Uuid>, ApiError>,
    auth: Auth,
    State(state): State<SharedState>,
    WithRejection(Json(value), _): WithRejection<Json<Value>, ApiError>,
) -> AxumResult<Json<DetailedBlogPost>> {
    auth.current_user.ok_or(StatusCode::FORBIDDEN)?;
    // Collect list of submitted fields
    let jdata: JMap<String, Value> =
        serde_json::from_value(value.clone()).map_err(ApiError::JsonExtractionError)?;
    let db_conn = &state.db;
    // User submit no field to update
    if jdata.is_empty() {
        let post = retrievers::get_blogpost(post_id, db_conn)
            .await
            .map_err(ApiError::EdgeDBQueryError)?;
        let post = post.ok_or(ApiError::ObjectNotFound("BlogPost".into()))?;
        return Ok(Json(post));
    };
    // Check that data has invalid fields
    let patch_data: BlogPostPatchData =
        serde_json::from_value(value).map_err(ApiError::JsonExtractionError)?;
    let submitted_fields: Vec<&String> = jdata.keys().collect();
    let set_clause = patch_data.gen_set_clause(&submitted_fields);
    let args = patch_data.make_edgedb_object(post_id, &submitted_fields);
    let q = format!(
        "SELECT (
            UPDATE BlogPost
            FILTER .id = <uuid>$id
            SET {{
                {set_clause}
            }}
        ) {{
            id,
            title,
            slug,
            is_published,
            published_at,
            created_at,
            updated_at,
            categories: {{ id, title, slug }},
            body,
            format,
            locale,
            excerpt,
            html,
            seo_description,
            og_image,
        }}"
    );
    tracing::debug!("To query: {}", q);
    tracing::debug!("Query with params: {:?}", args);
    let updated_post: Option<DetailedBlogPost> = db_conn
        .query_single(&q, &args)
        .await
        .map_err(ApiError::EdgeDBQueryError)?;
    let updated_post = updated_post.ok_or(ApiError::ObjectNotFound("BlogPost".into()))?;
    Ok(Json(updated_post))
}

pub async fn create_post(
    auth: Auth,
    State(state): State<SharedState>,
    WithRejection(Json(value), _): WithRejection<Json<Value>, ApiError>,
) -> AxumResult<(StatusCode, Json<DetailedBlogPost>)> {
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
    let post_data: BlogPostCreateData =
        serde_json::from_value(value).map_err(ApiError::JsonExtractionError)?;
    tracing::debug!("Post data: {:?}", post_data);
    let submitted_fields: Vec<&String> = jdata.keys().collect();
    let set_clause = post_data.gen_set_clause(&submitted_fields);
    let args = post_data.make_edgedb_object(&submitted_fields);
    let q = format!(
        "
    SELECT (
        INSERT BlogPost {{
            {set_clause}
        }}
    ) {{
        id,
        title,
        slug,
        is_published,
        published_at,
        created_at,
        updated_at,
        categories: {{id, title, slug}},
        body,
        format,
        locale,
        excerpt,
        html,
        seo_description,
        og_image,
    }}"
    );
    tracing::debug!("To query: {}", q);
    tracing::debug!("Query with params: {:?}", args);
    let created_post: DetailedBlogPost = db_conn
        .query_single(&q, &args)
        .await
        .map_err(ApiError::EdgeDBQueryError)?
        .ok_or(ApiError::Other("Failed to create BlogPost".into()))?;
    Ok((StatusCode::CREATED, Json(created_post)))
}
