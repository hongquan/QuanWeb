pub mod users;
pub mod blogs;

pub use users::{User, Role};
pub use blogs::{DocFormat, MediumBlogPost, DetailedBlogPost, BlogCategory, MiniBlogPost};

#[derive(Debug, serde::Serialize, serde::Deserialize, edgedb_derive::Queryable)]
pub struct MinimalObject {
    pub id: uuid::Uuid,
}
