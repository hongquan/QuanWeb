pub mod blog;
pub mod feeds;
pub mod minors;
pub mod old_urls;

use std::num::NonZeroU16;

use axum::extract::Form;
use axum::extract::{OriginalUri, Query, State};
use axum::response::{Html, IntoResponse, Result as AxumResult};
use http::{HeaderName, StatusCode, Uri};
use minijinja::context;
use tower_sessions::Session;
use unic_langid::LanguageIdentifier;

use super::structs::{LaxPaging, SetLangReq};
use crate::auth::AuthSession;
use crate::consts::{DEFAULT_LANG, DEFAULT_PAGE_SIZE, KEY_LANG, STATIC_URL};
pub use crate::errors::PageError;
use crate::stores;
use crate::types::{AppState, Paginator, StaticFile};
use crate::utils::html::render_with;

pub async fn fallback_view() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not found")
}

pub async fn home(
    auth_session: AuthSession,
    OriginalUri(current_url): OriginalUri,
    Query(paging): Query<LaxPaging>,
    session: Session,
    State(state): State<AppState>,
) -> AxumResult<Html<String>> {
    let AppState { db, jinja } = state;
    let current_page = paging.get_page_as_number();
    let total = stores::blog::count_all_published_posts(&db)
        .await
        .map_err(PageError::GelQueryError)?;
    let page_size = DEFAULT_PAGE_SIZE;
    let total_pages = NonZeroU16::try_from((total as f64 / page_size as f64).ceil() as u16)
        .unwrap_or(NonZeroU16::MIN);
    let paginator = Paginator {
        current_page,
        total_pages,
    };
    let pagelink_items = paginator.generate_items();
    tracing::debug!("Pagination links: {:?}", pagelink_items);
    let next_page_url = paginator.next_url(&current_url);
    let prev_page_url = paginator.previous_url(&current_url);
    let offset = ((current_page.get() - 1) * (page_size as u16)) as i64;
    let posts = stores::blog::get_published_posts(Some(offset), Some(page_size as i64), &db)
        .await
        .map_err(PageError::GelQueryError)?;
    let categories = stores::blog::get_blog_categories(None, None, &db)
        .await
        .map_err(PageError::GelQueryError)?;
    let no_tracking = auth_session.user.is_some();
    let lang = session
        .get::<String>(KEY_LANG)
        .await
        .ok()
        .flatten()
        .unwrap_or(DEFAULT_LANG.into());
    let context = context!(
        lang => lang,
        posts => posts,
        categories => categories,
        pagelink_items => pagelink_items,
        next_page_url => next_page_url,
        prev_page_url => prev_page_url,
        no_tracking => no_tracking);
    let content = render_with("home.jinja", context, jinja)?;
    Ok(Html(content))
}

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    // URI is like "/static/css/style.css", we need to strip to "css/style.css"
    let path = uri
        .path()
        .trim_start_matches(&format!("{STATIC_URL}/"))
        .to_string();
    StaticFile(path)
}

type HTMXResponse = ([(HeaderName, &'static str); 1], Html<String>);

pub async fn set_lang(
    session: Session,
    Form(payload): Form<SetLangReq>,
) -> AxumResult<HTMXResponse> {
    let li: LanguageIdentifier = payload
        .lang
        .parse()
        .map_err(|_e| StatusCode::UNPROCESSABLE_ENTITY)?;
    session
        .insert(KEY_LANG, li.clone())
        .await
        .map_err(|_e| StatusCode::SERVICE_UNAVAILABLE)?;
    let lang_code = li.to_string();
    let header_name = HeaderName::from_static("hx-refresh");
    let r = ([(header_name, "true")], Html(lang_code));
    Ok(r)
}
