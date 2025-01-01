use config::Config;
use edgedb_errors::kinds::ConfigurationError;
use edgedb_errors::ErrorKind;
use edgedb_tokio::TlsSecurity;
use fred::error::Error as FredError;
use fred::types::ConnectHandle;
use tower_sessions_redis_store::{fred::prelude::*, RedisStore};

use crate::conf::KEY_EDGEDB_INSTANCE;

pub async fn get_edgedb_client(
    app_config: &Config,
) -> Result<edgedb_tokio::Client, edgedb_tokio::Error> {
    let mut builder = edgedb_tokio::Builder::new();
    // Our EdgeDB is only available on localhost, so no need for TLS.
    builder.tls_security(TlsSecurity::Insecure);
    let instance_name = app_config.get_string(KEY_EDGEDB_INSTANCE).map_err(|e| {
        tracing::error!("Missing EdgeDB instance name in config");
        ConfigurationError::with_message(e.to_string())
    })?;
    tracing::info!("To connect to EdgeDB instance {}", instance_name);
    builder.instance(&instance_name)?;
    // EdgeDB v5 has to be used as local server, so it is safe to use default branch name "main"
    let config = builder.build_env().await?;
    Ok(edgedb_tokio::Client::new(&config))
}

pub async fn get_redis_store() -> Result<(RedisStore<Pool>, ConnectHandle), FredError> {
    let pool = Pool::new(fred::types::config::Config::default(), None, None, None, 2)?;
    let redis_conn = pool.connect();
    pool.wait_for_connect().await?;
    tracing::debug!("Connected to Redis");
    let store = RedisStore::new(pool);
    Ok((store, redis_conn))
}
