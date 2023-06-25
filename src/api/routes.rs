use axum::routing::{get, post};
use axum_named_routes::NamedRouter;

use super::views;
use super::auth;

pub fn get_router() -> NamedRouter {
    NamedRouter::new()
        .route("index", "/", get(views::root))
        .route("login", "/login", post(auth::login))
        .route("me", "/users/me", get(views::show_me))
        .route("post-list", "/posts", get(views::list_posts))
}
