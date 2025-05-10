use uuid::Uuid;
use serde::Serialize;
use gel_derive::Queryable;

#[derive(Debug, Clone, PartialEq, Serialize, Queryable)]
pub struct Presentation {
    pub id: Uuid,
    pub title: String,
    pub url: String,
    pub event: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Queryable)]
pub struct BookAuthor {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Queryable)]
pub struct Book {
    pub id: Uuid,
    pub title: String,
    pub download_url: Option<String>,
    pub author: Option<BookAuthor>,
}
