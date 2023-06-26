use axum::routing::{get, post};
use axum_named_routes::NamedRouter;

use crate::types::SharedState;
use super::views;
use super::auth;

pub fn get_router(state: SharedState) -> NamedRouter {
    NamedRouter::new()
        .route("index", "/", get(views::root))
        .route("login", "/login", post(auth::login))
        .route("me", "/users/me", get(views::show_me))
        .route("post-list", "/posts/", get(views::list_posts))
        .with_state(state)
}
