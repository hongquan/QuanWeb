use std::cmp::max;

use axum::extract::{OriginalUri, Path, State};
use axum::{http::StatusCode, response::Result as AxumResult, Json};
use axum_extra::extract::{Query, WithRejection};
use edgedb_protocol::value::Value as EValue;
use indexmap::{indexmap, IndexMap};
use serde_json::{Map as JMap, Value};
use uuid::Uuid;

use super::auth::Auth;
use super::errors::ApiError;
use super::paging::gen_pagination_links;
use super::structs::{BlogPostPatchData, ObjectListResponse, Paging};
use crate::consts::DEFAULT_PAGE_SIZE;
use crate::models::{MinimalObject, RawBlogPost, User};
use crate::retrievers::{self, get_all_posts_count};
use crate::types::{json_value_to_edgedb, SharedState, build_edgedb_object};

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
) -> AxumResult<Json<ObjectListResponse<RawBlogPost>>> {
    tracing::info!("Paging: {:?}", paging);
    let page = max(1, paging.0.page.unwrap_or(1));
    let per_page = max(0, paging.0.per_page.unwrap_or(DEFAULT_PAGE_SIZE));
    let offset: i64 = ((page - 1) * per_page).try_into().unwrap_or(0);
    let limit = per_page as i64;
    let db_conn = &state.db;
    let posts = retrievers::get_blogposts(Some(offset), Some(limit), db_conn)
        .await
        .map_err(ApiError::EdgeDBQueryError)?;
    let count = get_all_posts_count(&db_conn)
        .await
        .map_err(ApiError::EdgeDBQueryError)?;
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
    let post = retrievers::get_blogpost(post_id, db_conn)
        .await
        .map_err(ApiError::EdgeDBQueryError)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(post))
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
    let deleted_post: Option<MinimalObject> = db_conn
        .query_single(q, &(post_id,))
        .await
        .map_err(ApiError::EdgeDBQueryError)?;
    Ok(Json(deleted_post))
}

pub async fn update_post_partial(
    WithRejection(Path(post_id), _): WithRejection<Path<Uuid>, ApiError>,
    auth: Auth,
    State(state): State<SharedState>,
    WithRejection(Json(value), _): WithRejection<Json<Value>, ApiError>,
) -> AxumResult<Json<RawBlogPost>> {
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
        let post = post.ok_or(StatusCode::NOT_FOUND)?;
        return Ok(Json(post));
    };
    // Check that data has invalid fields
    let _patch_data: BlogPostPatchData =
        serde_json::from_value(value).map_err(ApiError::JsonExtractionError)?;
    let mut eql_params = indexmap! {
        "id" => EValue::Uuid(post_id),
    };
    let valid_fields = BlogPostPatchData::fields();
    let values_to_update = jdata.iter().filter_map(|(field_name, v)| {
        valid_fields.contains(&field_name.as_str()).then(|| {
            let value = json_value_to_edgedb(v);
            (field_name.as_str(), value)
        })
    });
    eql_params.extend(values_to_update);
    tracing::debug!("EQL params: {:?}", eql_params);
    // Build Value::Object to use as QueryArgs
    let args_obj = build_edgedb_object(&eql_params);
    let set_clause = gen_set_clause(&eql_params);
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
            is_published,
            published_at,
            created_at,
            updated_at,
            categories: {{
                id,
                title,
                slug,
            }},
        }}"
    );
    tracing::debug!("To query: {}", q);
    tracing::debug!("Query with params: {:?}", args_obj);
    let updated_post: Option<RawBlogPost> = db_conn
        .query_single(&q, &args_obj)
        .await
        .map_err(ApiError::EdgeDBQueryError)?;
    let updated_post = updated_post.ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(updated_post))
}

fn gen_set_clause(params: &IndexMap<&str, EValue>) -> String {
    params
        .get_range(1..)
        .map(|entries| {
            entries
                .iter()
                .map(|(field_name, _v)| {
                    let etype = RawBlogPost::type_cast_for_field(field_name);
                    let statement = format!("{field_name} := <{etype}>${field_name}");
                    statement
                })
                .collect::<Vec<String>>()
                .join(",\n    ")
        })
        .unwrap_or_default()
}
