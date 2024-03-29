pub mod conversions;
pub mod ext;
#[cfg(test)]
pub mod tests;

use std::num::NonZeroU16;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::extract::FromRef;
use axum::http::header::{CONTENT_TYPE, LAST_MODIFIED};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use edgedb_protocol::codec::ShapeElement;
use edgedb_protocol::common::Cardinality;
use edgedb_tokio::Client;
use http::Uri;
use indexmap::IndexMap;
use minijinja::Environment;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::utils::urls::update_entry_in_query;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorShape {
    pub message: String,
    pub fields: Option<IndexMap<String, String>>,
    pub code: Option<String>,
}

impl Default for ApiErrorShape {
    fn default() -> Self {
        Self {
            message: "Some error".to_string(),
            fields: None,
            code: None,
        }
    }
}

impl From<String> for ApiErrorShape {
    fn from(message: String) -> Self {
        Self {
            message,
            ..Default::default()
        }
    }
}

impl From<IndexMap<&str, String>> for ApiErrorShape {
    fn from(value: IndexMap<&str, String>) -> Self {
        Self {
            fields: Some(value.into_iter().map(|(k, v)| (k.to_string(), v)).collect()),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, FromRef)]
pub struct AppState {
    pub db: Client,
    pub jinja: Environment<'static>,
}

#[derive(RustEmbed)]
#[folder = "static"]
#[exclude = "vendor/alpine*.js"]
#[exclude = "fonts/*"]
pub struct Assets;

pub struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();

        match Assets::get(path.as_str()) {
            Some(file) => {
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                let last_modified = file.metadata.last_modified();
                let last_modified: DateTime<Utc> = last_modified
                    .and_then(|t| UNIX_EPOCH.checked_add(Duration::from_secs(t)))
                    .unwrap_or(SystemTime::now())
                    .into();
                let headers = [
                    (CONTENT_TYPE, mime.to_string()),
                    (LAST_MODIFIED, last_modified.to_rfc2822()),
                ];
                (headers, file.data).into_response()
            }
            None => (StatusCode::NOT_FOUND, "File Not Found").into_response(),
        }
    }
}

#[derive(RustEmbed)]
#[folder = "minijinja"]
pub struct BundledTemplates;

#[derive(Debug, Clone, PartialEq, Serialize, SmartDefault)]
pub struct PageLinkItem {
    #[default(NonZeroU16::MIN)]
    pub page: NonZeroU16,
    pub is_current: bool,
    pub is_ellipsis: bool,
}

impl PageLinkItem {
    pub fn new(page: NonZeroU16, is_current: bool, is_ellipsis: bool) -> Self {
        Self {
            page,
            is_current,
            is_ellipsis,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, SmartDefault)]
pub struct Paginator {
    #[default(NonZeroU16::MIN)]
    pub current_page: NonZeroU16,
    #[default(NonZeroU16::MIN)]
    pub total_pages: NonZeroU16,
}

impl Paginator {
    pub const PADDING: u16 = 3;
    const TOTAL_DISPLAY: u16 = Self::PADDING * 2 + 1;
    // Rule:
    // - Current page is always surrounded by 1 to 2 pages on each side, except when it is the first or the last.
    // - When current page is around beginning or around the end, we show one ellipsis
    // - When current page is at the middle, we show two ellipses.
    // - Total of items to show is always 7 or less, including ellipses.
    pub fn generate_items(&self) -> Vec<PageLinkItem> {
        let total_pages = self.total_pages.get();
        if total_pages < self.current_page.get() {
            return Vec::new();
        }
        if total_pages <= Self::TOTAL_DISPLAY {
            let range = 1..=total_pages;
            let items = range.map(|i| {
                let page = NonZeroU16::new(i).unwrap_or(NonZeroU16::MIN);
                PageLinkItem::new(page, page == self.current_page, false)
            });
            return items.collect();
        }
        let current_page = self.current_page.get();
        let elli_index = if current_page == Self::PADDING {
            current_page + 1
        } else if current_page == total_pages - Self::PADDING + 1 {
            Self::TOTAL_DISPLAY - Self::PADDING - 2
        } else {
            Self::PADDING
        };
        (0..Self::TOTAL_DISPLAY)
            .map(|i| {
                // When current page is around beginning or around the end, we show one ellipsis
                let (is_ellipsis, page) = if current_page <= Self::PADDING
                    || current_page > total_pages - Self::PADDING
                {
                    let is_ellipsis = i == elli_index;
                    let page = if i < elli_index {
                        i + 1
                    } else {
                        total_pages - 2 * Self::PADDING + i
                    };
                    (is_ellipsis, page)
                } else {
                    // When current page is at the middle, we show two ellipses.
                    // We don't base on ellipsisIndex anymore.
                    let page = if i == 0 {
                        // Always show first page
                        1
                    } else if i == Self::TOTAL_DISPLAY - 1 {
                        // Always show last page
                        total_pages
                    } else {
                        // Show the closest neighbor pages
                        current_page - Self::PADDING + i
                    };
                    let is_ellipsis = i == 1 || i == Self::TOTAL_DISPLAY - 2;
                    (is_ellipsis, page)
                };
                let page = NonZeroU16::new(page).unwrap_or(NonZeroU16::MIN);
                PageLinkItem::new(page, page == self.current_page, is_ellipsis)
            })
            .collect()
    }

    pub fn next_url(&self, current_url: &Uri) -> Option<String> {
        let next_page = self.current_page.saturating_add(1u16);
        if next_page <= self.total_pages {
            Some(update_entry_in_query("page", next_page, current_url).to_string())
        } else {
            None
        }
    }

    pub fn previous_url(&self, current_url: &Uri) -> Option<String> {
        if self.current_page > NonZeroU16::MIN {
            let prev_page = self.current_page.get() - 1;
            Some(update_entry_in_query("page", prev_page, current_url).to_string())
        } else {
            None
        }
    }

    pub fn first_url(&self, current_url: &Uri) -> String {
        update_entry_in_query("page", 1, current_url).to_string()
    }

    pub fn last_url(&self, current_url: &Uri) -> String {
        update_entry_in_query("page", self.total_pages, current_url).to_string()
    }
}

pub trait EdgeSelectable {
    fn fields_as_shape() -> String;
}

pub fn create_shape_element<N: ToString>(name: N, cardinality: Cardinality) -> ShapeElement {
    ShapeElement {
        name: name.to_string(),
        cardinality: Some(cardinality),
        flag_link: false,
        flag_link_property: false,
        flag_implicit: false,
    }
}

#[derive(Debug, Deserialize, SmartDefault)]
#[serde(default)]
pub struct CodeFenceOptions {
    pub lines: bool,
    #[default = 1]
    pub start_line: u8,
}
