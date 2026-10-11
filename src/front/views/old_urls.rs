use axum::extract::{Path, State};
use axum::response::{Redirect, Result};
use chrono::{DateTime, Utc};
use gel_tokio::Client;
use http::StatusCode;
use lazy_regex::{regex_captures, regex_is_match};

use crate::errors::PageError;
use crate::models::blogs::MiniBlogPost;
use crate::stores;

pub async fn redirect_old_blog_view(
    Path(rest): Path<String>,
    State(db): State<Client>,
) -> Result<Redirect> {
    let is_cat = regex_is_match!(r"^[-\w]+/$", &rest);
    if is_cat {
        return Ok(Redirect::temporary(&format!("/category/{}", rest)));
    }
    let Some((_, old_id, _slug)) = regex_captures!(r"^\d+/\d+/(\d+)-([-\w]+)$", &rest) else {
        return Err((StatusCode::NOT_FOUND, "URL should be \"id-slug\"").into());
    };
    let old_id = old_id
        .parse()
        .map_err(|_e| (StatusCode::NOT_FOUND, "ID must be number"))?;
    tracing::debug!("Look for post with old ID {}", old_id);
    let post: MiniBlogPost = stores::blog::get_mini_post_by_old_id(old_id, &db)
        .await
        .map_err(PageError::GelQueryError)?
        .ok_or((StatusCode::NOT_FOUND, "No post with this ID"))?;
    let created_at: DateTime<Utc> = post.created_at.into();
    let new_url = format!("/post/{}/{}", created_at.format("%Y/%m"), post.slug);
    Ok(Redirect::temporary(&new_url))
}
