use axum::{routing::get, Router};

use crate::types::SharedState;
use super::base;


pub fn get_router() -> Router<SharedState> {
    Router::new().route("/", get(base::home))
}
