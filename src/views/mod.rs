pub mod api;
pub mod base;

use axum::{routing::get, Router};
pub use base::*;

pub fn get_api_router() -> Router {
    Router::new()
        .route("/", get(api::root))
        .route("/posts", get(api::list_posts))
}
