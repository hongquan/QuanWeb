use axum::{routing::get, Router};

use crate::types::AppState;
use super::home;


pub fn get_router() -> Router<AppState> {
    Router::new().route("/", get(home))
}
