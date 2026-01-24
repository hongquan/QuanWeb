use std::str::FromStr;

use atom_syndication::{
    Category as AtomCategory, CategoryBuilder, Entry as AtomEntry, EntryBuilder, LinkBuilder,
    Person, Text,
};
use chrono::{DateTime, Utc};
use field_names::FieldNames;
use gel_derive::Queryable;
use gel_protocol::model::Datetime as EDatetime;
use gel_protocol::value::Value as EValue;
use serde::{Deserialize, Serialize};
use serde_json::Value as JValue;
use sitemap_writer::SitemapUrl;
use strum::{Display, EnumString, IntoStaticStr};
use uuid::Uuid;

use super::feeds::{JsonAuthor, JsonItem};
use super::users::MiniUser;
use crate::types::EdgeSelectable;
use crate::types::conversions::{serialize_edge_datetime, serialize_optional_edge_datetime};
use crate::utils::html::strip_tags;

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
#[derive(Debug, Clone, Serialize, Queryable, FieldNames)]
pub struct MediumBlogPost {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub locale: Option<String>,
    pub excerpt: Option<String>,
    pub is_published: Option<bool>,
    pub published_at: Option<EDatetime>,
    pub created_at: EDatetime,
    pub updated_at: Option<EDatetime>,
    pub categories: Vec<BlogCategory>,
    pub author: Option<MiniUser>,
}

impl MediumBlogPost {
    pub fn get_view_url(&self) -> String {
        let created_at: DateTime<Utc> = self.created_at.into();
        format!("/post/{}/{}", created_at.format("%Y/%m"), self.slug)
    }
}

impl Default for MediumBlogPost {
    fn default() -> Self {
        let created_at = Utc::now().try_into().unwrap_or(EDatetime::MIN);
        Self {
            id: Uuid::default(),
            title: String::default(),
            slug: String::default(),
            locale: None,
            excerpt: None,
            is_published: Some(false),
            published_at: None,
            created_at,
            updated_at: None,
            categories: Vec::default(),
            author: None,
        }
    }
}

impl EdgeSelectable for MediumBlogPost {
    fn fields_as_shape() -> String {
        let fields: Vec<String> = Self::FIELDS
            .into_iter()
            .map(|s| match s {
                "categories" => {
                    let cat_shape = BlogCategory::fields_as_shape();
                    format!("categories: {cat_shape}")
                }
                "author" => {
                    let user_shape = MiniUser::fields_as_shape();
                    format!("author: {user_shape}")
                }
                _ => s.to_string(),
            })
            .collect();
        format!("{{ {} }}", fields.join(", "))
    }
}

impl From<MediumBlogPost> for AtomEntry {
    fn from(value: MediumBlogPost) -> Self {
        let url = value.get_view_url();
        let MediumBlogPost {
            id,
            title,
            excerpt,
            published_at,
            created_at,
            updated_at,
            categories,
            author,
            ..
        } = value;
        let entry_id = format!("urn:uuid:{id}");
        let updated_at: DateTime<Utc> = updated_at.unwrap_or(created_at).into();
        let link = LinkBuilder::default()
            .href(url)
            .mime_type(Some("text/html".into()))
            .build();
        let categories: Vec<AtomCategory> = categories.into_iter().collect();
        let authors = if let Some(author) = author {
            vec![Person::from(author)]
        } else {
            vec![]
        };
        let mut builder = EntryBuilder::default();
        builder
            .title(title)
            .id(entry_id)
            .summary(excerpt.map(Text::html))
            .links(vec![link])
            .published(published_at.map(|d| DateTime::<Utc>::from(d).into()))
            .updated(updated_at)
            .categories(categories)
            .authors(authors);
        builder.build()
    }
}

impl From<MediumBlogPost> for JsonItem {
    fn from(value: MediumBlogPost) -> Self {
        let url = value.get_view_url();
        let MediumBlogPost {
            id,
            title,
            excerpt,
            locale,
            published_at,
            created_at,
            updated_at,
            categories,
            author,
            ..
        } = value;
        let entry_id = format!("urn:uuid:{id}");
        let updated_at: DateTime<Utc> = updated_at.unwrap_or(created_at).into();
        let categories: Vec<String> = categories.into_iter().map(|c| c.title).collect();
        let author = author.map(JsonAuthor::from);
        JsonItem {
            id: entry_id,
            url: Some(url),
            external_url: None,
            title: Some(title),
            content_html: None,
            content_text: None,
            summary: excerpt.as_deref().map(strip_tags),
            date_published: published_at.map(|d| DateTime::<Utc>::from(d).to_rfc3339()),
            date_modified: Some(updated_at.to_rfc3339()),
            author,
            tags: Some(categories),
            language: locale,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Queryable, FieldNames)]
pub struct BlogCategory {
    pub id: Uuid,
    pub title: String,
    pub title_vi: Option<String>,
    pub slug: String,
}

impl EdgeSelectable for BlogCategory {
    fn fields_as_shape() -> String {
        let fields = Self::FIELDS.join(", ");
        format!("{{ {fields} }}")
    }
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
#[derive(Debug, Serialize, Queryable, FieldNames)]
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
    pub author: Option<MiniUser>,
    pub seo_description: Option<String>,
    pub og_image: Option<String>,
}

impl DetailedBlogPost {
    pub fn get_canonical_url(&self) -> String {
        let created_at = DateTime::<Utc>::from(self.created_at);
        format!("/post/{}/{}", created_at.format("%Y/%m"), self.slug)
    }
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
            author: None,
            seo_description: None,
            og_image: None,
        }
    }
}

impl EdgeSelectable for DetailedBlogPost {
    fn fields_as_shape() -> String {
        let fields: Vec<String> = Self::FIELDS
            .into_iter()
            .map(|s| match s {
                "categories" => {
                    let cat_shape = BlogCategory::fields_as_shape();
                    format!("categories: {cat_shape}")
                }
                "author" => {
                    let user_shape = MiniUser::fields_as_shape();
                    format!("author: {user_shape}")
                }
                _ => s.to_string(),
            })
            .collect();
        format!("{{ {} }}", fields.join(", "))
    }
}

// Struct to represent a BlogPost in the database, with just a few fields enough to build links.
#[derive(Debug, Clone, Serialize, Queryable, FieldNames)]
pub struct MiniBlogPost {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    #[serde(serialize_with = "serialize_edge_datetime")]
    pub created_at: EDatetime,
    #[serde(serialize_with = "serialize_optional_edge_datetime")]
    pub updated_at: Option<EDatetime>,
}

impl MiniBlogPost {
    pub fn to_sitemap_entry(&self, base_url: &str) -> SitemapUrl {
        let created_at = DateTime::<Utc>::from(self.created_at);
        let loc = format!(
            "{}/post/{}/{}",
            base_url,
            created_at.format("%Y/%m"),
            self.slug
        );
        let lastmod = self
            .updated_at
            .map(DateTime::<Utc>::from)
            .map(|d| format!("{}", d.format("%Y-%m-%d")));
        SitemapUrl {
            loc,
            lastmod,
            ..Default::default()
        }
    }
}

impl EdgeSelectable for MiniBlogPost {
    fn fields_as_shape() -> String {
        let fields = Self::FIELDS.join(", ");
        format!("{{ {fields} }}")
    }
}
