use std::num::NonZeroU16;

use axum::extract::{OriginalUri, Path, Query, State};
use axum::{response::Result as AxumResult, Json};
use axum_extra::extract::WithRejection;
use edgedb_tokio::Client as EdgeClient;
use http::StatusCode;
use serde_json::{Map as JMap, Value};
use uuid::Uuid;
use validify::Validify;

use super::errors::ApiError;
use super::paging::gen_pagination_links;
use super::structs::{
    BookAuthorPatchData, NPaging, ObjectListResponse, PresentationCreateData, PresentationPatchData, BookPatchData, BookCreateData,
};
use crate::auth::Auth;
use crate::consts::DEFAULT_PAGE_SIZE;
use crate::models::minors::{BookAuthor, Book};
use crate::models::{MinimalObject, Presentation};
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
    auth.current_user.ok_or(ApiError::Unauthorized)?;
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

pub async fn create_presentation(
    auth: Auth,
    State(db): State<EdgeClient>,
    WithRejection(Json(value), _): WithRejection<Json<Value>, ApiError>,
) -> AxumResult<Json<Presentation>> {
    auth.current_user.ok_or(ApiError::Unauthorized)?;
    // Collect list of submitted fields
    let jdata: JMap<String, Value> =
        serde_json::from_value(value.clone()).map_err(ApiError::JsonExtractionError)?;
    // User submit no field to create Presentation
    (!jdata.is_empty())
        .then_some(())
        .ok_or(ApiError::NotEnoughData)?;
    let post_data: PresentationCreateData =
        serde_json::from_value(value).map_err(ApiError::JsonExtractionError)?;
    let post_data = PresentationCreateData::validify(post_data.into()).map_err(ApiError::ValidationErrors)?;
    let set_clause = post_data.gen_set_clause();
    let args = post_data.make_edgedb_object();
    let q = format!(
        "
    SELECT (
        INSERT Presentation {{
            {set_clause}
        }}
    ) {{
        id,
        title,
        url,
        event,
    }}"
    );
    let p: Presentation = db
        .query_single(&q, &args)
        .await
        .map_err(ApiError::EdgeDBQueryError)?
        .ok_or(ApiError::Other("Failed to create Presentation".into()))?;
    Ok(Json(p))
}

pub async fn delete_presentation(
    WithRejection(Path(id), _): WithRejection<Path<Uuid>, ApiError>,
    auth: Auth,
    State(db): State<EdgeClient>,
) -> AxumResult<StatusCode> {
    auth.current_user.ok_or(ApiError::Unauthorized)?;
    let q = "DELETE Presentation FILTER .id = <uuid>$0";
    let _p: MinimalObject = db
        .query_single(q, &(id,))
        .await
        .map_err(ApiError::EdgeDBQueryError)?
        .ok_or(ApiError::ObjectNotFound("Presentation".into()))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_book_authors(
    Query(paging): Query<NPaging>,
    OriginalUri(original_uri): OriginalUri,
    State(db): State<EdgeClient>,
) -> AxumResult<Json<ObjectListResponse<BookAuthor>>> {
    let NPaging { page, per_page } = paging;
    let page = page.unwrap_or(NonZeroU16::MIN);
    let per_page = per_page.unwrap_or(DEFAULT_PAGE_SIZE);
    let offset = ((page.get() - 1) * (per_page as u16)) as i64;
    let limit = per_page as i64;
    let authors = stores::minors::get_book_authors(Some(offset), Some(limit), &db)
        .await
        .map_err(ApiError::EdgeDBQueryError)?;
    let count = stores::minors::get_all_book_authors_count(&db)
        .await
        .map_err(ApiError::EdgeDBQueryError)?;
    let total_pages =
        NonZeroU16::new((count as f64 / per_page as f64).ceil() as u16).unwrap_or(NonZeroU16::MIN);
    let links = gen_pagination_links(&paging.into(), count as usize, original_uri);
    let resp = ObjectListResponse {
        objects: authors,
        count: count as usize,
        total_pages,
        links,
    };
    Ok(Json(resp))
}

pub async fn get_book_author(
    WithRejection(Path(id), _): WithRejection<Path<Uuid>, ApiError>,
    State(db): State<EdgeClient>,
) -> AxumResult<Json<BookAuthor>> {
    let author = stores::minors::get_book_author(id, &db)
        .await
        .map_err(ApiError::EdgeDBQueryError)?
        .ok_or(ApiError::ObjectNotFound("BookAuthor".into()))?;
    Ok(Json(author))
}

pub async fn update_book_author_partial(
    auth: Auth,
    WithRejection(Path(id), _): WithRejection<Path<Uuid>, ApiError>,
    State(db): State<EdgeClient>,
    WithRejection(Json(post_data), _): WithRejection<Json<BookAuthorPatchData>, ApiError>,
) -> AxumResult<Json<BookAuthor>> {
    auth.current_user.ok_or(ApiError::Unauthorized)?;
    let post_data = BookAuthorPatchData::validify(post_data.into()).map_err(ApiError::ValidationErrors)?;
    let q = "SELECT (
        UPDATE BookAuthor FILTER .id = <uuid>$0 SET {
            name := <str>$1,
        }
    ) { id, name }
    ";
    let author: BookAuthor = db
        .query_single(q, &(id, &post_data.name))
        .await
        .map_err(ApiError::EdgeDBQueryError)?
        .ok_or(ApiError::ObjectNotFound("BookAuthor".into()))?;
    Ok(Json(author))
}

