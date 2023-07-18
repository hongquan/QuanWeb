use std::num::NonZeroU16;

use axum::extract::{OriginalUri, Path, State};
use axum::http::StatusCode;
use axum::response::{Html, Result as AxumResult};
use axum_extra::extract::Query;
use axum_sessions::extractors::ReadableSession;
use indexmap::indexmap;
use minijinja::{context, value::Value as MJValue};
use uuid::Uuid;

use super::super::structs::{LaxPaging, PostPageParams};
use super::render_with;
use crate::auth::Auth;
use crate::consts::{DEFAULT_PAGE_SIZE, KEY_LANG, DEFAULT_LANG};
use crate::errors::PageError;
use crate::stores;
use crate::stores::blog::{get_detailed_post_by_slug, get_next_post, get_previous_post};
use crate::types::{AppState, Paginator};

pub async fn show_post(
    auth: Auth,
    Path((_y, _m, slug)): Path<(u16, u16, String)>,
    Query(params): Query<PostPageParams>,
    session: ReadableSession,
    State(state): State<AppState>,
) -> AxumResult<Html<String>> {
    let AppState { db, jinja } = state;
    let post = get_detailed_post_by_slug(slug, &db)
        .await
        .map_err(PageError::EdgeDBQueryError)?
        .ok_or((StatusCode::NOT_FOUND, "No post at this URL"))?;
    let user = auth.current_user;
    let no_tracking = !post.is_published.unwrap_or(false) || user.is_some();
    let cat = match params.cat {
        Some(slug) => stores::blog::get_category_by_slug(&slug, &db).await.map_err(PageError::EdgeDBQueryError)?,
        None => None,
    };
    let cat_slug = cat.as_ref().map(|c| c.slug.as_str());

    let prev_post = get_previous_post(post.created_at, cat_slug, &db)
        .await
        .map_err(PageError::EdgeDBQueryError)?;
    let next_post = get_next_post(post.created_at, cat_slug, &db)
        .await
        .map_err(PageError::EdgeDBQueryError)?;
    let categories = stores::blog::get_blog_categories(None, None, &db)
        .await
        .map_err(PageError::EdgeDBQueryError)?;
    let lang = session.get::<String>(KEY_LANG).unwrap_or(DEFAULT_LANG.into());
    let mut vcontext = indexmap! {
        "post" => MJValue::from_serializable(&post),
        "prev_post" => MJValue::from_serializable(&prev_post),
        "next_post" => MJValue::from_serializable(&next_post),
        "categories" => MJValue::from_serializable(&categories),
        "lang" => MJValue::from(lang),
        "no_tracking" => MJValue::from(no_tracking),
    };
    if let Some(cat) = cat {
        vcontext.insert("cat", MJValue::from_serializable(&cat));
    }
    let content = render_with("blog/post.jinja", vcontext, jinja)?;
    Ok(Html(content))
}

pub async fn list_posts(
    auth: Auth,
    Path(cat_slug): Path<String>,
    OriginalUri(current_url): OriginalUri,
    Query(paging): Query<LaxPaging>,
    session: ReadableSession,
    State(state): State<AppState>,
) -> AxumResult<Html<String>> {
    let AppState { db, jinja } = state;
    let current_page = paging
        .page
        .and_then(|p| NonZeroU16::new(p.parse().ok()?))
        .unwrap_or(NonZeroU16::MIN);
    let cat = stores::blog::get_category_by_slug(&cat_slug, &db)
        .await
        .map_err(PageError::EdgeDBQueryError)?
        .ok_or((StatusCode::NOT_FOUND, "No post at this URL"))?;
    let posts = stores::blog::get_published_posts_under_category(Some(cat_slug), None, None, &db)
        .await
        .map_err(PageError::EdgeDBQueryError)?;
    tracing::debug!("To count posts under category {}", cat.id);
    let total = stores::blog::count_blogposts_under_category(cat.id, &db)
        .await
        .map_err(PageError::EdgeDBQueryError)?;
    let page_size = DEFAULT_PAGE_SIZE;
    let total_pages = NonZeroU16::try_from((total as f64 / page_size as f64).ceil() as u16)
        .unwrap_or(NonZeroU16::MIN);
    let paginator = Paginator {
        current_page,
        total_pages,
    };
    let pagelink_items = paginator.generate_items();
    let next_page_url = paginator.next_url(&current_url);
    let prev_page_url = paginator.previous_url(&current_url);
    let categories = stores::blog::get_blog_categories(None, None, &db)
        .await
        .map_err(PageError::EdgeDBQueryError)?;
    let lang = session.get::<String>(KEY_LANG).unwrap_or(DEFAULT_LANG.into());
    let no_tracking = auth.current_user.is_some();
    let context = context!(
        posts => posts,
        cat => cat,
        pagelink_items => pagelink_items,
        next_page_url => next_page_url,
        prev_page_url => prev_page_url,
        categories => categories,
        lang => lang,
        no_tracking => no_tracking);
    let content = render_with("blog/post_list.jinja", context, jinja)?;
    Ok(Html(content))
}

