use axum::extract::State;
use axum::response::{Html, Result as AxumResult};
use axum_sessions::extractors::ReadableSession;
use minijinja::context;

use crate::auth::Auth;
use crate::consts::{KEY_LANG, DEFAULT_LANG};
use crate::types::AppState;
use crate::stores::{minors::get_all_talks, blog::get_blog_categories};
use crate::errors::PageError;

use super::render_with;

pub async fn list_talks(auth: Auth, session: ReadableSession, State(state): State<AppState>) -> AxumResult<Html<String>> {
    let AppState { db, jinja } = state;
    let presentations = get_all_talks(&db).await.map_err(PageError::EdgeDBQueryError)?;
    let lang = session.get::<String>(KEY_LANG).unwrap_or(DEFAULT_LANG.into());
    let no_tracking = auth.current_user.is_some();
    let categories = get_blog_categories(None, None, &db)
        .await
        .map_err(PageError::EdgeDBQueryError)?;
    let ctx = context!(
        presentations,
        lang,
        categories,
        no_tracking,
    );
    let content = render_with("minors/talk_list.jinja", ctx, jinja)?;
    Ok(Html(content))
}
