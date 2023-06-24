pub mod api;
pub mod base;
pub mod structs;

use axum::routing::get;
use axum_named_routes::NamedRouter;
pub use base::*;

pub fn get_api_router() -> NamedRouter {
    NamedRouter::new()
        .route("index", "/", get(api::root))
        .route("post-list", "/posts", get(api::list_posts))
}
