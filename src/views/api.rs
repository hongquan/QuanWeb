use axum::http::StatusCode;
use axum::Json;
use edgedb_errors::display::display_error_verbose;
use axum_extra::extract::Query;

use super::super::consts::DB_NAME;
use super::structs::{Paging, RawBlogPost, BlogPost};

pub async fn root() -> &'static str {
    "API root"
}

async fn get_edgedb_client() -> Result<edgedb_tokio::Client, edgedb_tokio::Error> {
    let mut builder = edgedb_tokio::Builder::new();
    builder.database(DB_NAME)?;
    let config = builder.build_env().await?;
    Ok(edgedb_tokio::Client::new(&config))
}

pub async fn list_posts(paging: Query<Paging>) -> Result<Json<Vec<BlogPost>>, StatusCode> {
    tracing::info!("Paging: {:?}", paging);
    let db_conn = get_edgedb_client().await.map_err(|e| {
        eprintln!("Error connecting to EdgeDB: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let q = "
    SELECT BlogPost {
        id,
        title,
        is_published,
        published_at,
    }
    ORDER BY .created_at DESC EMPTY FIRST";
    let posts: Vec<RawBlogPost> = db_conn.query(q, &()).await.map_err(|e| {
        tracing::error!("Error querying EdgeDB: {}", display_error_verbose(&e));
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(posts.into_iter().collect()))
}
