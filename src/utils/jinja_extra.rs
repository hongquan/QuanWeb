use std::str::FromStr;

use once_cell::sync::Lazy;
use http::Uri;
use regex::Regex;
use chrono::DateTime;
use minijinja::State;
use minijinja::value::{Value as MJValue, Kwargs};
use fluent_templates::Loader;
use unic_langid::LanguageIdentifier;

use crate::consts::{KEY_LANG, DEFAULT_LANG};
use crate::utils::urls::update_entry_in_query;
use crate::thingsup::LOCALES;
use crate::types::conversions::jinja_kwargs_to_fluent_args;

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
            tracing::error!("Failed to parse {} as datetime: {:?}", created_at, e);
            format!("/post/y/m/{}", slug)
        }
    }
}

pub fn category_url(slug: String) -> String {
    format!("/category/{}/", slug)
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

// Function to provide translation via Fluent
pub fn fluent(state: &State, key: &str, kwargs: Kwargs) -> String {
    let fluent_args = jinja_kwargs_to_fluent_args(kwargs);
    if fluent_args.is_some() {
        tracing::debug!("Args to pass to fluent message: {:?}", fluent_args);
    }
    let lang_in_context = state.lookup(KEY_LANG).and_then(|v| v.as_str().map(|s| s.to_string())).unwrap_or(DEFAULT_LANG.into());
    let li = LanguageIdentifier::from_str(&lang_in_context).unwrap_or_else(|e| {
        tracing::error!("Failed to parse {} as LanguageIdentifier. Error: {}", lang_in_context, e);
        LanguageIdentifier::default()
    });
    LOCALES.lookup_complete(&li, key, fluent_args.as_ref()).unwrap_or_else(|| key.to_string())
}
