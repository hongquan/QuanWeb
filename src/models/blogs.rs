use std::str::FromStr;

use chrono::{DateTime, Utc};
use edgedb_derive::Queryable;
use edgedb_protocol::model::Datetime as EDatetime;
use edgedb_protocol::value::Value as EValue;
use serde::{Deserialize, Serialize};
use serde_json::Value as JValue;
use strum_macros::{Display, EnumString, IntoStaticStr};
use uuid::Uuid;

use crate::types::{serialize_edge_datetime, serialize_optional_edge_datetime};

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

/*
Because EdgeDB client cannot retrieve datetime field as chrono type,
We have to use an intermediate type to grab result from EdgeDB.
then convert to final struct with chrono types (which can be serialized with serde).
*/
#[serde_with::apply(
    EDatetime => #[serde(serialize_with = "serialize_edge_datetime")],
    Option<EDatetime> => #[serde(serialize_with = "serialize_optional_edge_datetime")],
)]
#[derive(Debug, Serialize, edgedb_derive::Queryable)]
pub struct RawBlogPost {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub is_published: Option<bool>,
    pub published_at: Option<EDatetime>,
    pub created_at: EDatetime,
    pub updated_at: Option<EDatetime>,
    pub categories: Vec<BlogCategory>,
}

impl RawBlogPost {
    pub fn type_cast_for_field<'a>(name: &'a str) -> &'a str {
        match name {
            "title" => "str",
            "slug" => "str",
            "is_published" => "bool",
            "published_at" => "optional datetime",
            "updated_at" => "optional datetime",
            _ => "json",
        }
    }
}

impl Default for RawBlogPost {
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
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, edgedb_derive::Queryable)]
pub struct BlogCategory {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
}

#[serde_with::apply(
    EDatetime => #[serde(serialize_with = "serialize_edge_datetime")],
    Option<EDatetime> => #[serde(serialize_with = "serialize_optional_edge_datetime")],
)]
#[derive(Debug, Serialize, edgedb_derive::Queryable)]
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

#[allow(dead_code)]
impl DetailedBlogPost {
    pub fn type_cast_for_field<'a>(name: &'a str) -> &'a str {
        match name {
            "title" => "str",
            "slug" => "str",
            "is_published" => "bool",
            "body" => "str",
            "format" => "DocFormat",
            "locale" => "str",
            "excerpt" => "str",
            "html" => "str",
            "seo_description" => "str",
            "og_image" => "str",
            _ => "str",
        }
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
            seo_description: None,
            og_image: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlogPost {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub is_published: Option<bool>,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub categories: Vec<BlogCategory>,
}

impl From<RawBlogPost> for BlogPost {
    fn from(post: RawBlogPost) -> Self {
        let published_at: Option<DateTime<Utc>> = post.published_at.map(|d| d.into());
        let created_at: DateTime<Utc> = post.created_at.into();
        let updated_at: Option<DateTime<Utc>> = post.updated_at.map(|d| d.into());
        BlogPost {
            id: post.id,
            title: post.title,
            slug: post.slug,
            is_published: post.is_published,
            published_at,
            created_at,
            updated_at,
            categories: post.categories,
        }
    }
}

impl FromIterator<RawBlogPost> for Vec<BlogPost> {
    fn from_iter<T: IntoIterator<Item = RawBlogPost>>(iter: T) -> Self {
        iter.into_iter().map(BlogPost::from).collect()
    }
}
