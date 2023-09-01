// Just copy from https://github.com/feed-rs/feed-rs/

use http::Uri;
use serde::Serialize;
use atom_syndication::Entry;

pub const DEFAULT_SITE_URL: &str = "https://quan.hoabinh.vn";

#[derive(Debug, Serialize)]
pub struct JsonFeed {
    pub version: String,
    pub title: String,
    pub home_page_url: Option<String>,
    pub feed_url: Option<String>,
    pub next_url: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub favicon: Option<String>,
    pub authors: Option<Vec<JsonAuthor>>,
    pub items: Vec<JsonItem>,
}

impl Default for JsonFeed {
    fn default() -> Self {
        Self {
            version: "https://jsonfeed.org/version/1.1".into(),
            title: "QuanWeb".into(),
            home_page_url: Some(DEFAULT_SITE_URL.to_string()),
            feed_url: None,
            next_url: None,
            description: Some("Blog about programming, culture, history".into()),
            icon: None,
            favicon: None,
            authors: None,
            items: vec![],
        }
    }
}

#[derive(Debug, Serialize)]
pub struct JsonAuthor {
    pub name: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct JsonItem {
    pub id: String,
    pub url: Option<String>,
    pub external_url: Option<String>,
    pub title: Option<String>,
    pub content_html: Option<String>,
    pub content_text: Option<String>,
    pub summary: Option<String>,
    pub date_published: Option<String>,
    pub date_modified: Option<String>,
    pub authors: Option<Vec<JsonAuthor>>,
    pub tags: Option<Vec<String>>,
    pub language: Option<String>,
}

pub trait EntryExt {
    fn prepend_url(&mut self, base_url: &Uri);
}

impl EntryExt for Entry {
    fn prepend_url(&mut self, base_url: &Uri) {
        self.links.iter_mut().for_each(|u| {
            let old_url = u.href();
            if old_url.starts_with("/") {
                u.set_href(format!("{base_url}{old_url}"));
            }
        });
    }
}
