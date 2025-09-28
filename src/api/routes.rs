use axum::routing::{Router, get, post};

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

    let single_presentation_router = get(views::get_presentation)
        .patch(views::update_presentation_partial)
        .delete(views::delete_presentation);

    let single_book_author_router = get(views::get_book_author)
        .patch(views::update_book_author_partial)
        .delete(views::delete_book_author);

    let single_book_router = get(views::get_book)
        .delete(views::delete_book)
        .patch(views::update_book_partial);

    Router::new()
        .route("/", get(views::root))
        .route("/login", post(auth::login))
        .route("/logout", post(auth::logout))
        .route("/users/me", get(views::show_me))
        .route("/posts/", get(views::list_posts).post(views::create_post))
        .route("/posts/{post_id}", single_post_router)
        .route(
            "/categories/",
            get(views::list_categories).post(views::create_category),
        )
        .route("/categories/{category_id}", single_category_router)
        .route("/users/", get(views::list_users))
        .route(
            "/presentations/",
            get(views::list_presentations).post(views::create_presentation),
        )
        .route("/presentations/{id}", single_presentation_router)
        .route(
            "/book-authors/",
            get(views::list_book_authors).post(views::create_book_author),
        )
        .route("/book-authors/{id}", single_book_author_router)
        .route("/books/", get(views::list_books).post(views::create_book))
        .route("/books/{id}", single_book_router)
        .route("/markdown-to-html/", post(views::convert_to_html))
        .route("/generate-slug", post(views::generate_slug))
}
