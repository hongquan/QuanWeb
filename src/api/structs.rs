use edgedb_protocol::codec::ObjectShape;
use edgedb_protocol::common::Cardinality;
use edgedb_protocol::value::Value as EValue;
use fievar::Fields;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::DocFormat;
use crate::types::create_shape_element;
use crate::utils::markdown::{make_excerpt, markdown_to_html};

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
    pub links: PaginationLinks,
    pub objects: Vec<T>,
}

impl<T> Default for ObjectListResponse<T> {
    fn default() -> Self {
        Self {
            count: 0,
            links: Default::default(),
            objects: vec![],
        }
    }
}

#[allow(dead_code)]
impl<T> ObjectListResponse<T>
where
    T: Serialize,
{
    pub fn new(objects: Vec<T>) -> Self {
        let count = objects.len();
        Self {
            count,
            objects,
            ..Default::default()
        }
    }

    pub fn with_count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    pub fn with_pagination_links(mut self, links: PaginationLinks) -> Self {
        self.links = links;
        self
    }

    pub fn with_next_url(mut self, next_url: String) -> Self {
        self.links.next = Some(next_url);
        self
    }

    pub fn with_prev_url(mut self, prev_url: String) -> Self {
        self.links.prev = Some(prev_url);
        self
    }
}

#[derive(Debug, Deserialize, Fields)]
pub struct BlogPostPatchData {
    pub title: Option<String>,
    pub slug: Option<String>,
    pub is_published: Option<bool>,
    pub format: Option<DocFormat>,
    pub body: Option<String>,
}

#[derive(Debug, Deserialize, Fields)]
pub struct BlogCategoryPatchData {
    pub title: Option<String>,
    pub slug: Option<String>,
}

#[derive(Debug, Default, Deserialize, Fields)]
pub struct BlogPostCreateData {
    pub title: String,
    pub slug: String,
    pub is_published: Option<bool>,
    pub format: Option<DocFormat>,
    pub body: Option<String>,
    pub categories: Option<Vec<Uuid>>,
}

#[allow(dead_code)]
impl BlogPostCreateData {
    pub fn new(title: String, slug: String) -> Self {
        Self {
            title,
            slug,
            ..Default::default()
        }
    }

    pub fn gen_set_clause<'a>(&self, submitted_fields: &Vec<&String>) -> String {
        let mut lines = vec![
            "title := <str>$title",
            "slug := <str>$slug",
            // TODO: "format := <optional DocFormat>$format",
        ];
        if submitted_fields.iter().any(|&f| f == "is_published") {
            lines.push("is_published := <optional bool>$is_published");
        }
        if submitted_fields.iter().any(|&f| f == "body") {
            // If user submitted "body" field, we will generate "html", "excerpt" and write, too
            lines.push("body := <optional str>$body");
            lines.push("html := <optional str>$html");
            lines.push("excerpt := <optional str>$excerpt");
        }
        if submitted_fields.iter().any(|&f| f == "format") {
            lines.push("format := <optional DocFormat>$format");
        }
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
        let categories: Vec<EValue> = self
            .categories
            .clone()
            .map_or(Vec::new(), |v| v.into_iter().map(EValue::Uuid).collect());
        let mut object_values = vec![
            Some(EValue::from(self.title.clone())),
            Some(EValue::from(self.slug.clone())),
            // self.format.clone().map(EValue::from),
        ];
        let mut elms = vec![
            create_shape_element("title", Cardinality::One),
            create_shape_element("slug", Cardinality::One),
        ];
        if submitted_fields.iter().any(|&f| f == "is_published") {
            object_values.push(self.is_published.map(EValue::Bool));
            elms.push(create_shape_element("is_published", Cardinality::AtMostOne));
        }
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
        if submitted_fields.iter().any(|&f| f == "format") {
            object_values.push(self.format.clone().map(EValue::from));
            elms.push(create_shape_element("format", Cardinality::AtMostOne));
        }
        // "categories" is a link property
        if self.categories.is_some() {
            let mut categories_elm = create_shape_element("categories", Cardinality::Many);
            categories_elm.flag_link = true;
            elms.push(categories_elm);
            object_values.push(Some(EValue::Array(categories)));
        }
        EValue::Object {
            shape: ObjectShape::new(elms),
            fields: object_values,
        }
    }
}
