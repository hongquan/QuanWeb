use std::num::NonZeroU16;

use edgedb_protocol::common::Cardinality as Cd;
use edgedb_protocol::value::Value as EValue;
use garde::Validate;
use indexmap::indexmap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::macros::append_set_statement;
use crate::models::DocFormat;
use crate::types::conversions::{edge_object_from_pairs, edge_object_from_simple_pairs};
use crate::utils::markdown::{make_excerpt, markdown_to_html};

#[derive(Deserialize, Debug)]
pub struct Paging {
    pub page: Option<u16>,
    pub per_page: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct NPaging {
    pub page: Option<NonZeroU16>,
    pub per_page: Option<u8>,
}

impl From<NPaging> for Paging {
    fn from(npaging: NPaging) -> Self {
        Self {
            page: npaging.page.map(|i| i.get()),
            per_page: npaging.per_page,
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct PaginationLinks {
    pub prev: Option<String>,
    pub next: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ObjectListResponse<T> {
    pub count: usize,
    pub total_pages: NonZeroU16,
    pub links: PaginationLinks,
    pub objects: Vec<T>,
}

impl<T> Default for ObjectListResponse<T> {
    fn default() -> Self {
        Self {
            count: 0,
            total_pages: NonZeroU16::MIN,
            links: Default::default(),
            objects: vec![],
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BlogPostPatchData {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub is_published: Option<bool>,
    pub format: Option<DocFormat>,
    pub body: Option<String>,
    pub locale: Option<String>,
    pub categories: Option<Vec<Uuid>>,
    pub og_image: Option<String>,
}

impl BlogPostPatchData {
    pub fn gen_set_clause<'a>(&self, submitted_fields: &Vec<&String>) -> String {
        let mut lines = Vec::<&str>::new();
        append_set_statement!("title", "optional str", lines, submitted_fields);
        append_set_statement!("slug", "optional str", lines, submitted_fields);
        append_set_statement!("is_published", "optional bool", lines, submitted_fields);
        append_set_statement!("format", "optional DocFormat", lines, submitted_fields);
        if submitted_fields.iter().any(|&f| f == "body") {
            // If user submitted "body" field, we will generate "html", "excerpt" and write, too
            lines.push("body := <optional str>$body");
            lines.push("html := <optional str>$html");
            lines.push("excerpt := <optional str>$excerpt");
        }
        append_set_statement!("locale", "optional str", lines, submitted_fields);
        append_set_statement!("og_image", "optional str", lines, submitted_fields);
        if submitted_fields.iter().any(|&f| f == "categories") && self.categories.is_some() {
            let line = "categories := (
                SELECT BlogCategory FILTER .id IN array_unpack(<array<uuid>>$categories)
            )";
            lines.push(line);
        }
        lines.join(&format!(",\n{}", " ".repeat(8)))
    }

    pub fn make_edgedb_object<'a>(&self, post_id: Uuid, submitted_fields: &Vec<&String>) -> EValue {
        let mut pairs = indexmap! {
            "id" => (Some(EValue::Uuid(post_id)), Cd::One),
        };
        if submitted_fields.iter().any(|&f| f == "title") {
            pairs.insert(
                "title",
                (self.title.clone().map(EValue::Str), Cd::AtMostOne),
            );
        }
        if submitted_fields.iter().any(|&f| f == "slug") {
            pairs.insert(
                "slug",
                (self.slug.clone().map(EValue::Str), Cd::AtMostOne),
            );
        }
        if submitted_fields.iter().any(|&f| f == "is_published") {
            pairs.insert(
                "is_published",
                (self.is_published.map(EValue::Bool), Cd::AtMostOne),
            );
        }
        if submitted_fields.iter().any(|&f| f == "format") {
            pairs.insert(
                "format",
                (
                    self.format.clone().map(EValue::from),
                    Cd::AtMostOne,
                ),
            );
        }
        if submitted_fields.iter().any(|&f| f == "body") {
            let body = self.body.clone();
            let html = body.as_ref().map(|b| markdown_to_html(b));
            let excerpt = body.as_ref().map(|b| make_excerpt(b));
            pairs.insert("body", (body.map(EValue::Str), Cd::AtMostOne));
            pairs.insert("html", (html.map(EValue::Str), Cd::AtMostOne));
            pairs.insert(
                "excerpt",
                (excerpt.map(EValue::Str), Cd::AtMostOne),
            );
        }
        if submitted_fields.iter().any(|&f| f == "locale") {
            pairs.insert(
                "locale",
                (self.locale.clone().map(EValue::Str), Cd::AtMostOne),
            );
        }
        if submitted_fields.iter().any(|&f| f == "og_image") {
            pairs.insert(
                "og_image",
                (self.og_image.clone().map(EValue::Str), Cd::AtMostOne),
            );
        }
        if let Some(categories) = &self.categories {
            let categories: Vec<EValue> = categories.iter().map(|&i| EValue::Uuid(i)).collect();
            pairs.insert(
                "categories",
                (Some(EValue::Array(categories)), Cd::One),
            );
        }
        edge_object_from_pairs(pairs)
    }
}

#[derive(Debug, Default, Deserialize, Validate)]
pub struct BlogPostCreateData {
    #[garde(length(min = 1))]
    pub title: String,
    #[garde(length(min = 1))]
    pub slug: String,
    #[garde(skip)]
    pub is_published: Option<bool>,
    #[garde(skip)]
    pub format: Option<DocFormat>,
    #[garde(skip)]
    pub body: Option<String>,
    #[garde(skip)]
    pub locale: Option<String>,
    #[garde(skip)]
    pub categories: Option<Vec<Uuid>>,
    #[garde(skip)]
    pub og_image: Option<String>,
}

impl BlogPostCreateData {
    pub fn gen_set_clause<'a>(&self, submitted_fields: &Vec<&String>) -> String {
        let mut lines = vec!["title := <str>$title", "slug := <str>$slug"];
        append_set_statement!("is_published", "optional bool", lines, submitted_fields);
        if submitted_fields.iter().any(|&f| f == "body") {
            // If user submitted "body" field, we will generate "html", "excerpt" and write, too
            lines.push("body := <optional str>$body");
            lines.push("html := <optional str>$html");
            lines.push("excerpt := <optional str>$excerpt");
        }
        append_set_statement!("format", "optional DocFormat", lines, submitted_fields);
        append_set_statement!("locale", "optional str", lines, submitted_fields);
        append_set_statement!("og_image", "optional str", lines, submitted_fields);
        if self.categories.is_some() {
            let line = "categories := (
                SELECT BlogCategory FILTER .id IN array_unpack(<array<uuid>>$categories)
            )";
            lines.push(line);
        }
        let sep = format!(",\n{}", " ".repeat(12));
        lines.join(&sep)
    }

    pub fn make_edgedb_object<'a>(&self, submitted_fields: &Vec<&String>) -> EValue {
        let mut pairs = indexmap! {
            "title" => (Some(EValue::Str(self.title.clone())), Cd::One),
            "slug" => (Some(EValue::Str(self.slug.clone())), Cd::One),
        };
        if submitted_fields.iter().any(|&f| f == "is_published") {
            pairs.insert(
                "is_published",
                (self.is_published.map(EValue::Bool), Cd::AtMostOne),
            );
        }
        if submitted_fields.iter().any(|&f| f == "body") {
            let body = self.body.clone();
            let html = body.as_ref().map(|v| markdown_to_html(v));
            let excerpt = body.as_ref().map(|v| make_excerpt(v));
            pairs.insert("body", (body.map(EValue::Str), Cd::AtMostOne));
            pairs.insert("html", (html.map(EValue::Str), Cd::AtMostOne));
            pairs.insert(
                "excerpt",
                (excerpt.map(EValue::Str), Cd::AtMostOne),
            );
        }
        if submitted_fields.iter().any(|&f| f == "format") {
            pairs.insert(
                "format",
                (
                    self.format.clone().map(EValue::from),
                    Cd::AtMostOne,
                ),
            );
        }
        if submitted_fields.iter().any(|&f| f == "locale") {
            pairs.insert(
                "locale",
                (self.locale.clone().map(EValue::Str), Cd::AtMostOne),
            );
        }
        if submitted_fields.iter().any(|&f| f == "og_image") {
            pairs.insert(
                "og_image",
                (self.og_image.clone().map(EValue::Str), Cd::AtMostOne),
            );
        }
        if let Some(categories) = &self.categories {
            let categories: Vec<EValue> = categories.iter().map(|&i| EValue::Uuid(i)).collect();
            pairs.insert(
                "categories",
                (Some(EValue::Array(categories)), Cd::One),
            );
        }
        edge_object_from_pairs(pairs)
    }
}

#[derive(Debug, Deserialize)]
pub struct BlogCategoryPatchData {
    pub title: Option<String>,
    pub slug: Option<String>,
}

impl BlogCategoryPatchData {
    pub fn gen_set_clause<'a>(&self, submitted_fields: &Vec<&String>) -> String {
        let mut lines = Vec::<&str>::new();
        append_set_statement!("title", "optional str", lines, submitted_fields);
        append_set_statement!("slug", "optional str", lines, submitted_fields);
        let sep = format!(",\n{}", " ".repeat(12));
        lines.join(&sep)
    }

