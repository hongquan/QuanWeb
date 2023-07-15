use std::str::FromStr;

use chrono::Utc;
use edgedb_derive::Queryable;
use edgedb_protocol::model::Datetime as EDatetime;
use edgedb_protocol::value::Value as EValue;
use minijinja::value::{StructObject, Value as MJValue};
use serde::{Deserialize, Serialize};
use serde_json::Value as JValue;
use strum_macros::{Display, EnumString, IntoStaticStr};
use uuid::Uuid;

use crate::types::conversions::{
    edge_datetime_to_jinja, serialize_edge_datetime, serialize_optional_edge_datetime,
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
#[derive(Debug, Serialize, Queryable)]
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
    pub fn type_cast_for_field<'a>(name: &'a str) -> &'a str {
        match name {
            "title" => "str",
            "slug" => "str",
            "excerpt" => "optional str",
            "is_published" => "bool",
            "published_at" => "optional datetime",
            "updated_at" => "optional datetime",
            _ => "json",
        }
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

impl StructObject for MediumBlogPost {
    fn get_field(&self, name: &str) -> Option<MJValue> {
        match name {
            "id" => Some(MJValue::from(self.id.to_string())),
            "title" => Some(MJValue::from(self.title.as_str())),
            "slug" => Some(MJValue::from(self.slug.as_str())),
            "excerpt" => self.excerpt.clone().map(MJValue::from),
            "is_published" => self.is_published.map(MJValue::from),
            "published_at" => self.published_at.map(edge_datetime_to_jinja),
            "created_at" => Some(edge_datetime_to_jinja(self.created_at)),
            "updated_at" => self.updated_at.map(edge_datetime_to_jinja),
            "categories" => Some(self.categories.clone().into_iter().collect()),
            _ => None,
        }
    }
    fn static_fields(&self) -> Option<&'static [&'static str]> {
        Some(
            &[
                "id",
                "title",
                "slug",
                "excerpt",
                "is_published",
                "published_at",
                "created_at",
                "updated_at",
                "categories",
            ][..],
        )
    }

    fn field_count(&self) -> usize {
        9
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Queryable)]
pub struct BlogCategory {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
}

impl StructObject for BlogCategory {
    fn get_field(&self, name: &str) -> Option<MJValue> {
        match name {
            "id" => Some(MJValue::from(self.id.to_string())),
            "title" => Some(MJValue::from(self.title.as_str())),
            "slug" => Some(MJValue::from(self.slug.as_str())),
            _ => None,
        }
    }
    fn static_fields(&self) -> Option<&'static [&'static str]> {
        Some(&["id", "title", "slug"][..])
    }

    fn field_count(&self) -> usize {
        3
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

impl StructObject for DetailedBlogPost {
    fn get_field(&self, name: &str) -> Option<MJValue> {
        match name {
            "id" => Some(MJValue::from(self.id.to_string())),
            "title" => Some(MJValue::from(self.title.as_str())),
            "slug" => Some(MJValue::from(self.slug.as_str())),
            "is_published" => self.is_published.map(MJValue::from),
            "published_at" => self.published_at.map(edge_datetime_to_jinja),
            "created_at" => Some(edge_datetime_to_jinja(self.created_at)),
            "updated_at" => self.updated_at.map(edge_datetime_to_jinja),
            "categories" => Some(self.categories.clone().into_iter().collect()),
            "body" => self.body.clone().map(MJValue::from),
            "format" => Some(MJValue::from(self.format.to_string())),
            "locale" => self.locale.clone().map(MJValue::from),
            "excerpt" => self.excerpt.clone().map(MJValue::from),
            "html" => self.html.clone().map(MJValue::from),
            "seo_description" => self.seo_description.clone().map(MJValue::from),
            "og_image" => self.og_image.clone().map(MJValue::from),
            _ => None,
        }
    }
    fn static_fields(&self) -> Option<&'static [&'static str]> {
        Some(
            &[
                "id",
                "title",
                "slug",
                "is_published",
                "published_at",
                "created_at",
                "updated_at",
                "categories",
                "body",
                "format",
                "locale",
                "excerpt",
                "html",
                "seo_description",
                "og_image",
            ][..],
        )
    }

    fn field_count(&self) -> usize {
        15
    }
}

impl From<MediumBlogPost> for MJValue {
    fn from(value: MediumBlogPost) -> Self {
        MJValue::from_struct_object(value)
    }
}

impl FromIterator<MediumBlogPost> for Vec<MJValue> {
    fn from_iter<T: IntoIterator<Item = MediumBlogPost>>(iter: T) -> Self {
        iter.into_iter().map(MJValue::from).collect()
    }
}

impl From<BlogCategory> for MJValue {
    fn from(value: BlogCategory) -> Self {
        MJValue::from_struct_object(value)
    }
}

impl FromIterator<BlogCategory> for Vec<MJValue> {
    fn from_iter<T: IntoIterator<Item = BlogCategory>>(iter: T) -> Self {
        iter.into_iter().map(MJValue::from).collect()
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
}
