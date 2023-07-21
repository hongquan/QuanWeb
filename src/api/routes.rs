use axum::routing::{get, post, Router};

use super::auth;
use super::views;
use crate::types::AppState;

pub fn get_router() -> Router<AppState> {
    let single_post_router = get(views::get_post)
        .patch(views::update_post_partial)
        .delete(views::delete_post);
    let single_category_router = get(views::get_category)
        .patch(views::update_category_partial)
        .delete(views::delete_category);

    let single_presentation_router = get(views::get_presentation);

    Router::new()
        .route("/", get(views::root))
        .route("/login", post(auth::login))
        .route("/logout", post(auth::logout))
        .route("/users/me", get(views::show_me))
        .route("/posts/", get(views::list_posts).post(views::create_post))
        .route("/posts/:post_id", single_post_router)
        .route(
            "/categories/",
            get(views::list_categories).post(views::create_category),
        )
        .route("/categories/:category_id", single_category_router)
        .route("/presentations/", get(views::list_presentations))
        .route("/presentations/:id", single_presentation_router)
        .route("/markdown-to-html/", post(views::convert_to_html))
}
