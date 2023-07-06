
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Result as AxumResult};
use http::Uri;
use minijinja::context;
use minijinja::value::Value as MJValue;

use crate::consts::STATIC_URL;
use crate::types::AppState;
use crate::stores::blog::{get_blogposts, get_blogpost_by_slug};
use crate::errors::PageError;
use crate::types::StaticFile;
use super::render_with;

pub async fn home(State(state): State<AppState>) -> AxumResult<Html<String>> {
    let AppState { db, jinja } = state;
    let result = get_blogposts(Some(0), Some(10), &db).await.map_err(PageError::EdgeDBQueryError)?;
    let posts: Vec<MJValue> = result.into_iter().collect();
    let context = context!(posts => posts);
    let content = render_with("home.jinja", context, jinja)?;
    Ok(Html(content))
}

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    // URI is like "/static/css/style.css", we need to strip to "css/style.css"
    let path = uri.path().trim_start_matches(&format!("{STATIC_URL}/")).to_string();
    StaticFile(path)
}

pub async fn show_post(Path((_y, _m, slug)): Path<(u16, u16, String)>, State(state): State<AppState>) -> AxumResult<Html<String>> {
    let AppState { db, jinja } = state;
    let post = get_blogpost_by_slug(slug, &db).await.map_err(PageError::EdgeDBQueryError)?.ok_or(StatusCode::NOT_FOUND)?;
    let context = context!(post => post);
    let content = render_with("blog/post.jinja", context, jinja)?;
    Ok(Html(content))
}
