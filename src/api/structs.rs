use std::num::NonZeroU16;

use edgedb_protocol::codec::ObjectShape;
use edgedb_protocol::common::Cardinality;
use edgedb_protocol::value::Value as EValue;
use fievar::Fields;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use garde::Validate;

use crate::models::DocFormat;
use crate::types::create_shape_element;
use crate::utils::markdown::{make_excerpt, markdown_to_html};
use super::macros::{append_field_general, append_field, append_set_statement};

#[derive(Deserialize, Debug)]
pub struct Paging {
    pub page: Option<u16>,
    pub per_page: Option<u8>,
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

#[derive(Debug, Deserialize, Fields)]
pub struct BlogPostPatchData {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub is_published: Option<bool>,
    pub format: Option<DocFormat>,
    pub body: Option<String>,
    pub categories: Option<Vec<Uuid>>,
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
        if submitted_fields.iter().any(|&f| f == "categories") && self.categories.is_some() {
            let line = "categories := (
                SELECT BlogCategory FILTER .id IN array_unpack(<array<uuid>>$categories)
            )";
            lines.push(line);
        }
        lines.join(&format!(",\n{}", " ".repeat(8)))
    }

    pub fn make_edgedb_object<'a>(&self, post_id: Uuid, submitted_fields: &Vec<&String>) -> EValue {
        let mut object_values = vec![Some(EValue::Uuid(post_id))];
        let mut elms = vec![create_shape_element("id", Cardinality::One)];

        append_field_general!("title", Cardinality::AtMostOne, elms, object_values, self.title, submitted_fields);
        append_field_general!("slug", Cardinality::AtMostOne, elms, object_values, self.slug, submitted_fields);
        append_field!("is_published", EValue::Bool, Cardinality::AtMostOne, elms, object_values, self.is_published, submitted_fields);

        if submitted_fields.iter().any(|&f| f == "body") {
            object_values.push(self.body.clone().map(EValue::Str));
            elms.push(create_shape_element("body", Cardinality::AtMostOne));
            let html = markdown_to_html(self.body.as_ref().unwrap_or(&"".to_string()));
            object_values.push(Some(EValue::Str(html)));
            elms.push(create_shape_element("html", Cardinality::AtMostOne));
            let excerpt = make_excerpt(self.body.as_ref().unwrap_or(&"".to_string()));
            object_values.push(Some(EValue::Str(excerpt)));
            elms.push(create_shape_element("excerpt", Cardinality::AtMostOne));
        }
        append_field_general!("format", Cardinality::AtMostOne, elms, object_values, self.format, submitted_fields);
        if submitted_fields.iter().any(|&f| f == "categories") {
            if let Some(categories) = &self.categories {
                object_values.push(Some(EValue::Array(categories.iter().map(|&c| EValue::Uuid(c)).collect())));
                elms.push(create_shape_element("categories", Cardinality::One));
            }
        }
        EValue::Object {
            shape: ObjectShape::new(elms),
            fields: object_values,
        }
    }
}

#[derive(Debug, Default, Deserialize, Fields, Validate)]
pub struct BlogPostCreateData {
    #[garde(length(min=1))]
    pub title: String,
    #[garde(length(min=1))]
    pub slug: String,
    #[garde(skip)]
    pub is_published: Option<bool>,
    #[garde(skip)]
    pub format: Option<DocFormat>,
    #[garde(skip)]
    pub body: Option<String>,
    #[garde(skip)]
    pub categories: Option<Vec<Uuid>>,
}

