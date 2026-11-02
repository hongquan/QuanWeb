use std::str::FromStr;

use chrono::DateTime;
use fluent_templates::Loader;
use http::Uri;
use lazy_regex::{regex_captures, regex_remove_all};
use minijinja::value::{Kwargs, Value as MJValue, ValueKind};
use minijinja::{Error, ErrorKind, State};
use unic_langid::LanguageIdentifier;

use crate::consts::{DEFAULT_LANG, KEY_LANG};
use crate::models::blogs::build_post_view_url;
use crate::thingsup::LOCALES;
use crate::types::BundledTemplates;
use crate::types::conversions::jinja_kwargs_to_fluent_args;
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
    let created_at: chrono::DateTime<chrono::Utc> = created_at.into();
    Ok(build_post_view_url(created_at, &slug))
}

pub fn category_url(slug: String) -> String {
    format!("/category/{}/", slug)
}

pub fn img_srcset(url: MJValue) -> String {
    let Some(url) = url.as_str() else {
        return String::new();
    };
    let Some((_, dir, file)) = regex_captures!(r"^(.*/blogs/\d{4}/\d{2}/)([^/?]+)$", url)
    else {
        return String::new();
    };
    format!("{dir}.thumb-320/{file} 320w, {dir}.thumb-400/{file} 400w, {url} 800w")
}

pub fn gen_element_attr(name: &str, value: MJValue) -> String {
    match value.as_str() {
        Some(value) => format!("{}=\"{}\"", name, value),
        None => String::new(),
    }
}

pub fn add_url_param(url: String, name: String, value: String) -> String {
    match Uri::from_str(&url) {
        Ok(x) => update_entry_in_query(&name, value, &x).to_string(),
        _ => url,
    }
}

// Ref: https://github.com/pallets/markupsafe/blob/main/src/markupsafe/__init__.py
pub fn striptags(html: String) -> String {
    let stripped = regex_remove_all!(r"<!--.*?-->", &html);
    regex_remove_all!(r"<[^>]*>", &stripped).to_string()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn img_srcset_rewrites_cdn_blog_urls() {
        assert_eq!(
            img_srcset(MJValue::from(
                "https://quan-images.b-cdn.net/blogs/2026/08/huong_buoi_bao_vinh_long.jpg"
            )),
            "https://quan-images.b-cdn.net/blogs/2026/08/.thumb-320/huong_buoi_bao_vinh_long.jpg 320w, \
             https://quan-images.b-cdn.net/blogs/2026/08/.thumb-400/huong_buoi_bao_vinh_long.jpg 400w, \
             https://quan-images.b-cdn.net/blogs/2026/08/huong_buoi_bao_vinh_long.jpg 800w"
        );
    }

    #[test]
    fn img_srcset_ignores_unmatched_urls() {
        for url in [
            "https://i.imgur.com/abc123.jpg",
            "https://quan-images.b-cdn.net/blogs/2026/x.jpg",
            "https://quan-images.b-cdn.net/blogs/2026/08/x.jpg?v=2",
            "",
        ] {
            assert_eq!(img_srcset(MJValue::from(url)), "", "url: {url}");
        }
        assert_eq!(img_srcset(MJValue::from(())), "");
    }

    #[test]
    fn all_icon_macros_render() {
        let icon_names = [
            "search", "rss", "menu_toggle", "chevron_down", "chevron_left", "chevron_right",
            "chevron_right_outline", "arrow_right_outline", "document_text", "photograph",
            "calendar", "presentation", "book_open", "link", "markdown", "folder", "github",
            "bitbucket", "gitlab", "linkedin",
        ];
        for name in icon_names {
            let source = format!("{{% from 'icons.jinja' import {name} %}}{{{{ {name}() }}}}");
            let mut probe = minijinja::Environment::new();
            probe.set_loader(get_embedded_template);
            probe.add_template("icon_probe", &source).unwrap();
            let rendered = probe.get_template("icon_probe").unwrap().render(()).unwrap();
            assert!(rendered.contains("<svg"), "icon '{name}' did not render an <svg>");
        }
    }

    #[test]
    fn img_srcset_rewrites_root_relative_blog_urls() {
        assert_eq!(
            img_srcset(MJValue::from("/blogs/2026/08/x.jpg")),
            "/blogs/2026/08/.thumb-320/x.jpg 320w, \
             /blogs/2026/08/.thumb-400/x.jpg 400w, \
             /blogs/2026/08/x.jpg 800w"
        );
    }
}
