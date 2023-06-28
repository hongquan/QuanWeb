use fievar::Fields;
use serde::{Deserialize, Serialize};

use crate::models::DocFormat;

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
}

#[derive(Debug, Deserialize, Fields)]
pub struct BlogCategoryPatchData {
    pub title: Option<String>,
    pub slug: Option<String>,
}
