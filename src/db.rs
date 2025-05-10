use config::Config;
use fred::error::Error as FredError;
use fred::types::ConnectHandle;
use gel_errors::kinds::ConfigurationError;
use gel_errors::ErrorKind;
use gel_tokio::TlsSecurity;
use tower_sessions_redis_store::{fred::prelude::*, RedisStore};

use crate::conf::KEY_EDGEDB_INSTANCE;

pub async fn get_gel_client(app_config: &Config) -> Result<gel_tokio::Client, gel_tokio::Error> {
    let mut builder = gel_tokio::Builder::new();
    // Our Gel is only available on localhost, so no need for TLS.
    builder.tls_security(TlsSecurity::Insecure);
    let instance_name = app_config.get_string(KEY_EDGEDB_INSTANCE).map_err(|e| {
        tracing::error!("Missing Gel instance name in config");
        ConfigurationError::with_message(e.to_string())
    })?;
    tracing::info!("To connect to Gel instance {}", instance_name);
    builder.instance(&instance_name)?;
    // Gel v5 has to be used as local server, so it is safe to use default branch name "main"
    let config = builder.build_env().await?;
    Ok(gel_tokio::Client::new(&config))
}

pub async fn get_redis_store() -> Result<(RedisStore<Pool>, ConnectHandle), FredError> {
    let pool = Pool::new(fred::types::config::Config::default(), None, None, None, 2)?;
    let redis_conn = pool.connect();
    pool.wait_for_connect().await?;
    tracing::debug!("Connected to Redis");
    let store = RedisStore::new(pool);
    Ok((store, redis_conn))
}
