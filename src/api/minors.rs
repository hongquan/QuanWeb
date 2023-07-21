use std::num::NonZeroU16;

use axum::extract::{OriginalUri, Path, Query, State};
use axum::{http::StatusCode, response::Result as AxumResult, Json};
use axum_extra::extract::WithRejection;
use edgedb_tokio::Client as EdgeClient;
use serde_json::{Map as JMap, Value};
use uuid::Uuid;

use super::errors::ApiError;
use super::paging::gen_pagination_links;
use super::structs::{NPaging, ObjectListResponse, PresentationPatchData};
use crate::auth::Auth;
use crate::consts::DEFAULT_PAGE_SIZE;
use crate::models::Presentation;
use crate::stores;

pub async fn list_presentations(
    Query(paging): Query<NPaging>,
    OriginalUri(original_uri): OriginalUri,
    State(db): State<EdgeClient>,
) -> AxumResult<Json<ObjectListResponse<Presentation>>> {
    let NPaging { page, per_page } = paging;
    let page = page.unwrap_or(NonZeroU16::MIN);
    let per_page = per_page.unwrap_or(DEFAULT_PAGE_SIZE);
    let offset = ((page.get() - 1) * (per_page as u16)) as i64;
    let limit = per_page as i64;
    let presentations = stores::minors::get_presentations(Some(offset), Some(limit), &db)
        .await
        .map_err(ApiError::EdgeDBQueryError)?;
    let count = stores::minors::get_all_presentations_count(&db)
        .await
        .map_err(ApiError::EdgeDBQueryError)?;
    let total_pages =
        NonZeroU16::new((count as f64 / per_page as f64).ceil() as u16).unwrap_or(NonZeroU16::MIN);
    let links = gen_pagination_links(&paging.into(), count as usize, original_uri);
    let resp = ObjectListResponse {
        objects: presentations,
        count: count as usize,
        total_pages,
        links,
    };
    Ok(Json(resp))
}

pub async fn get_presentation(
    WithRejection(Path(id), _): WithRejection<Path<Uuid>, ApiError>,
    State(db): State<EdgeClient>,
) -> AxumResult<Json<Presentation>> {
    let presentation = stores::minors::get_presentation(id, &db)
        .await
        .map_err(ApiError::EdgeDBQueryError)?
        .ok_or(ApiError::ObjectNotFound("Presentation".into()))?;
    Ok(Json(presentation))
}

pub async fn update_presentation_partial(
    WithRejection(Path(id), _): WithRejection<Path<Uuid>, ApiError>,
    auth: Auth,
    State(db): State<EdgeClient>,
    WithRejection(Json(value), _): WithRejection<Json<Value>, ApiError>,
) -> AxumResult<Json<Presentation>> {
    auth.current_user.ok_or_else(|| {
        tracing::debug!("Not logged in!");
        StatusCode::FORBIDDEN
    })?;
    // Collect list of submitted fields
    let jdata: JMap<String, Value> =
        serde_json::from_value(value.clone()).map_err(ApiError::JsonExtractionError)?;
    // User submit no field to update
    if jdata.is_empty() {
        let obj = stores::minors::get_presentation(id, &db)
            .await
            .map_err(ApiError::EdgeDBQueryError)?
            .ok_or(ApiError::ObjectNotFound("Presentation".into()))?;
        return Ok(Json(obj));
    }
    let patch_data: PresentationPatchData =
        serde_json::from_value(value).map_err(ApiError::JsonExtractionError)?;
    let submitted_fields: Vec<&String> = jdata.keys().collect();
    let set_clause = patch_data.gen_set_clause(&submitted_fields);
    let args = patch_data.make_edgedb_object(id, &submitted_fields);
    let q = format!(
        "SELECT (
            UPDATE Presentation FILTER .id = <uuid>$id SET {{ {set_clause} }}
        ) {{
            id,
            title,
            url,
            event,
        }}"
    );
    let presentation: Presentation = db
        .query_single(&q, &args)
        .await
        .map_err(ApiError::EdgeDBQueryError)?
        .ok_or(ApiError::ObjectNotFound("Presentation".into()))?;
    Ok(Json(presentation))
}
