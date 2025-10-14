use axum::extract::State;
use axum::response::{Html, Result as AxumResult};
use minijinja::context;
use tower_sessions::Session;

use crate::auth::AuthSession;
use crate::consts::{DEFAULT_LANG, KEY_LANG};
use crate::errors::PageError;
use crate::stores::{
    blog::get_blog_categories,
    minors::{get_all_books, get_all_talks},
};
use crate::types::AppState;
use crate::utils::html::render_with;

pub async fn list_talks(
    auth_session: AuthSession,
    session: Session,
    State(state): State<AppState>,
) -> AxumResult<Html<String>> {
    let AppState { db, jinja } = state;
    let presentations = get_all_talks(&db).await.map_err(PageError::GelQueryError)?;
    let lang = session
        .get::<String>(KEY_LANG)
        .await
        .ok()
        .flatten()
        .unwrap_or(DEFAULT_LANG.into());
    let no_tracking = auth_session.user.is_some();
    let categories = get_blog_categories(None, None, &db)
        .await
        .map_err(PageError::GelQueryError)?;
    let ctx = context!(presentations, lang, categories, no_tracking,);
    let content = render_with("minors/talk_list.jinja", ctx, jinja)?;
    Ok(Html(content))
}

pub async fn list_books(
    auth_session: AuthSession,
    session: Session,
    State(state): State<AppState>,
) -> AxumResult<Html<String>> {
    let AppState { db, jinja } = state;
    let books = get_all_books(&db).await.map_err(PageError::GelQueryError)?;
    let lang = session
        .get::<String>(KEY_LANG)
        .await
        .ok()
        .flatten()
        .unwrap_or(DEFAULT_LANG.into());
    let no_tracking = auth_session.user.is_some();
    let categories = get_blog_categories(None, None, &db)
        .await
        .map_err(PageError::GelQueryError)?;
    let ctx = context!(books, lang, categories, no_tracking,);
    let content = render_with("minors/book_list.jinja", ctx, jinja)?;
    Ok(Html(content))
}
