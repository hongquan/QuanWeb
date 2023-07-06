use std::collections::HashMap;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Result as AxumResult};
use axum_template::TemplateEngine;
use http::Uri;
use minijinja::context;
use time::macros::format_description;
use uuid::Uuid;
use minijinja::value::Value as MJValue;

use crate::consts::STATIC_URL;
use crate::types::AppState;
use crate::models::blogs::JjBlogPost;
use crate::stores::blog::{get_blogposts, get_blogpost_by_slug};
use crate::errors::PageError;
use crate::types::StaticFile;

pub async fn home(State(state): State<AppState>) -> AxumResult<Html<String>> {
    let AppState { db, template_engine } = state;
    let result = get_blogposts(Some(0), Some(10), &db).await.map_err(PageError::EdgeDBQueryError)?;
    let posts: Vec<JjBlogPost> = result.into_iter().collect();
    let jj_posts: Vec<MJValue> = posts.clone().into_iter().map(MJValue::from).collect();
    let urls: HashMap<Uuid, String> = posts.iter().map(|p| (p.id, get_post_detail_url(p))).collect();
    let context = context!(posts => jj_posts, urls => urls);
    let html = template_engine.render("home.jinja", context)?;
    Ok(Html(html))
}

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    // URI is like "/static/css/style.css", we need to strip to "css/style.css"
    let path = uri.path().trim_start_matches(&format!("{STATIC_URL}/")).to_string();
    StaticFile(path)
}

pub async fn show_post(Path((_y, _m, slug)): Path<(u16, u16, String)>, State(state): State<AppState>) -> AxumResult<Html<String>> {
    let AppState { db, template_engine } = state;
    let post = get_blogpost_by_slug(slug, &db).await.map_err(PageError::EdgeDBQueryError)?.ok_or(StatusCode::NOT_FOUND)?;
    let context = context!(post => post);
    let html = template_engine.render("blog/post.jinja", context)?;
    Ok(Html(html))
}

pub fn get_post_detail_url(post: &JjBlogPost) -> String {
    let y = format_description!("[year]");
    let m = format_description!("[month]");
    format!("/post/{}/{}/{}", post.created_at.format(y).unwrap_or("1990".into()), post.created_at.format(m).unwrap_or("1".into()), post.slug)
}