pub async fn delete_book_author(
    auth: Auth,
    WithRejection(Path(id), _): WithRejection<Path<Uuid>, ApiError>,
    State(db): State<EdgeClient>,
) -> AxumResult<StatusCode> {
    auth.current_user.ok_or(ApiError::Unauthorized)?;
    let q = "DELETE BookAuthor FILTER .id = <uuid>$0";
    let _p: MinimalObject = db
        .query_single(q, &(id,))
        .await
        .map_err(ApiError::EdgeDBQueryError)?
        .ok_or(ApiError::ObjectNotFound("BookAuthor".into()))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn create_book_author(
    auth: Auth,
    State(db): State<EdgeClient>,
    WithRejection(Json(post_data), _): WithRejection<Json<BookAuthorPatchData>, ApiError>,
) -> AxumResult<Json<BookAuthor>> {
    auth.current_user.ok_or(ApiError::Unauthorized)?;
    let post_data = BookAuthorPatchData::validify(post_data.into()).map_err(ApiError::ValidationErrors)?;
    let q = "SELECT (
        INSERT BookAuthor {
            name := <str>$0,
        }
    ) { id, name }
    ";
    let author: BookAuthor = db
        .query_single(q, &(post_data.name,))
        .await
        .map_err(ApiError::EdgeDBQueryError)?
        .ok_or(ApiError::Other("Failed to create BookAuthor".into()))?;
    Ok(Json(author))
}

pub async fn list_books(
    Query(paging): Query<NPaging>,
    OriginalUri(original_uri): OriginalUri,
    State(db): State<EdgeClient>,
) -> AxumResult<Json<ObjectListResponse<Book>>> {
    let NPaging { page, per_page } = paging;
    let page = page.unwrap_or(NonZeroU16::MIN);
    let per_page = per_page.unwrap_or(DEFAULT_PAGE_SIZE);
    let offset = ((page.get() - 1) * (per_page as u16)) as i64;
    let limit = per_page as i64;
    let books = stores::minors::get_books(Some(offset), Some(limit), &db)
        .await
        .map_err(ApiError::EdgeDBQueryError)?;
    let count = stores::minors::get_all_books_count(&db)
        .await
        .map_err(ApiError::EdgeDBQueryError)?;
    let total_pages =
        NonZeroU16::new((count as f64 / per_page as f64).ceil() as u16).unwrap_or(NonZeroU16::MIN);
    let links = gen_pagination_links(&paging.into(), count as usize, original_uri);
    let resp = ObjectListResponse {
        objects: books,
        count: count as usize,
        total_pages,
        links,
    };
    Ok(Json(resp))
}

