use axum::extract::{Path, State};
use axum::response::{Redirect, Result};
use chrono::{DateTime, Utc};
use edgedb_tokio::Client;
use http::StatusCode;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::errors::PageError;
use crate::models::blogs::MiniBlogPost;
use crate::stores;

pub async fn redirect_old_blog_view(Path(rest): Path<String>, State(db): State<Client>) -> Result<Redirect> {
    static RE_OLD_CAT_URL: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[-\w]+/$").unwrap());
    static RE_OLD_POST_URL: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d+/\d+/(\d+)-([-\w]+)$").unwrap());
    let is_cat = RE_OLD_CAT_URL.is_match(&rest);
    if is_cat {
        return Ok(Redirect::temporary(&format!("/category/{}", rest)));
    }
    let capt = RE_OLD_POST_URL
        .captures(&rest)
        .ok_or((StatusCode::NOT_FOUND, "URL should be \"id-slug\""))?;
    let (_, [old_id, _slug]) = capt.extract();
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
