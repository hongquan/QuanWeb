use chrono::{DateTime, Utc};
use edgedb_protocol::model::Datetime as EDatetime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{serialize_optional_edge_datetime, serialize_edge_datetime};
/*
Because EdgeDB client cannot retrieve datetime field as chrono type,
We have to use an intermediate type to grab result from EdgeDB.
then convert to final struct with chrono types (which can be serialized with serde).
*/
#[derive(Debug, Serialize, edgedb_derive::Queryable)]
pub struct RawBlogPost {
    pub id: Uuid,
    pub title: String,
    pub is_published: bool,
    #[serde(serialize_with = "serialize_optional_edge_datetime")]
    pub published_at: Option<EDatetime>,
    #[serde(serialize_with = "serialize_edge_datetime")]
    pub created_at: EDatetime,
    #[serde(serialize_with = "serialize_optional_edge_datetime")]
    pub updated_at: Option<EDatetime>,
    pub categories: Vec<BlogCategory>,
}

#[derive(Debug, Serialize, Deserialize, edgedb_derive::Queryable)]
pub struct BlogCategory {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlogPost {
    pub id: Uuid,
    pub title: String,
    pub is_published: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub categories: Vec<BlogCategory>,
}

impl From<RawBlogPost> for BlogPost {
    fn from(post: RawBlogPost) -> Self {
        let published_at: Option<DateTime<Utc>> = post.published_at.map(|d| d.into());
        let created_at: DateTime<Utc> = post.created_at.into();
        let updated_at: Option<DateTime<Utc>> = post.updated_at.map(|d| d.into());
        BlogPost {
            id: post.id,
            title: post.title,
            is_published: post.is_published,
            published_at,
            created_at,
            updated_at,
            categories: post.categories,
        }
    }
}

impl FromIterator<RawBlogPost> for Vec<BlogPost> {
    fn from_iter<T: IntoIterator<Item = RawBlogPost>>(iter: T) -> Self {
        iter.into_iter().map(BlogPost::from).collect()
    }
}