    pub fn make_edgedb_object<'a>(&self, id: Uuid, submitted_fields: &Vec<&String>) -> EValue {
        let mut pairs = indexmap!(
            "id" => (Some(EValue::Uuid(id)), Cd::One),
        );
        if submitted_fields.iter().any(|&f| f == "title") {
            pairs.insert(
                "title",
                (self.title.clone().map(EValue::Str), Cd::AtMostOne),
            );
        }
        if submitted_fields.iter().any(|&f| f == "slug") {
            pairs.insert(
                "slug",
                (self.slug.clone().map(EValue::Str), Cd::AtMostOne),
            );
        }
        edge_object_from_pairs(pairs)
    }
}

#[derive(Debug, Default, Deserialize, Validate)]
pub struct BlogCategoryCreateData {
    #[garde(length(min = 1))]
    pub title: String,
    #[garde(length(min = 1))]
    pub slug: String,
}

impl BlogCategoryCreateData {
    pub fn gen_set_clause<'a>(&self) -> String {
        let lines = vec!["title := <str>$title", "slug := <str>$slug"];
        let sep = format!(",\n{}", " ".repeat(12));
        lines.join(&sep)
    }

    pub fn make_edgedb_object<'a>(&self) -> EValue {
        let pairs = indexmap! {
            "title" => Some(EValue::from(self.title.clone())),
            "slug" => Some(EValue::from(self.slug.clone())),
        };
        edge_object_from_simple_pairs(pairs)
    }
}

