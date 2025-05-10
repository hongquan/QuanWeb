use crate::models::{users::MiniUser, User};
use gel_tokio::{Client, Error};
use tracing::debug;

pub async fn get_user_by_email(email: &str, client: &Client) -> Result<Option<User>, Error> {
    let q = "SELECT User {id, username, email, password, is_active, is_superuser} FILTER .email = <str>$0 LIMIT 1";
    debug!("To query: {q}");
    let user: Option<User> = client.query_single(q, &(email,)).await?;
    Ok(user)
}

pub async fn list_mini_users(client: &Client) -> Result<Vec<MiniUser>, Error> {
    let q = "SELECT User {id, username, email}";
    let users: Vec<MiniUser> = client.query(q, &()).await?;
    Ok(users)
}
