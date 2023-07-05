use edgedb_tokio::{Client, Error};
use crate::models::User;

pub async fn get_user_by_email(email: &str, client: &Client) -> Result<Option<User>, Error> {
    let q = "SELECT User {id, username, email, password, is_active, is_superuser} FILTER .email = <str>$0 LIMIT 1";
    tracing::debug!("To query: {}", q);
    let user: Option<User> = client.query_single(q, &(email,)).await?;
    Ok(user)
}
