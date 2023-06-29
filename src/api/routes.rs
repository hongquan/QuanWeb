use axum::routing::{get, post, delete, patch};
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
        .route("post-retrieve", "/posts/:post_id", get(views::get_post))
        .route("post-delete", "/posts/:post_id", delete(views::delete_post))
        .route("post-update", "/posts/:post_id", patch(views::update_post_partial))
        .route("post-create", "/posts/", post(views::create_post))
        .route("category-list", "/categories/", get(views::list_categories))
        .route("category-retrieve", "/categories/:category_id", get(views::get_category))
        .route("category-delete", "/categories/:category_id", delete(views::delete_category))
        .route("category-update", "/categories/:category_id", patch(views::update_category_partial))
        .with_state(state)
}
