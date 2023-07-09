use axum::extract::{Path, State};
use axum::response::{Redirect, Result};
use chrono::{DateTime, Utc};
use http::StatusCode;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::errors::PageError;
use crate::models::blogs::MiniBlogPost;
use crate::stores;
use crate::types::AppState;

pub async fn redirect_old_post_view(
    Path((_y, _m, id_and_slug)): Path<(u16, u16, String)>,
    State(state): State<AppState>,
) -> Result<Redirect> {
    static REGEX_OLD_POST_URL: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\d+)-([-\w]+)$").unwrap());
    let capt = REGEX_OLD_POST_URL
        .captures(&id_and_slug)
        .ok_or((StatusCode::NOT_FOUND, "URL should be \"id-slug\""))?;
    let (_, [old_id, _slug]) = capt.extract();
    let AppState { db, jinja: _jinja } = state;
    let old_id = old_id
        .parse()
        .map_err(|_e| (StatusCode::NOT_FOUND, "ID must be number"))?;
    tracing::debug!("Look for post with old ID {}", old_id);
    let post: MiniBlogPost = stores::blog::get_mini_post_by_old_id(old_id, &db)
        .await
        .map_err(PageError::EdgeDBQueryError)?
        .ok_or((StatusCode::NOT_FOUND, "No post with this ID"))?;
    let created_at: DateTime<Utc> = post.created_at.into();
    let new_url = format!("/post/{}/{}", created_at.format("%Y/%m"), post.slug);
    Ok(Redirect::temporary(&new_url))
}

pub async fn redirect_old_category_view(Path(cat_slug): Path<String>) -> Result<Redirect> {
    static RE_OLD_CATEGORY_URL: Lazy<Regex> = Lazy::new(|| Regex::new(r"[-\w]+").unwrap());
    let matched = RE_OLD_CATEGORY_URL.is_match(&cat_slug);
    if !matched {
        return Err((StatusCode::NOT_FOUND, "Must be category slug".to_string()).into());
    }
    Ok(Redirect::temporary(&format!("/category/{}/", cat_slug)))
}