#[derive(Debug, Deserialize)]
pub struct PresentationPatchData {
    pub title: Option<String>,
    pub url: Option<String>,
    pub event: Option<String>,
}

impl PresentationPatchData {
    pub fn gen_set_clause<'a>(&self, submitted_fields: &Vec<&String>) -> String {
        let mut lines = Vec::<&str>::new();
        append_set_statement!("title", "optional str", lines, submitted_fields);
        append_set_statement!("url", "optional str", lines, submitted_fields);
        append_set_statement!("event", "optional str", lines, submitted_fields);
        let sep = format!(",\n{}", " ".repeat(12));
        lines.join(&sep)
    }

    pub fn make_edgedb_object<'a>(&self, id: Uuid, submitted_fields: &Vec<&String>) -> EValue {
        let mut pairs = indexmap!(
            "id" => (Some(EValue::Uuid(id)), Cd::One),
        );
        if submitted_fields.iter().any(|&f| f == "title") {
            pairs.insert(
                "title",
                (self.title.clone().map(EValue::Str), Cd::AtMostOne),
            );
        }
        if submitted_fields.iter().any(|&f| f == "url") {
            pairs.insert(
                "url",
                (self.url.clone().map(EValue::Str), Cd::AtMostOne),
            );
        }
        if submitted_fields.iter().any(|&f| f == "event") {
            pairs.insert(
                "event",
                (self.event.clone().map(EValue::Str), Cd::AtMostOne),
            );
        }
        edge_object_from_pairs(pairs)
    }
}
