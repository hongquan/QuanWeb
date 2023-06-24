use edgedb_tokio::{Client, Error};
use crate::models::User;

pub async fn get_first_user(client: Client) -> Result<Option<User>, Error> {
    let q = "SELECT User {id, username, email, password, is_active, is_superuser} LIMIT 1";
    tracing::debug!("To query: {}", q);
    let user: Option<User> = client.query_single(q, &()).await?;
    Ok(user)
}
