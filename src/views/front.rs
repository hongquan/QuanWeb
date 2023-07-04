use axum::extract::State;
use axum::response::{Html, Result as AxumResult};
use axum_template::TemplateEngine;
use minijinja::context;

use crate::types::AppState;
use crate::retrievers;
use crate::errors::PageError;
use crate::models::BlogPost;

pub async fn home(State(state): State<AppState>) -> AxumResult<Html<String>> {
    let AppState { db, template_engine } = state;
    let result = retrievers::get_blogposts(Some(0), Some(10), &db).await.map_err(PageError::EdgeDBQueryError)?;
    let posts: Vec<BlogPost> = result.into_iter().collect();
    let context = context!(posts => posts);
    let html = template_engine.render("home.jinja", context)?;
    Ok(Html(html))
}
