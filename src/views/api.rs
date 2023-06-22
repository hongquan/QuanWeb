use std::cmp::max;

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
    let page = max(1, paging.0.page.unwrap_or(1));
    let per_page = max(0, paging.0.per_page.unwrap_or(10));
    let offset: i64 = ((page - 1) * per_page).try_into().unwrap_or(0);
    let limit = per_page as i64;
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
        created_at,
        updated_at,
    }
    ORDER BY .created_at DESC EMPTY FIRST OFFSET <optional int64>$0 LIMIT <optional int64>$1";
    tracing::debug!("To query: {}", q);
    let posts: Vec<RawBlogPost> = db_conn.query(q, &(offset, limit)).await.map_err(|e| {
        tracing::error!("Error querying EdgeDB: {}", display_error_verbose(&e));
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(posts.into_iter().collect()))
}
