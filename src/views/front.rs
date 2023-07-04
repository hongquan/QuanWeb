use axum::extract::State;
use axum::response::{Html, Result as AxumResult};
use axum_template::TemplateEngine;

use crate::types::AppState;
use crate::retrievers;
use crate::errors::PageError;

pub async fn home(State(state): State<AppState>) -> AxumResult<Html<String>> {
    let AppState { db, template_engine } = state;
    let result = retrievers::get_blogposts(Some(0), Some(10), &db).await.map_err(PageError::EdgeDBQueryError)?;
    let html = template_engine.render("home.jinja", &result)?;
    Ok(Html(html))
}
