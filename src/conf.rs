use libpassgen::{generate_password, Pool};
use miette::{miette, Report};

use config::{Config, ConfigError, File};

pub const KEY_SECRET: &str = "secret_key";
pub const KEY_EDGEDB_INSTANCE: &str = "edgedb_instance";
pub const DEFAULT_PORT: u16 = 3721;
pub const ALPHANUMERIC: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

pub fn gen_fallback_secret() -> String {
    let pool: Pool = ALPHANUMERIC.parse().unwrap_or_default();
    // 64 is the secret bytes count required by axum-sessions
    generate_password(&pool, 64)
}

pub fn get_config() -> Result<Config, ConfigError> {
    let fallback_secret = gen_fallback_secret();
    Config::builder()
        .set_default(KEY_SECRET, fallback_secret)?
        .add_source(File::with_name("base_settings.toml").required(true))
        .add_source(File::with_name("custom_settings.toml").required(false))
        .add_source(File::with_name(".secrets.toml").required(false))
        .build()
}

#[allow(dead_code)]
pub fn get_secret_bytes(config: &Config) -> Result<Vec<u8>, Report> {
    let secret_str = config
        .get_string(KEY_SECRET)
        .map_err(|e| miette!("Failed to get secret key: {e}"))?;
    tracing::debug!("Secret key: {}", secret_str);
    Ok(secret_str.as_bytes().into())
}