pub async fn preview_post(
    auth: Auth,
    Path(id): Path<Uuid>,
    session: ReadableSession,
    State(state): State<AppState>,
) -> AxumResult<Html<String>> {
    let _user = auth.current_user.ok_or(PageError::PermissionDenied);
    let AppState { db, jinja } = state;
    let post = stores::blog::get_post(id, &db)
        .await
        .map_err(PageError::EdgeDBQueryError)?
        .ok_or((StatusCode::NOT_FOUND, "No post at this URL"))?;
    let prev_post = get_previous_post(post.created_at, None, &db)
        .await
        .map_err(PageError::EdgeDBQueryError)?;
    tracing::debug!("Previous post: {:?}", prev_post);
    let next_post = get_next_post(post.created_at, None, &db)
        .await
        .map_err(PageError::EdgeDBQueryError)?;
    tracing::debug!("Next post: {:?}", next_post);
    let lang = session.get::<String>(KEY_LANG).unwrap_or(DEFAULT_LANG.into());
    let context =
        context!(post => post, prev_post => prev_post, next_post => next_post, lang => lang, no_tracking => true);
    let content = render_with("blog/post.jinja", context, jinja)?;
    Ok(Html(content))
}

pub async fn list_uncategorized_posts(
    auth: Auth,
    OriginalUri(current_url): OriginalUri,
    Query(paging): Query<LaxPaging>,
    session: ReadableSession,
    State(state): State<AppState>,
) -> AxumResult<Html<String>> {
    let AppState { db, jinja } = state;
    let current_page = paging
        .page
        .and_then(|p| NonZeroU16::new(p.parse().ok()?))
        .unwrap_or(NonZeroU16::MIN);
    let total = stores::blog::count_published_uncategorized_posts(&db)
        .await
        .map_err(PageError::EdgeDBQueryError)?;
    let page_size = DEFAULT_PAGE_SIZE;
    let total_pages = NonZeroU16::try_from((total as f64 / page_size as f64).ceil() as u16)
        .unwrap_or(NonZeroU16::MIN);
    let paginator = Paginator {
        current_page,
        total_pages,
    };
    let pagelink_items = paginator.generate_items();
    let next_page_url = paginator.next_url(&current_url);
    let prev_page_url = paginator.previous_url(&current_url);
    let offset = ((current_page.get() - 1) * (page_size as u16)) as i64;
    let posts = stores::blog::get_published_uncategorized_blogposts(
        Some(offset),
        Some(page_size as i64),
        &db,
    )
    .await
    .map_err(PageError::EdgeDBQueryError)?;
    let categories = stores::blog::get_blog_categories(None, None, &db)
        .await
        .map_err(PageError::EdgeDBQueryError)?;
    let lang = session.get::<String>(KEY_LANG).unwrap_or(DEFAULT_LANG.into());
    let no_tracking = auth.current_user.is_some();
    let context = context!(
        posts => posts,
        pagelink_items => pagelink_items,
        next_page_url => next_page_url,
        prev_page_url => prev_page_url,
        categories => categories,
        lang => lang,
        no_tracking => no_tracking);
    let content = render_with("blog/post_list.jinja", context, jinja)?;
    Ok(Html(content))
}
