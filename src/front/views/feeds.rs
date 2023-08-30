use atom_syndication::{Entry, FeedBuilder};
use axum::extract::{Query, State, Host};
use axum::response::{Result as AxumResult, IntoResponseParts};
use chrono::{TimeZone, Utc};
use edgedb_tokio::Client as EdgeClient;
use http::header::CONTENT_TYPE;

use super::super::structs::LaxPaging;
use crate::errors::PageError;
use crate::stores;

// Generate from Python: uuid.uuid5(uuid.NAMESPACE_DNS, 'quan.hoabinh.vn'
const SITE_UUID: &str = "4543aea6-ab17-5c18-9279-19e73529594d";

pub async fn gen_atom_feeds(
    Host(host): Host,
    Query(_paging): Query<LaxPaging>,
    State(db): State<EdgeClient>,
) -> AxumResult<(impl IntoResponseParts, String)> {
    let posts = stores::blog::get_published_posts(None, None, &db)
        .await
        .map_err(PageError::EdgeDBQueryError)?;
    let entries: Vec<Entry> = posts.iter().map(|p| p.to_atom_entry(Some(&host))).collect();
    let latest_post = stores::blog::get_latest_post(&db)
        .await
        .map_err(PageError::EdgeDBQueryError)?;
    let updated_at = latest_post
        .map(|p| p.updated_at.map(|d| d.into()))
        .flatten()
        .unwrap_or_else(|| Utc.with_ymd_and_hms(2013, 1, 1, 0, 0, 0).unwrap());
    let feed = FeedBuilder::default()
        .title("QuanWeb")
        .id(format!("urn:uuid:{SITE_UUID}"))
        .updated(updated_at)
        .entries(entries)
        .build();
    Ok(([(CONTENT_TYPE, "application/atom+xml; charset=utf-8")], feed.to_string()))
}
