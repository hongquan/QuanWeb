use axum::http::StatusCode;
use axum_extra::response::ErasedJson;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use edgedb_errors::display::display_error_verbose;

use super::super::consts::DB_NAME;

#[derive(Debug, edgedb_derive::Queryable, Deserialize, Serialize)]
pub struct BlogPost {
    pub id: Uuid,
    pub title: String,
}

pub async fn root() -> &'static str {
    "API root"
}

async fn get_edgedb_client() -> Result<edgedb_tokio::Client, edgedb_tokio::Error> {
    let mut builder = edgedb_tokio::Builder::new();
    builder.database(DB_NAME)?;
    let config = builder.build_env().await?;
    Ok(edgedb_tokio::Client::new(&config))
}

pub async fn list_posts() -> Result<ErasedJson, StatusCode> {
    let db_conn = get_edgedb_client().await.map_err(|e| {
        eprintln!("Error connecting to EdgeDB: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let posts: Vec<BlogPost> = db_conn.query("SELECT BlogPost {id, title}", &()).await.map_err(|e| {
        tracing::error!("Error querying EdgeDB: {}", display_error_verbose(&e));
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let resp = ErasedJson::pretty(&posts);
    Ok(resp)
}
