use uuid::Uuid;
use fievar::Fields;
use serde::{Deserialize, Serialize};
use edgedb_protocol::value::Value as EValue;
use edgedb_protocol::codec::ObjectShape;
use edgedb_protocol::common::Cardinality;

use crate::models::DocFormat;
use crate::types::create_shape_element;

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
where T: Serialize
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
    pub categories: Vec<Uuid>,
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

    pub fn gen_set_clause(&self) -> String {
        let lines = vec![
            "title := <str>$title",
            "slug := <str>$slug",
            "is_published := <optional bool>$is_published",
            "body := <optional str>$body",
            "categories := (
                SELECT BlogCategory FILTER .id IN array_unpack(<array<Uuid>>$categories)
            )",
        ];
        let sep = format!(",\n{}", " ".repeat(8));
        lines.join(&sep)
    }

    pub fn make_edgedb_object(&self) -> EValue {
        let categories: Vec<EValue> = self.categories.clone().into_iter().map(EValue::Uuid).collect();
        let object_values = vec![
            EValue::from(self.title.clone()),
            EValue::from(self.slug.clone()),
            self.is_published.map_or(EValue::Nothing, EValue::Bool),
            self.format.clone().map_or(EValue::Nothing, EValue::from),
            self.body.clone().map_or(EValue::Nothing, EValue::Str),
            EValue::Array(categories),
        ].into_iter().map(Some).collect();
        let elms = vec![
            create_shape_element("title", Cardinality::One),
            create_shape_element("slug", Cardinality::One),
            create_shape_element("is_published", Cardinality::AtMostOne),
            create_shape_element("body", Cardinality::AtMostOne),
            create_shape_element("categories", Cardinality::Many),
        ];
        EValue::Object { shape: ObjectShape::new(elms), fields: object_values }
    }
}
