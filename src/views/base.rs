use axum::response::IntoResponse;
use axum::extract::State;
use axum_template::RenderHtml;

use crate::types::SharedState;

pub async fn home(State(state): State<SharedState>) -> impl IntoResponse {
    let engine = state.template_engine.clone();
    RenderHtml("home.jinja", engine, &())
}
