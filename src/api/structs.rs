use std::collections::HashMap;
use std::num::NonZeroU16;

use edgedb_protocol::named_args;
use edgedb_protocol::value::Value as EValue;
use edgedb_protocol::value_opt::ValueOpt;
use field_access::FieldAccess;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validify::Validify;

use super::macros::append_set_statement;
use crate::models::DocFormat;
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

    pub fn make_edgedb_args(
        &self,
        post_id: Uuid,
        submitted_fields: &Vec<&String>,
    ) -> HashMap<&str, ValueOpt> {
        let mut hm = named_args! {
            "id" => post_id
        };
        if submitted_fields.contains("title") {
            hm.insert("title", self.title.clone().into());
        }
        if submitted_fields.contains("slug") {
            hm.insert("slug", self.slug.clone().into());
        }
        if submitted_fields.contains("is_published") {
            hm.insert("is_published", self.is_published.into());
        }
        if submitted_fields.contains("format") {
            hm.insert("format", self.format.clone().into());
        }
        if submitted_fields.contains("body") {
            let html = self.body.as_ref().map(|b| markdown_to_html(b));
            let excerpt = self.body.as_ref().map(|b| make_excerpt(b));
            hm.insert("body", self.body.clone().into());
            hm.insert("html", html.into());
            hm.insert("excerpt", excerpt.into());
        }
        if submitted_fields.contains("locale") {
            hm.insert("locale", self.locale.clone().into());
        }
        if submitted_fields.contains("author") {
            hm.insert("author", self.author.into());
        }
        if submitted_fields.contains("og_image") {
            hm.insert("og_image", self.og_image.clone().into());
        }
        if let Some(categories) = &self.categories {
            let categories: Vec<EValue> = categories.iter().map(|&i| EValue::Uuid(i)).collect();
            hm.insert("categories", categories.into());
        }
        hm
    }
}

#[derive(Debug, Default, Deserialize, FieldAccess, Validify)]
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

    pub fn make_edgedb_args(&self, submitted_fields: &Vec<&String>) -> HashMap<&str, ValueOpt> {
        let mut hm = named_args! {
            "title" => self.title.clone(),
            "slug" => self.slug.clone()
        };
        if submitted_fields.contains("is_published") {
            hm.insert("is_published", self.is_published.into());
        }
        if submitted_fields.contains("body") {
            hm.insert("body", self.body.clone().into());
            let html = self.body.as_ref().map(|v| markdown_to_html(v));
            let excerpt = self.body.as_ref().map(|v| make_excerpt(v));
            hm.insert("html", html.into());
            hm.insert("excerpt", excerpt.into());
        }
        if submitted_fields.contains("format") {
            hm.insert("format", self.format.clone().into());
        }
        if submitted_fields.contains("locale") {
            hm.insert("locale", self.locale.clone().into());
        }
        if submitted_fields.contains("author") {
            hm.insert("author", self.author.into());
        }
        if submitted_fields.contains("og_image") {
            hm.insert("og_image", self.og_image.clone().into());
        }
        if let Some(categories) = &self.categories {
            let categories: Vec<EValue> = categories.iter().map(|&i| EValue::Uuid(i)).collect();
            hm.insert("categories", categories.into());
        }
        hm
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
    pub fn make_edgedb_args(
        &self,
        id: Uuid,
        submitted_fields: &Vec<&String>,
    ) -> HashMap<&str, ValueOpt> {
        let mut hm = named_args! {
            "id" => id
        };
        if submitted_fields.contains("title") {
            hm.insert("title", self.title.clone().into());
        }
        if submitted_fields.contains("slug") {
            hm.insert("slug", self.slug.clone().into());
        }
        if submitted_fields.contains("title_vi") {
            hm.insert("title_vi", self.title_vi.clone().into());
        }
        hm
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
    pub fn make_edgedb_args(&self) -> HashMap<&str, ValueOpt> {
        let hm = named_args! {
            "title" => self.title.clone(),
            "slug" => self.slug.clone(),
            "title_vi" => self.title_vi.clone()
        };
        hm
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
    pub fn make_edgedb_args(
        &self,
        id: Uuid,
        submitted_fields: &Vec<&String>,
    ) -> HashMap<&str, ValueOpt> {
        let mut hm = named_args! {
            "id"=> id
        };
        if submitted_fields.contains("title") {
            hm.insert("title", self.title.clone().into());
        }
        if submitted_fields.contains("url") {
            hm.insert("url", self.url.clone().into());
        }
        if submitted_fields.contains("event") {
            hm.insert("event", self.event.clone().into());
        }
        hm
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
    pub fn make_edgedb_args(&self) -> HashMap<&str, ValueOpt> {
        let hm = named_args! {
            "title" => self.title.clone(),
            "url" => self.url.clone(),
            "event" => self.event.clone()
        };
        hm
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

    pub fn make_edgedb_args(
        &self,
        id: Uuid,
        submitted_fields: &Vec<&String>,
    ) -> HashMap<&str, ValueOpt> {
        let mut hm = named_args! {
            "id" => id,
        };
        if submitted_fields.contains("title") {
            hm.insert("title", self.title.clone().into());
        }
        if submitted_fields.contains("download_url") {
            hm.insert("download_url", self.download_url.clone().into());
        }
        if submitted_fields.contains("author") {
            hm.insert("author", self.author.into());
        }
        hm
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

    pub fn make_edgedb_args(&self) -> HashMap<&str, ValueOpt> {
        let mut hm = named_args! {
            "title" => self.title.as_str(),
            "download_url" => self.download_url.as_str()
        };
        if let Some(a) = self.author {
            hm.insert("author", a.into());
        }
        hm
    }
}
