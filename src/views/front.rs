use axum::extract::State;
use axum::response::{Html, IntoResponse, Result as AxumResult};
use axum_template::TemplateEngine;
use http::Uri;
use minijinja::context;

use crate::consts::STATIC_URL;
use crate::types::AppState;
use crate::models::blogs::JjBlogPost;
use crate::stores::blog::get_blogposts;
use crate::errors::PageError;
use crate::types::StaticFile;

pub async fn home(State(state): State<AppState>) -> AxumResult<Html<String>> {
    let AppState { db, template_engine } = state;
    let result = get_blogposts(Some(0), Some(10), &db).await.map_err(PageError::EdgeDBQueryError)?;
    let posts: Vec<JjBlogPost> = result.into_iter().collect();
    let context = context!(posts => posts);
    let html = template_engine.render("home.jinja", context)?;
    Ok(Html(html))
}

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    // URI is like "/static/css/style.css", we need to strip to "css/style.css"
    let path = uri.path().trim_start_matches(&format!("{STATIC_URL}/")).to_string();
    StaticFile(path)
}
