use config::Config;
use fred::error::Error as FredError;
use gel_errors::ErrorKind;
use gel_errors::kinds::ConfigurationError;
use gel_tokio::{InstanceName, TlsSecurity};
use tower_sessions_redis_store::{RedisStore, fred::prelude::*};

use crate::conf::KEY_EDGEDB_INSTANCE;

pub async fn get_gel_client(app_config: &Config) -> Result<gel_tokio::Client, gel_tokio::Error> {
    let instance_name = app_config.get_string(KEY_EDGEDB_INSTANCE).map_err(|e| {
        tracing::error!("Missing Gel instance name in config");
        ConfigurationError::with_message(e.to_string())
    })?;
    tracing::info!("To connect to Gel instance {}", instance_name);
    // Our Gel is only available on localhost, so no need for TLS.
    let builder = gel_tokio::Builder::new()
        .instance(InstanceName::Local(instance_name))
        .tls_security(TlsSecurity::Insecure);
    // Gel v5 has to be used as local server, so it is safe to use default branch name "main"
    let config = builder.build()?;
    Ok(gel_tokio::Client::new(&config))
}

pub async fn get_redis_store() -> Result<RedisStore<Pool>, FredError> {
    let pool = Pool::new(fred::types::config::Config::default(), None, None, None, 2)?;
    let _redis_conn = pool.connect();
    pool.wait_for_connect().await?;
    tracing::debug!("Connected to Redis");
    let store = RedisStore::new(pool);
    Ok(store)
}
