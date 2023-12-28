use std::num::NonZeroU16;

use edgedb_protocol::common::Cardinality as Cd;
use edgedb_protocol::value::Value as EValue;
use indexmap::indexmap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validify::Validify;

use super::macros::append_set_statement;
use crate::models::DocFormat;
use crate::types::conversions::{edge_object_from_pairs, edge_object_from_simple_pairs};
use crate::types::ext::VecExt;
use crate::utils::markdown::{make_excerpt, markdown_to_html};

#[derive(Debug, Deserialize)]
pub struct NPaging {
    pub page: Option<NonZeroU16>,
    pub per_page: Option<u8>,
}

#[derive(Debug, Deserialize, Validify)]
pub struct OtherQuery {
    #[modify(trim)]
    pub q: Option<String>,
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
    pub author: Option<Uuid>,
    pub og_image: Option<String>,
}

impl BlogPostPatchData {
    pub fn gen_set_clause(&self, submitted_fields: &Vec<&String>) -> String {
        let mut lines = Vec::<&str>::new();
        append_set_statement!("title", "optional str", lines, submitted_fields);
        append_set_statement!("slug", "optional str", lines, submitted_fields);
        append_set_statement!("is_published", "optional bool", lines, submitted_fields);
        append_set_statement!("format", "optional DocFormat", lines, submitted_fields);
        if submitted_fields.contains("body") {
            // If user submitted "body" field, we will generate "html", "excerpt" and write, too
            lines.push("body := <optional str>$body");
            lines.push("html := <optional str>$html");
            lines.push("excerpt := <optional str>$excerpt");
        }
        append_set_statement!("locale", "optional str", lines, submitted_fields);
        append_set_statement!("author", "optional User", lines, submitted_fields);
        append_set_statement!("og_image", "optional str", lines, submitted_fields);
        if submitted_fields.contains("categories") && self.categories.is_some() {
            let line = "categories := (
                SELECT BlogCategory FILTER .id IN array_unpack(<array<uuid>>$categories)
            )";
            lines.push(line);
        }
        lines.join(&format!(",\n{}", " ".repeat(8)))
    }

    pub fn make_edgedb_object(&self, post_id: Uuid, submitted_fields: &Vec<&String>) -> EValue {
        let mut pairs = indexmap! {
            "id" => (Some(EValue::Uuid(post_id)), Cd::One),
        };
        if submitted_fields.contains("title") {
            pairs.insert(
                "title",
                (self.title.clone().map(EValue::Str), Cd::AtMostOne),
            );
        }
        if submitted_fields.contains("slug") {
            pairs.insert("slug", (self.slug.clone().map(EValue::Str), Cd::AtMostOne));
        }
        if submitted_fields.contains("is_published") {
            pairs.insert(
                "is_published",
                (self.is_published.map(EValue::Bool), Cd::AtMostOne),
            );
        }
        if submitted_fields.contains("format") {
            pairs.insert(
                "format",
                (self.format.clone().map(EValue::from), Cd::AtMostOne),
            );
        }
        if submitted_fields.contains("body") {
            let body = self.body.clone();
            let html = body.as_ref().map(|b| markdown_to_html(b));
            let excerpt = body.as_ref().map(|b| make_excerpt(b));
            pairs.insert("body", (body.map(EValue::Str), Cd::AtMostOne));
            pairs.insert("html", (html.map(EValue::Str), Cd::AtMostOne));
            pairs.insert("excerpt", (excerpt.map(EValue::Str), Cd::AtMostOne));
        }
        if submitted_fields.contains("locale") {
            pairs.insert(
                "locale",
                (self.locale.clone().map(EValue::Str), Cd::AtMostOne),
            );
        }
        if submitted_fields.contains("author") {
            pairs.insert("author", (self.author.map(EValue::Uuid), Cd::AtMostOne));
        }
        if submitted_fields.contains("og_image") {
            pairs.insert(
                "og_image",
                (self.og_image.clone().map(EValue::Str), Cd::AtMostOne),
            );
        }
        if let Some(categories) = &self.categories {
            let categories: Vec<EValue> = categories.iter().map(|&i| EValue::Uuid(i)).collect();
            pairs.insert("categories", (Some(EValue::Array(categories)), Cd::One));
        }
        edge_object_from_pairs(pairs)
    }
}

#[derive(Debug, Default, Deserialize, Validify)]
pub struct BlogPostCreateData {
    #[validate(length(min = 2))]
    pub title: String,
    #[validate(length(min = 2))]
    pub slug: String,
    pub is_published: Option<bool>,
    pub format: Option<DocFormat>,
    pub body: Option<String>,
    pub locale: Option<String>,
    pub categories: Option<Vec<Uuid>>,
    pub author: Option<Uuid>,
    #[validate(url)]
    pub og_image: Option<String>,
}

