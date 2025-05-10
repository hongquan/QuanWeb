pub mod blogs;
pub mod feeds;
pub mod minors;
pub mod users;

pub use blogs::{BlogCategory, DetailedBlogPost, DocFormat, MediumBlogPost, MiniBlogPost};
pub use minors::Presentation;
pub use users::User;

#[derive(Debug, serde::Serialize, serde::Deserialize, gel_derive::Queryable)]
pub struct MinimalObject {
    pub id: uuid::Uuid,
}
