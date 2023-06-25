use axum::routing::{get, post};
use axum_named_routes::NamedRouter;

use super::views;
use super::auth;

pub fn get_router() -> NamedRouter {
    NamedRouter::new()
        .route("index", "/", get(views::root))
        .route("login", "/login", post(auth::login))
        .route("login-short", "/login-short", post(auth::login_short))
        .route("post-list", "/posts", get(views::list_posts))
}
