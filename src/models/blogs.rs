use std::str::FromStr;

use chrono::{Utc, DateTime};
use edgedb_derive::Queryable;
use edgedb_protocol::model::Datetime as EDatetime;
use edgedb_protocol::value::Value as EValue;
use http::Uri;
use serde::{Deserialize, Serialize};
use serde_json::Value as JValue;
use strum_macros::{Display, EnumString, IntoStaticStr};
use uuid::Uuid;
use atom_syndication::{Entry as AtomEntry, EntryBuilder, Link, LinkBuilder, Category as AtomCategory, CategoryBuilder, Text};

use crate::types::conversions::{
    serialize_edge_datetime, serialize_optional_edge_datetime,
};

#[derive(
    Debug,
    Eq,
    PartialEq,
    Default,
    Clone,
    EnumString,
    Display,
    IntoStaticStr,
    Serialize,
    Deserialize,
    Queryable,
)]
pub enum DocFormat {
    #[default]
    Md,
    Rst,
}

impl From<&JValue> for DocFormat {
    fn from(v: &JValue) -> Self {
        match v {
            JValue::String(s) => DocFormat::from_str(s.as_str()).unwrap_or_default(),
            _ => DocFormat::Md,
        }
    }
}

impl From<DocFormat> for EValue {
    fn from(df: DocFormat) -> Self {
        let v: &str = df.into();
        EValue::Enum(v.into())
    }
}

// Struct to represent a BlogPost in the database, but with just enough fields to display in a list.
#[serde_with::apply(
    EDatetime => #[serde(serialize_with = "serialize_edge_datetime")],
    Option<EDatetime> => #[serde(serialize_with = "serialize_optional_edge_datetime")],
)]
#[derive(Debug, Clone, Serialize, Queryable)]
pub struct MediumBlogPost {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub excerpt: Option<String>,
    pub is_published: Option<bool>,
    pub published_at: Option<EDatetime>,
    pub created_at: EDatetime,
    pub updated_at: Option<EDatetime>,
    pub categories: Vec<BlogCategory>,
}

impl MediumBlogPost {
    pub fn get_view_url(&self, base_url: Option<&Uri>) -> String {
        let created_at: DateTime<Utc> = self.created_at.into();
        let url_path = format!("/post/{}/{}", created_at.format("%Y/%m"), self.slug);
        let host = base_url.map(|u| u.authority()).flatten();
        if let Some(host) = host {
            let scheme = base_url.map(|u| u.scheme_str()).flatten().unwrap_or("https");
            format!("{scheme}://{host}{url_path}")
        } else {
            url_path
        }
    }

    pub fn to_atom_entry(&self, host: Option<&str>) -> AtomEntry {
        let mut entry = AtomEntry::from(self.clone());
        if let Some(host) = host {
            let links = entry.links();
            let absolute_links: Vec<Link> = links.into_iter().map(|l| {
                let mut link = l.clone();
                if l.href().starts_with("/") {
                    let path = l.href();
                    let scheme = if host == "localhost" { "http" } else { "https" };
                    link.set_href(format!("{scheme}://{host}{path}"));
                    link
                } else {
                    link
                }
            }).collect();
            entry.set_links(absolute_links);
        };
        entry
    }
}

impl Default for MediumBlogPost {
    fn default() -> Self {
        let created_at = Utc::now().try_into().unwrap_or(EDatetime::MIN);
        Self {
            id: Uuid::default(),
            title: String::default(),
            slug: String::default(),
            excerpt: None,
            is_published: Some(false),
            published_at: None,
            created_at,
            updated_at: None,
            categories: Vec::default(),
        }
    }
}

impl From<MediumBlogPost> for AtomEntry {
    fn from(value: MediumBlogPost) -> Self {
        let url = value.get_view_url(None);
        let MediumBlogPost {
            id,
            title,
            excerpt,
            published_at,
            created_at,
            updated_at,
            categories,
            ..
        } = value;
        let entry_id = format!("urn:uuid:{id}");
        let updated_at: DateTime<Utc> = updated_at.unwrap_or(created_at).into();
        let link = LinkBuilder::default()
            .href(url)
            .mime_type(Some("text/html".into()))
            .build();
        let categories: Vec<AtomCategory> = categories.into_iter().collect();
        let mut builder = EntryBuilder::default();
        builder.title(title)
            .id(entry_id)
            .summary(excerpt.map(Text::html))
            .links(vec![link])
            .published(published_at.map(|d| DateTime::<Utc>::from(d).into()))
            .updated(updated_at)
            .categories(categories);
        builder.build()
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Queryable)]
pub struct BlogCategory {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
}

impl From<BlogCategory> for AtomCategory {
    fn from(value: BlogCategory) -> Self {
        let BlogCategory { title, slug, .. } = value;
        CategoryBuilder::default()
            .term(slug)
            .label(Some(title))
            .build()
    }
}

impl FromIterator<BlogCategory> for Vec<AtomCategory> {
    fn from_iter<T: IntoIterator<Item = BlogCategory>>(iter: T) -> Self {
        iter.into_iter().map(AtomCategory::from).collect()
    }
}

// Struct to represent a BlogPost in the database, with all fields to display in a detail page.
#[serde_with::apply(
    EDatetime => #[serde(serialize_with = "serialize_edge_datetime")],
    Option<EDatetime> => #[serde(serialize_with = "serialize_optional_edge_datetime")],
)]
#[derive(Debug, Serialize, Queryable)]
pub struct DetailedBlogPost {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub is_published: Option<bool>,
    pub published_at: Option<EDatetime>,
    pub created_at: EDatetime,
    pub updated_at: Option<EDatetime>,
    pub categories: Vec<BlogCategory>,
    pub body: Option<String>,
    pub format: DocFormat,
    pub locale: Option<String>,
    pub excerpt: Option<String>,
    pub html: Option<String>,
    pub seo_description: Option<String>,
    pub og_image: Option<String>,
}

impl Default for DetailedBlogPost {
    fn default() -> Self {
        let created_at = Utc::now().try_into().unwrap_or(EDatetime::MIN);
        Self {
            id: Uuid::default(),
            title: String::default(),
            slug: String::default(),
            is_published: Some(false),
            published_at: None,
            created_at,
            updated_at: None,
            categories: Vec::default(),
            body: None,
            format: DocFormat::Md,
            locale: None,
            excerpt: None,
            html: None,
            seo_description: None,
            og_image: None,
        }
    }
}

// Struct to represent a BlogPost in the database, with just a few fields enough to build links.
#[derive(Debug, Serialize, Queryable)]
pub struct MiniBlogPost {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    #[serde(serialize_with = "serialize_edge_datetime")]
    pub created_at: EDatetime,
    #[serde(serialize_with = "serialize_optional_edge_datetime")]
    pub updated_at: Option<EDatetime>,
}
