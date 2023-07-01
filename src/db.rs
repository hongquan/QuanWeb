use async_fred_session::RedisSessionStore;
use fred::{pool::RedisPool, types::RedisConfig, error::RedisError};

use crate::consts::DB_NAME;

pub async fn get_edgedb_client() -> Result<edgedb_tokio::Client, edgedb_tokio::Error> {
    let mut builder = edgedb_tokio::Builder::new();
    builder.database(DB_NAME)?;
    let config = builder.build_env().await?;
    Ok(edgedb_tokio::Client::new(&config))
}

pub async fn get_redis_store() -> Result<RedisSessionStore, RedisError> {
    let config = RedisConfig::default();
    let pool = RedisPool::new(config, None, None, 2)?;
    pool.connect();
    pool.wait_for_connect().await?;
    let store = RedisSessionStore::from_pool(pool, Some(DB_NAME.to_string()));
    Ok(store)
}
