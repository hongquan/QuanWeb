use std::num::NonZeroU16;

use atom_syndication::{Entry, FeedBuilder, LinkBuilder};
use axum::extract::{OriginalUri, Query, State};
use axum::http::header;
use axum::response::{IntoResponseParts, Json, Result as AxumResult};
use axum_extra::extract::Host;
use chrono::{TimeZone, Utc};
use gel_tokio::Client as EdgeClient;
use http::{HeaderName, StatusCode};
use http::{Uri, header::CONTENT_TYPE};
use sitemap_writer::SitemapWriter;

use super::super::structs::LaxPaging;
use crate::consts::DEFAULT_PAGE_SIZE;
use crate::errors::PageError;
use crate::models::feeds::{DEFAULT_SITE_URL, EntryExt, JsonFeed, JsonItem};
use crate::stores;
use crate::types::{Paginator, ext::UriExt};

// Generate from Python: uuid.uuid5(uuid.NAMESPACE_DNS, 'quan.hoabinh.vn'
const SITE_UUID: &str = "4543aea6-ab17-5c18-9279-19e73529594d";

pub async fn gen_atom_feeds(
    Host(host): Host,
    OriginalUri(current_url): OriginalUri,
    Query(paging): Query<LaxPaging>,
    State(db): State<EdgeClient>,
) -> AxumResult<(impl IntoResponseParts, String)> {
    let base_url: Uri = format!("https://{host}")
        .parse()
        .unwrap_or(Uri::from_static(DEFAULT_SITE_URL));
    let current_page = paging.get_page_as_number();
    let page_size = DEFAULT_PAGE_SIZE;
    let offset = ((current_page.get() - 1) * page_size as u16) as i64;
    let posts = stores::blog::get_published_posts(Some(offset), Some(page_size as i64), &db)
        .await
        .map_err(PageError::GelQueryError)?;
    let total = stores::blog::count_all_published_posts(&db)
        .await
        .map_err(PageError::GelQueryError)?;
    let total_pages = NonZeroU16::try_from((total as f64 / page_size as f64).ceil() as u16)
        .unwrap_or(NonZeroU16::MIN);
    let paginator = Paginator {
        current_page,
        total_pages,
    };
    let self_url = base_url.join(&current_url.to_string()).to_string();
    let first_page_url = paginator.first_url(&current_url);
    let last_page_url = paginator.last_url(&current_url);
    let next_page_url = paginator.next_url(&current_url);
    let prev_page_url = paginator.previous_url(&current_url);
    let mut links = vec![
        LinkBuilder::default()
            .rel("self".to_string())
            .href(self_url)
            .build(),
        LinkBuilder::default()
            .rel("first".to_string())
            .href(base_url.join(&first_page_url).to_string())
            .build(),
        LinkBuilder::default()
            .rel("last".to_string())
            .href(base_url.join(&last_page_url).to_string())
            .build(),
    ];
    if let Some(url) = next_page_url {
        links.push(
            LinkBuilder::default()
                .rel("next".to_string())
                .href(base_url.join(&url).to_string())
                .build(),
        )
    }
    if let Some(url) = prev_page_url {
        links.push(
            LinkBuilder::default()
                .rel("previous".to_string())
                .href(base_url.join(&url).to_string())
                .build(),
        )
    }
    let mut entries: Vec<Entry> = posts.into_iter().map(Entry::from).collect();
    entries.iter_mut().for_each(|e| e.prepend_url(&base_url));
    let latest_post = stores::blog::get_last_updated_post(&db)
        .await
        .map_err(PageError::GelQueryError)?;
    let updated_at = latest_post
        .and_then(|p| p.updated_at.map(|d| d.into()))
        .unwrap_or_else(|| Utc.with_ymd_and_hms(2013, 1, 1, 0, 0, 0).unwrap());
    let feed = FeedBuilder::default()
        .title("QuanWeb")
        .id(format!("urn:uuid:{SITE_UUID}"))
        .links(links)
        .updated(updated_at)
        .entries(entries)
        .build();
    Ok((
        [(CONTENT_TYPE, "application/atom+xml; charset=utf-8")],
        feed.to_string(),
    ))
}

pub async fn gen_json_feeds(
    Host(host): Host,
    OriginalUri(current_url): OriginalUri,
    Query(paging): Query<LaxPaging>,
    State(db): State<EdgeClient>,
) -> AxumResult<Json<JsonFeed>> {
    let base_url = format!("https://{host}");
    let current_page = paging.get_page_as_number();
    let page_size = DEFAULT_PAGE_SIZE;
    let offset = ((current_page.get() - 1) * page_size as u16) as i64;
    let posts = stores::blog::get_published_posts(Some(offset), Some(page_size as i64), &db)
        .await
        .map_err(PageError::GelQueryError)?;
    let total = stores::blog::count_all_published_posts(&db)
        .await
        .map_err(PageError::GelQueryError)?;
    let total_pages = NonZeroU16::try_from((total as f64 / page_size as f64).ceil() as u16)
        .unwrap_or(NonZeroU16::MIN);
    let paginator = Paginator {
        current_page,
        total_pages,
    };
    let next_page_url = paginator.next_url(&current_url);
    let mut feed = JsonFeed {
        feed_url: Some(format!("{base_url}{current_url}")),
        next_url: next_page_url.map(|url| format!("{base_url}{url}")),
        ..Default::default()
    };
    let mut items: Vec<JsonItem> = posts.into_iter().map(JsonItem::from).collect();
    items.iter_mut().for_each(|it| match it.url {
        Some(ref url) if url.starts_with('/') => {
            it.url = Some(format!("{base_url}{url}"));
        }
        _ => {}
    });
    feed.items = items;
    Ok(Json(feed))
}

pub async fn gen_sitemaps(
    State(db): State<EdgeClient>,
) -> AxumResult<(StatusCode, [(HeaderName, &'static str); 1], String)> {
    let posts = stores::blog::get_all_published_mini_posts(&db)
        .await
        .map_err(PageError::GelQueryError)?;
    
    let entries: Vec<_> = posts
        .iter()
        .map(|p| p.to_sitemap_entry(DEFAULT_SITE_URL))
        .collect();
    let headers = [(header::CONTENT_TYPE, "application/xml")];
    let xml = SitemapWriter::build(entries);
    Ok((StatusCode::OK, headers, xml))
}
