use axum::routing::{Router, get, post, delete, patch};

use crate::types::SharedState;
use super::views;
use super::auth;

pub fn get_router() -> Router<SharedState> {
    Router::new()
        .route("/", get(views::root))
        .route("/login", post(auth::login))
        .route("/users/me", get(views::show_me))
        .route("/posts/", get(views::list_posts))
        .route("/posts/:post_id", get(views::get_post))
        .route("/posts/:post_id", delete(views::delete_post))
        .route("/posts/:post_id", patch(views::update_post_partial))
        .route("/posts/", post(views::create_post))
        .route("/categories/", get(views::list_categories))
        .route("/categories/:category_id", get(views::get_category))
        .route("/categories/:category_id", delete(views::delete_category))
        .route("/categories/:category_id", patch(views::update_category_partial))
        .route("/categories/", post(views::create_category))
}
