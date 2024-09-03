use std::str::FromStr;

use chrono::DateTime;
use fluent_templates::Loader;
use http::Uri;
use minijinja::value::{Kwargs, Value as MJValue, ValueKind};
use minijinja::{Error, ErrorKind, State};
use once_cell::sync::Lazy;
use regex::Regex;
use unic_langid::LanguageIdentifier;

use crate::consts::{DEFAULT_LANG, KEY_LANG};
use crate::thingsup::LOCALES;
use crate::types::conversions::jinja_kwargs_to_fluent_args;
use crate::types::BundledTemplates;
use crate::utils::urls::update_entry_in_query;

pub fn debug_value(value: MJValue) -> &'static str {
    tracing::debug!("MiniJinja value: {:?}", value);
    tracing::debug!("Kind: {:?}", value.kind());
    ""
}

pub fn post_detail_url(post: MJValue) -> Result<String, Error> {
    if post.kind() != ValueKind::Map {
        return Ok("#".into());
    }
    let created_at: String = post
        .get_attr("created_at")?
        .as_str()
        .ok_or(Error::new(
            ErrorKind::NonPrimitive,
            "created_at is not a string",
        ))?
        .into();
    let created_at = DateTime::parse_from_rfc3339(&created_at).map_err(|_e| {
        Error::new(
            ErrorKind::BadSerialization,
            "Expect RFC3339 datetime string",
        )
    })?;
    let slug: String = post
        .get_attr("slug")?
        .as_str()
        .ok_or(Error::new(ErrorKind::NonPrimitive, "slug is not a string"))?
        .into();
    Ok(format!("/post/{}/{}", created_at.format("%Y/%m"), slug))
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
    let lang_in_context = state
        .lookup(KEY_LANG)
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or(DEFAULT_LANG.into());
    let li = LanguageIdentifier::from_str(&lang_in_context).unwrap_or_else(|e| {
        tracing::error!(
            "Failed to parse {} as LanguageIdentifier. Error: {}",
            lang_in_context,
            e
        );
        LanguageIdentifier::default()
    });
    LOCALES.lookup_complete(&li, key, fluent_args.as_ref())
}

// Template loader for MiniJinja
pub fn get_embedded_template(name: &str) -> Result<Option<String>, Error> {
    tracing::debug!("To load embedded template: {}", name);
    let file = BundledTemplates::get(name);
    file.map(|f| String::from_utf8(f.data.into()))
        .transpose()
        .map_err(|e| Error::new(ErrorKind::SyntaxError, e.to_string()))
}
