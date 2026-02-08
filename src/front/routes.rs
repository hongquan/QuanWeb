use axum::{
    Router,
    routing::{get, post},
};

use super::views;
use crate::consts::STATIC_URL;
use crate::types::AppState;

pub fn get_router() -> Router<AppState> {
    Router::new()
        .route("/", get(views::home))
        .route("/posts/", get(views::list_recent_posts))
        .route(
            &format!("{STATIC_URL}/{{*file}}"),
            get(views::static_handler),
        )
        .route("/post/{year}/{month}/{slug}", get(views::blog::show_post))
        .route(
            "/category/_uncategorized/",
            get(views::blog::list_uncategorized_posts),
        )
        .route("/category/{category}/", get(views::blog::list_posts))
        .route("/preview/{id}", get(views::blog::preview_post))
        .route(
            "/blog/{*rest}",
            get(views::old_urls::redirect_old_blog_view),
        )
        .route("/talk/", get(views::minors::list_talks))
        .route("/book/", get(views::minors::list_books))
        .route("/feeds.atom", get(views::feeds::gen_atom_feeds))
        .route("/feeds.json", get(views::feeds::gen_json_feeds))
        .route("/sitemap.xml", get(views::feeds::gen_sitemaps))
        .route("/api/set-lang", post(views::set_lang))
}
