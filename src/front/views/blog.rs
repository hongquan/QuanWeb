use std::num::NonZeroU16;

use uuid::Uuid;
use axum::extract::{Path, State, OriginalUri};
use axum::http::StatusCode;
use axum::response::{Html, Result as AxumResult};
use axum_extra::extract::Query;
use minijinja::context;

use super::super::structs::{LaxPaging, PostPageParams};
use crate::auth::Auth;
use crate::consts::DEFAULT_PAGE_SIZE;
use crate::errors::PageError;
use crate::stores;
use crate::stores::blog::{
    get_blogpost_by_slug, get_next_post, get_previous_post,
};
use crate::types::{AppState, Paginator};
use super::render_with;

pub async fn show_post(
    auth: Auth,
    Path((_y, _m, slug)): Path<(u16, u16, String)>,
    _params: Query<PostPageParams>,
    State(state): State<AppState>,
) -> AxumResult<Html<String>> {
    let AppState { db, jinja } = state;
    let post = get_blogpost_by_slug(slug, &db)
        .await
        .map_err(PageError::EdgeDBQueryError)?
        .ok_or((StatusCode::NOT_FOUND, "No post at this URL"))?;
    let user = auth.current_user;
    let no_tracking = !post.is_published.unwrap_or(false) || user.is_some();
    let prev_post = get_previous_post(post.created_at, None, &db)
        .await
        .map_err(PageError::EdgeDBQueryError)?;
    tracing::debug!("Previous post: {:?}", prev_post);
    let next_post = get_next_post(post.created_at, None, &db)
        .await
        .map_err(PageError::EdgeDBQueryError)?;
    tracing::debug!("Next post: {:?}", next_post);
    let context = context!(post => post, prev_post => prev_post, next_post => next_post, no_tracking => no_tracking);
    let content = render_with("blog/post.jinja", context, jinja)?;
    Ok(Html(content))
}

pub async fn list_posts(
    auth: Auth,
    Path(cat_slug): Path<String>,
    OriginalUri(current_url): OriginalUri,
    Query(paging): Query<LaxPaging>,
    State(state): State<AppState>,
) -> AxumResult<Html<String>> {
    let AppState { db, jinja } = state;
    let current_page = paging
        .page
        .and_then(|p| NonZeroU16::new(p.parse().ok()?))
        .unwrap_or(NonZeroU16::MIN);
    let cat = stores::blog::get_blog_category_by_slug(&cat_slug, &db)
        .await
        .map_err(PageError::EdgeDBQueryError)?
        .ok_or((StatusCode::NOT_FOUND, "No post at this URL"))?;
    let posts = stores::blog::get_blogposts_under_category(Some(cat_slug), None, None, &db)
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
    let no_tracking = auth.current_user.is_some();
    let context = context!(
        posts => posts,
        cat => cat,
        pagelink_items => pagelink_items,
        next_page_url => next_page_url,
        prev_page_url => prev_page_url,
        categories => categories,
        no_tracking => no_tracking);
    let content = render_with("blog/post_list.jinja", context, jinja)?;
    Ok(Html(content))
}

pub async fn preview_post(auth: Auth, Path(id): Path<Uuid>, State(state): State<AppState>) -> AxumResult<Html<String>> {
    let _user = auth.current_user.ok_or(PageError::PermissionDenied);
    let AppState { db, jinja } = state;
    let post = stores::blog::get_blogpost(id, &db)
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
    let context = context!(post => post, prev_post => prev_post, next_post => next_post, no_tracking => true);
    let content = render_with("blog/post.jinja", context, jinja)?;
    Ok(Html(content))
}
