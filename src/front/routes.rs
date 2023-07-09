use axum::{routing::get, Router};

use crate::types::AppState;
use crate::consts::STATIC_URL;
use super::views;

pub fn get_router() -> Router<AppState> {
    Router::new()
    .route("/", get(views::home))
    .route(&format!("{STATIC_URL}/*file"), get(views::static_handler))
    .route("/post/:year/:month/:slug", get(views::blog::show_post))
    .route("/category/_uncategorized/", get(views::blog::list_uncategorized_posts))
    .route("/category/:category/", get(views::blog::list_posts))
    .route("/preview/:id", get(views::blog::preview_post))
    .route("/blog/:year/:month/:id_and_slug", get(views::old_urls::redirect_old_post_view))
}
