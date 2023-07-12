use std::str::FromStr;

use once_cell::sync::Lazy;
use http::Uri;
use regex::Regex;
use chrono::DateTime;
use minijinja::value::Value as MJValue;

use crate::utils::urls::update_entry_in_query;

pub fn debug_value(value: MJValue) -> &'static str {
    tracing::debug!("MiniJinja value: {:?}", value);
    tracing::debug!("Kind: {:?}", value.kind());
    ""
}

pub fn post_detail_url(slug: String, created_at: String) -> String {
    match DateTime::parse_from_rfc3339(&created_at) {
        Ok(x) => {
            format!("/post/{}/{}", x.format("%Y/%m"), slug)
        },
        Err(e) => {
            tracing::error!("Failed to parse datetime: {:?}", e);
            format!("/post/y/m/{}", slug)
        }
    }
}

pub fn gen_element_attr(name: &str, value: MJValue) -> String {
    match value.as_str() {
        Some(value) => format!("{}=\"{}\"", name, value),
        None => String::new(),
    }
}

pub fn add_url_param(url: String, name: String, value: String) -> String {
    if let Ok(x) = Uri::from_str(&url) {
        update_entry_in_query(&name, value, &x).to_string()
    } else {
        url
    }
}

// Ref: https://github.com/pallets/markupsafe/blob/main/src/markupsafe/__init__.py
pub fn striptags(html: String) -> String {
    static RE_COMMENTS: Lazy<Regex> = Lazy::new(|| Regex::new(r"<!--.*?-->").unwrap());
    static RE_TAGS: Lazy<Regex> = Lazy::new(|| Regex::new(r"<[^>]*>").unwrap());
    let stripped = RE_COMMENTS.replace_all(&html, "");
    let stripped = RE_TAGS.replace_all(&stripped, "");
    stripped.to_string()
}