impl BlogPostCreateData {
    pub fn gen_set_clause(&self, submitted_fields: &Vec<&String>) -> String {
        let mut lines = vec!["title := <str>$title", "slug := <str>$slug"];
        append_set_statement!("is_published", "optional bool", lines, submitted_fields);
        if submitted_fields.contains("body") {
            // If user submitted "body" field, we will generate "html", "excerpt" and write, too
            lines.push("body := <optional str>$body");
            lines.push("html := <optional str>$html");
            lines.push("excerpt := <optional str>$excerpt");
        }
        append_set_statement!("format", "optional DocFormat", lines, submitted_fields);
        append_set_statement!("locale", "optional str", lines, submitted_fields);
        append_set_statement!("author", "optional User", lines, submitted_fields);
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

    pub fn make_edgedb_object(&self, submitted_fields: &Vec<&String>) -> EValue {
        let mut pairs = indexmap! {
            "title" => (Some(EValue::Str(self.title.clone())), Cd::One),
            "slug" => (Some(EValue::Str(self.slug.clone())), Cd::One),
        };
        if submitted_fields.contains("is_published") {
            pairs.insert(
                "is_published",
                (self.is_published.map(EValue::Bool), Cd::AtMostOne),
            );
        }
        if submitted_fields.contains("body") {
            let body = self.body.clone();
            let html = body.as_ref().map(|v| markdown_to_html(v));
            let excerpt = body.as_ref().map(|v| make_excerpt(v));
            pairs.insert("body", (body.map(EValue::Str), Cd::AtMostOne));
            pairs.insert("html", (html.map(EValue::Str), Cd::AtMostOne));
            pairs.insert("excerpt", (excerpt.map(EValue::Str), Cd::AtMostOne));
        }
        if submitted_fields.contains("format") {
            pairs.insert(
                "format",
                (self.format.clone().map(EValue::from), Cd::AtMostOne),
            );
        }
        if submitted_fields.contains("locale") {
            pairs.insert(
                "locale",
                (self.locale.clone().map(EValue::Str), Cd::AtMostOne),
            );
        }
        if submitted_fields.contains("author") {
            pairs.insert("author", (self.author.map(EValue::Uuid), Cd::AtMostOne));
        }
        if submitted_fields.contains("og_image") {
            pairs.insert(
                "og_image",
                (self.og_image.clone().map(EValue::Str), Cd::AtMostOne),
            );
        }
        if let Some(categories) = &self.categories {
            let categories: Vec<EValue> = categories.iter().map(|&i| EValue::Uuid(i)).collect();
            pairs.insert("categories", (Some(EValue::Array(categories)), Cd::One));
        }
        edge_object_from_pairs(pairs)
    }
}

#[derive(Debug, Deserialize)]
pub struct BlogCategoryPatchData {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub title_vi: Option<String>,
}

impl BlogCategoryPatchData {
    pub fn gen_set_clause(&self, submitted_fields: &[&String]) -> String {
        let mut lines = Vec::<&str>::new();
        append_set_statement!("title", "optional str", lines, submitted_fields);
        append_set_statement!("slug", "optional str", lines, submitted_fields);
        append_set_statement!("title_vi", "optional str", lines, submitted_fields);
        let sep = format!(",\n{}", " ".repeat(12));
        lines.join(&sep)
    }

    pub fn make_edgedb_object(&self, id: Uuid, submitted_fields: &Vec<&String>) -> EValue {
        let mut pairs = indexmap!(
            "id" => (Some(EValue::Uuid(id)), Cd::One),
        );
        if submitted_fields.contains("title") {
            pairs.insert(
                "title",
                (self.title.clone().map(EValue::Str), Cd::AtMostOne),
            );
        }
        if submitted_fields.contains("slug") {
            pairs.insert("slug", (self.slug.clone().map(EValue::Str), Cd::AtMostOne));
        }
        if submitted_fields.contains("title_vi") {
            pairs.insert(
                "title_vi",
                (self.title_vi.clone().map(EValue::Str), Cd::AtMostOne),
            );
        }
        edge_object_from_pairs(pairs)
    }
}

#[derive(Debug, Default, Deserialize, Validify)]
pub struct BlogCategoryCreateData {
    #[validate(length(min = 2))]
    pub title: String,
    #[validate(length(min = 2))]
    pub slug: String,
    #[validate(length(min = 2))]
    pub title_vi: String,
}

impl BlogCategoryCreateData {
    pub fn gen_set_clause(&self) -> String {
        let lines = [
            "title := <str>$title",
            "slug := <str>$slug",
            "title_vi :=<str>$title_vi",
        ];
        let sep = format!(",\n{}", " ".repeat(12));
        lines.join(&sep)
    }