impl BlogPostCreateData {
    pub fn gen_set_clause<'a>(&self, submitted_fields: &Vec<&String>) -> String {
        let mut lines = vec![
            "title := <str>$title",
            "slug := <str>$slug",
        ];
        append_set_statement!("is_published", "optional bool", lines, submitted_fields);
        if submitted_fields.iter().any(|&f| f == "body") {
            // If user submitted "body" field, we will generate "html", "excerpt" and write, too
            lines.push("body := <optional str>$body");
            lines.push("html := <optional str>$html");
            lines.push("excerpt := <optional str>$excerpt");
        }
        append_set_statement!("format", "optional DocFormat", lines, submitted_fields);
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
        let mut object_values = vec![
            Some(EValue::from(self.title.clone())),
            Some(EValue::from(self.slug.clone())),
        ];
        let mut elms = vec![
            create_shape_element("title", Cardinality::One),
            create_shape_element("slug", Cardinality::One),
        ];
        append_field!("is_published", EValue::Bool, Cardinality::AtMostOne, elms, object_values, self.is_published, submitted_fields);
        if submitted_fields.iter().any(|&f| f == "body") {
            object_values.push(self.body.clone().map(EValue::Str));
            elms.push(create_shape_element("body", Cardinality::AtMostOne));
            let html = markdown_to_html(self.body.as_ref().unwrap_or(&"".to_string()));
            object_values.push(Some(EValue::Str(html)));
            elms.push(create_shape_element("html", Cardinality::AtMostOne));
            let excerpt = make_excerpt(self.body.as_ref().unwrap_or(&"".to_string()));
            object_values.push(Some(EValue::Str(excerpt)));
            elms.push(create_shape_element("excerpt", Cardinality::AtMostOne));
        }
        append_field_general!("format", Cardinality::AtMostOne, elms, object_values, self.format, submitted_fields);

        if let Some(categories) = &self.categories {
            let categories: Vec<EValue> = categories.iter().map(|&i| EValue::Uuid(i)).collect();
            let elm = create_shape_element("categories", Cardinality::One);
            elms.push(elm);
            object_values.push(Some(EValue::Array(categories)));
        }
        EValue::Object {
            shape: ObjectShape::new(elms),
            fields: object_values,
        }
    }
}

#[derive(Debug, Deserialize, Fields)]
pub struct BlogCategoryPatchData {
    pub title: Option<String>,
    pub slug: Option<String>,
}

#[allow(dead_code)]
impl BlogCategoryPatchData {
    pub fn gen_set_clause<'a>(&self, submitted_fields: &Vec<&String>) -> String {
        let mut lines = Vec::<&str>::new();
        append_set_statement!("title", "optional str", lines, submitted_fields);
        append_set_statement!("slug", "optional str", lines, submitted_fields);
        let sep = format!(",\n{}", " ".repeat(12));
        lines.join(&sep)
    }

    pub fn make_edgedb_object<'a>(&self, id: Uuid, submitted_fields: &Vec<&String>) -> EValue {
        let mut object_values = vec![Some(EValue::Uuid(id))];
        let mut elms = vec![create_shape_element("id", Cardinality::One)];

        append_field_general!("title", Cardinality::AtMostOne, elms, object_values, self.title, submitted_fields);
        append_field_general!("slug", Cardinality::AtMostOne, elms, object_values, self.slug, submitted_fields);
        EValue::Object {
            shape: ObjectShape::new(elms),
            fields: object_values,
        }
    }
}

#[derive(Debug, Default, Deserialize, Validate)]
pub struct BlogCategoryCreateData {
    #[garde(length(min=1))]
    pub title: String,
    #[garde(length(min=1))]
    pub slug: String,
}

#[allow(dead_code)]
impl BlogCategoryCreateData {
    pub fn gen_set_clause<'a>(&self) -> String {
        let lines = vec![
            "title := <str>$title",
            "slug := <str>$slug",
        ];
        let sep = format!(",\n{}", " ".repeat(12));
        lines.join(&sep)
    }

    pub fn make_edgedb_object<'a>(&self) -> EValue {
        let object_values = vec![
            Some(EValue::from(self.title.clone())),
            Some(EValue::from(self.slug.clone())),
        ];
        let elms = vec![
            create_shape_element("title", Cardinality::One),
            create_shape_element("slug", Cardinality::One),
        ];
        EValue::Object {
            shape: ObjectShape::new(elms),
            fields: object_values,
        }
    }
}
