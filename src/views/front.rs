use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Result as AxumResult};
use axum_extra::extract::Query;
use http::Uri;
use minijinja::context;
use minijinja::value::Value as MJValue;

use super::render_with;
use super::structs::PostPageParams;
use crate::auth::Auth;
use crate::consts::STATIC_URL;
use crate::errors::PageError;
use crate::stores::blog::{get_blogpost_by_slug, get_blogposts, get_next_post, get_previous_post};
use crate::types::AppState;
use crate::types::StaticFile;

pub async fn home(State(state): State<AppState>) -> AxumResult<Html<String>> {
    let AppState { db, jinja } = state;
    let result = get_blogposts(Some(0), Some(10), &db)
        .await
        .map_err(PageError::EdgeDBQueryError)?;
    let posts: Vec<MJValue> = result.into_iter().collect();
    let context = context!(posts => posts);
    let content = render_with("home.jinja", context, jinja)?;
    Ok(Html(content))
}

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    // URI is like "/static/css/style.css", we need to strip to "css/style.css"
    let path = uri
        .path()
        .trim_start_matches(&format!("{STATIC_URL}/"))
        .to_string();
    StaticFile(path)
}

pub async fn show_post(
    auth: Auth,
    Path((_y, _m, slug)): Path<(u16, u16, String)>,
    _params: Query<PostPageParams>,
    State(state): State<AppState>,
) -> AxumResult<Html<String>> {
    let AppState { db, jinja } = state;
    let post = get_blogpost_by_slug(slug, &db)
        .await
        .map_err(PageError::EdgeDBQueryError)?
        .ok_or((StatusCode::NOT_FOUND, "No post at this URL"))?;
    let user = auth.current_user;
    let no_tracking = !post.is_published.unwrap_or(false) || user.is_some();
    let prev_post = get_previous_post(post.created_at, None, &db)
        .await
        .map_err(PageError::EdgeDBQueryError)?;
    tracing::debug!("Previous post: {:?}", prev_post);
    let next_post = get_next_post(post.created_at, None, &db)
        .await
        .map_err(PageError::EdgeDBQueryError)?;
    tracing::debug!("Next post: {:?}", next_post);
    let context = context!(post => post, prev_post => prev_post, next_post => next_post, no_tracking => no_tracking);
    let content = render_with("blog/post.jinja", context, jinja)?;
    Ok(Html(content))
}
