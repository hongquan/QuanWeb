use crate::consts::DB_NAME;

pub async fn get_edgedb_client() -> Result<edgedb_tokio::Client, edgedb_tokio::Error> {
    let mut builder = edgedb_tokio::Builder::new();
    builder.database(DB_NAME)?;
    let config = builder.build_env().await?;
    Ok(edgedb_tokio::Client::new(&config))
}
