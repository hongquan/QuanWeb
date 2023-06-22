use serde::{Serialize, Deserialize};
use uuid::Uuid;
use edgedb_protocol::model::Datetime as EDatetime;
use chrono::{DateTime, Utc};

#[derive(Deserialize, Debug)]
pub struct Paging {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

#[derive(Debug, edgedb_derive::Queryable)]
pub struct RawBlogPost {
    pub id: Uuid,
    pub title: String,
    pub is_published: bool,
    pub published_at: Option<EDatetime>,
}


#[derive(Serialize, Deserialize)]
pub struct BlogPost {
    pub id: Uuid,
    pub title: String,
    pub is_published: bool,
    pub published_at: Option<DateTime<Utc>>,
}

impl From<RawBlogPost> for BlogPost {
    fn from(post: RawBlogPost) -> Self {
        let published_at: Option<DateTime<Utc>> = post.published_at.map(|d| d.into());
        BlogPost {
            id: post.id,
            title: post.title,
            is_published: post.is_published,
            published_at: published_at,
        }
    }
}

impl FromIterator<RawBlogPost> for Vec<BlogPost> {
    fn from_iter<T: IntoIterator<Item = RawBlogPost>>(iter: T) -> Self {
        iter.into_iter()
        .map(BlogPost::from).collect()
    }
}
