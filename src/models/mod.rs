pub mod users;
pub mod blogs;
pub mod minors;
pub mod feeds;

pub use users::{User, Role};
pub use blogs::{DocFormat, MediumBlogPost, DetailedBlogPost, BlogCategory, MiniBlogPost};
pub use minors::Presentation;

#[derive(Debug, serde::Serialize, serde::Deserialize, edgedb_derive::Queryable)]
pub struct MinimalObject {
    pub id: uuid::Uuid,
}