pub async fn get_book(
    WithRejection(Path(id), _): WithRejection<Path<Uuid>, ApiError>,
    State(db): State<EdgeClient>,
) -> AxumResult<Json<Book>> {
    let book = stores::minors::get_book(id, &db)
        .await
        .map_err(ApiError::EdgeDBQueryError)?
        .ok_or(ApiError::ObjectNotFound("Book".into()))?;
    Ok(Json(book))
}

pub async fn delete_book(
    auth: Auth,
    WithRejection(Path(id), _): WithRejection<Path<Uuid>, ApiError>,
    State(db): State<EdgeClient>,
) -> AxumResult<StatusCode> {
    auth.current_user.ok_or(ApiError::Unauthorized)?;
    let q = "DELETE Book FILTER .id = <uuid>$0";
    let _p: MinimalObject = db
        .query_single(q, &(id,))
        .await
        .map_err(ApiError::EdgeDBQueryError)?
        .ok_or(ApiError::ObjectNotFound("Book".into()))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_book_partial(
    WithRejection(Path(id), _): WithRejection<Path<Uuid>, ApiError>,
    auth: Auth,
    State(db): State<EdgeClient>,
    WithRejection(Json(value), _): WithRejection<Json<Value>, ApiError>,
) -> AxumResult<Json<Book>> {
    auth.current_user.ok_or(ApiError::Unauthorized)?;
    // Collect list of submitted fields
    let jdata: JMap<String, Value> =
        serde_json::from_value(value.clone()).map_err(ApiError::JsonExtractionError)?;
    // User submit no field to update
    if jdata.is_empty() {
        let obj = stores::minors::get_book(id, &db)
            .await
            .map_err(ApiError::EdgeDBQueryError)?
            .ok_or(ApiError::ObjectNotFound("Book".into()))?;
        return Ok(Json(obj));
    }
    let patch_data: BookPatchData =
        serde_json::from_value(value).map_err(ApiError::JsonExtractionError)?;
    let patch_data = BookPatchData::validify(patch_data.into()).map_err(ApiError::ValidationErrors)?;
    let submitted_fields: Vec<&String> = jdata.keys().collect();
    let set_clause = patch_data.gen_set_clause(&submitted_fields);
    let args = patch_data.make_edgedb_object(id, &submitted_fields);
    let q = format!(
        "SELECT (
            UPDATE Book FILTER .id = <uuid>$id SET {{ {set_clause} }}
        ) {{
            id,
            title,
            download_url,
            author: {{ id, name }},
        }}"
    );
    tracing::debug!("Query: {}", q);
    tracing::debug!("Args: {:#?}", args);
    let book = db.query_single(&q, &args).await.map_err(ApiError::EdgeDBQueryError)?.ok_or(ApiError::ObjectNotFound("Book".into()))?;
    Ok(Json(book))
}

pub async fn create_book(
    auth: Auth,
    State(db): State<EdgeClient>,
    WithRejection(Json(value), _): WithRejection<Json<Value>, ApiError>,
) -> AxumResult<Json<Book>> {
    auth.current_user.ok_or(ApiError::Unauthorized)?;
    // Collect list of submitted fields
    let jdata: JMap<String, Value> =
        serde_json::from_value(value.clone()).map_err(ApiError::JsonExtractionError)?;
    // User submit no field to create Book
    (!jdata.is_empty())
        .then_some(())
        .ok_or(ApiError::NotEnoughData)?;
    let post_data: BookCreateData =
        serde_json::from_value(value).map_err(ApiError::JsonExtractionError)?;
    let post_data = BookCreateData::validify(post_data.into()).map_err(ApiError::ValidationErrors)?;
    let set_clause = post_data.gen_set_clause();
    let args = post_data.make_edgedb_object();
    let q = format!(
        "
    SELECT (
        INSERT Book {{
            {set_clause}
        }}
    ) {{
        id,
        title,
        download_url,
        author: {{ id, name }},
    }}"
    );
    let book = db.query_single(&q, &args).await.map_err(ApiError::EdgeDBQueryError)?.ok_or(ApiError::Other("Failed to create Book".into()))?;
    Ok(Json(book))
}
