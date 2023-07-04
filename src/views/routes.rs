use axum::{routing::get, Router};

use crate::types::AppState;
use super::base;


pub fn get_router() -> Router<AppState> {
    Router::new().route("/", get(base::home))
}
