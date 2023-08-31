use edgedb_tokio::{Client, Error};
use crate::models::{User, users::MiniUser};

pub async fn get_user_by_email(email: &str, client: &Client) -> Result<Option<User>, Error> {
    let q = "SELECT User {id, username, email, password, is_active, is_superuser} FILTER .email = <str>$0 LIMIT 1";
    tracing::debug!("To query: {}", q);
    let user: Option<User> = client.query_single(q, &(email,)).await?;
    Ok(user)
}

pub async fn list_mini_users(client: &Client) -> Result<Vec<MiniUser>, Error> {
    let q = "SELECT User {id, username, email}";
    let users: Vec<MiniUser> = client.query(q, &()).await?;
    Ok(users)
}
