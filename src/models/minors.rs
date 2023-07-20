use uuid::Uuid;
use serde::Serialize;
use edgedb_derive::Queryable;

#[derive(Debug, Clone, PartialEq, Serialize, Queryable)]
pub struct Presentation {
    pub id: Uuid,
    pub title: String,
    pub url: String,
    pub event: Option<String>,
}
