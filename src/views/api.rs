use axum_extra::response::ErasedJson;
use serde::Serialize;

#[derive(Serialize)]
pub struct Post {
    pub id: u64,
    pub title: String,
    pub body: String,
}

pub async fn root() -> &'static str {
    "API root"
}

pub async fn list_posts() -> ErasedJson {
    ErasedJson::pretty(vec![
        Post {
            id: 1,
            title: "Hello, world!".to_string(),
            body: "This is a post".to_string(),
        },
        Post {
            id: 2,
            title: "Hello, world!".to_string(),
            body: "This is a post".to_string(),
        },
        Post {
            id: 3,
            title: "Hello, world!".to_string(),
            body: "This is a post".to_string(),
        },
    ])
}
