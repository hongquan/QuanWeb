use axum::extract::State;
use axum::response::IntoResponse;
use axum_template::RenderHtml;

use crate::types::JinjaEngine;

pub async fn home(State(engine): State<JinjaEngine>) -> impl IntoResponse {
    RenderHtml("home.jinja", engine, &())
}