    pub fn make_edgedb_object(&self) -> EValue {
        let pairs = indexmap! {
            "title" => Some(EValue::from(self.title.clone())),
            "slug" => Some(EValue::from(self.slug.clone())),
            "title_vi" => Some(EValue::from(self.title_vi.clone())),
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
    pub fn gen_set_clause(&self, submitted_fields: &[&String]) -> String {
        let mut lines = Vec::<&str>::new();
        append_set_statement!("title", "optional str", lines, submitted_fields);
        append_set_statement!("url", "optional str", lines, submitted_fields);
        append_set_statement!("event", "optional str", lines, submitted_fields);
        let sep = format!(",\n{}", " ".repeat(12));
        lines.join(&sep)
    }

    pub fn make_edgedb_object(&self, id: Uuid, submitted_fields: &Vec<&String>) -> EValue {
        let mut pairs = indexmap!(
            "id" => (Some(EValue::Uuid(id)), Cd::One),
        );
        if submitted_fields.contains("title") {
            pairs.insert(
                "title",
                (self.title.clone().map(EValue::Str), Cd::AtMostOne),
            );
        }
        if submitted_fields.contains("url") {
            pairs.insert("url", (self.url.clone().map(EValue::Str), Cd::AtMostOne));
        }
        if submitted_fields.contains("event") {
            pairs.insert(
                "event",
                (self.event.clone().map(EValue::Str), Cd::AtMostOne),
            );
        }
        edge_object_from_pairs(pairs)
    }
}

#[derive(Debug, Default, Deserialize, Validify)]
pub struct PresentationCreateData {
    #[validate(length(min = 2))]
    pub title: String,
    #[validate(url)]
    pub url: String,
    #[validate(length(min = 2))]
    pub event: Option<String>,
}

impl PresentationCreateData {
    pub fn gen_set_clause(&self) -> String {
        let lines = [
            "title := <str>$title",
            "url := <str>$url",
            "event := <optional str>$event",
        ];
        let sep = format!(",\n{}", " ".repeat(12));
        lines.join(&sep)
    }

    pub fn make_edgedb_object(&self) -> EValue {
        let pairs = indexmap! {
            "title" => (Some(EValue::from(self.title.clone())), Cd::One),
            "url" => (Some(EValue::from(self.url.clone())), Cd::One),
            "event" => (self.event.clone().map(EValue::from), Cd::AtMostOne),
        };
        edge_object_from_pairs(pairs)
    }
}

#[derive(Debug, Default, Deserialize, Validify)]
pub struct BookAuthorPatchData {
    #[validate(length(min = 2))]
    pub name: String,
}

#[derive(Debug, Deserialize, Validify)]
pub struct BookPatchData {
    #[validate(length(min = 2))]
    pub title: Option<String>,
    #[validate(url)]
    pub download_url: Option<String>,
    pub author: Option<Uuid>,
}

impl BookPatchData {
    pub fn gen_set_clause(&self, submitted_fields: &Vec<&String>) -> String {
        let mut lines = Vec::<&str>::new();
        append_set_statement!("title", "optional str", lines, submitted_fields);
        append_set_statement!("download_url", "optional str", lines, submitted_fields);
        if submitted_fields.contains("author") {
            let line = "author := (
                SELECT BookAuthor FILTER .id = <uuid>$author
            )";
            lines.push(line);
        }
        let sep = format!(",\n{}", " ".repeat(12));
        lines.join(&sep)
    }

    pub fn make_edgedb_object(&self, id: Uuid, submitted_fields: &Vec<&String>) -> EValue {
        let mut pairs = indexmap!(
            "id" => (Some(EValue::Uuid(id)), Cd::One),
        );
        if submitted_fields.contains("title") {
            pairs.insert(
                "title",
                (self.title.clone().map(EValue::Str), Cd::AtMostOne),
            );
        }
        if submitted_fields.contains("download_url") {
            pairs.insert(
                "download_url",
                (self.download_url.clone().map(EValue::Str), Cd::AtMostOne),
            );
        }
        if submitted_fields.contains("author") {
            pairs.insert("author", (self.author.map(EValue::Uuid), Cd::One));
        }
        edge_object_from_pairs(pairs)
    }
}

#[derive(Debug, Deserialize, Validify)]
pub struct BookCreateData {
    #[validate(length(min = 2))]
    pub title: String,
    #[validate(url)]
    pub download_url: String,
    pub author: Option<Uuid>,
}

impl BookCreateData {
    pub fn gen_set_clause(&self) -> String {
        let mut lines = vec!["title := <str>$title", "download_url := <str>$download_url"];
        if self.author.is_some() {
            let line = "author := (
                SELECT BookAuthor FILTER .id = <uuid>$author
            )";
            lines.push(line);
        }
        let sep = format!(",\n{}", " ".repeat(12));
        lines.join(&sep)
    }

    pub fn make_edgedb_object(&self) -> EValue {
        let mut pairs = indexmap! {
            "title" => (Some(EValue::from(self.title.clone())), Cd::One),
            "download_url" => (Some(EValue::from(self.download_url.clone())), Cd::One),
        };
        if self.author.is_some() {
            pairs.insert("author", (self.author.map(EValue::Uuid), Cd::One));
        }
        edge_object_from_pairs(pairs)
    }
}
