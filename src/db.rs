use config::Config;
use edgedb_errors::kinds::ConfigurationError;
use edgedb_errors::ErrorKind;
use edgedb_tokio::TlsSecurity;
use tower_sessions::fred::{clients::RedisClient, error::RedisError, interfaces::ClientLike};
use tower_sessions::RedisStore;

use crate::conf::KEY_EDGEDB_INSTANCE;
use crate::consts::DB_NAME;

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
    let config = builder.build_env().await?.with_database(DB_NAME)?;
    Ok(edgedb_tokio::Client::new(&config))
}

pub async fn get_redis_store() -> Result<RedisStore, RedisError> {
    let client = RedisClient::default();
    let _c = client.connect();
    client.wait_for_connect().await?;
    tracing::debug!("Connected to Redis");
    let store = RedisStore::new(client);
    Ok(store)
}
