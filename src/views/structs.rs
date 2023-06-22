use serde::{Serialize, Deserialize};
use uuid::Uuid;
use edgedb_protocol::model::Datetime as EDatetime;
use chrono::{DateTime, Utc, NaiveDateTime};

#[derive(Deserialize, Debug)]
pub struct Paging {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

fn to_unix_micros(dt: &EDatetime) -> i64 {
    dt.to_unix_micros()
}


#[derive(Serialize, Deserialize)]
#[serde(remote = "EDatetime")]
pub struct IMDatetime {
    #[serde(getter = "to_unix_micros")]
    micros: i64,
}

impl From<IMDatetime> for EDatetime {
    fn from(dt: IMDatetime) -> Self {
        EDatetime::from_unix_micros(dt.micros)
    }
}

impl From<EDatetime> for IMDatetime {
    fn from(dt: EDatetime) -> Self {
        IMDatetime {
            micros: dt.to_unix_micros(),
        }
    }
}

#[derive(Debug, edgedb_derive::Queryable)]
pub struct RawBlogPost {
    pub id: Uuid,
    pub title: String,
    pub is_published: bool,
    pub published_at: Option<EDatetime>,
}

pub fn edgedb_datetime_to_chrono(dt: EDatetime) -> Option<DateTime<Utc>> {
    let naive = NaiveDateTime::from_timestamp_micros(dt.to_unix_micros())?;
    let chro_dt = DateTime::<Utc>::from_utc(naive, Utc);
    Some(chro_dt)
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
        BlogPost {
            id: post.id,
            title: post.title,
            is_published: post.is_published,
            published_at: post.published_at.map(edgedb_datetime_to_chrono).flatten(),
        }
    }
}
