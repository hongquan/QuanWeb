use edgedb_tokio::{Client, Error};
use crate::models::User;

#[allow(dead_code)]
pub async fn get_first_user(client: Client) -> Result<Option<User>, Error> {
    let q = "SELECT User {id, username, email, password, is_active, is_superuser} LIMIT 1";
    tracing::debug!("To query: {}", q);
    let user: Option<User> = client.query_single(q, &()).await?;
    Ok(user)
}

pub async fn get_user_by_email(email: &str, client: &Client) -> Result<Option<User>, Error> {
    let q = "SELECT User {id, username, email, password, is_active, is_superuser} FILTER .email = <str>$0 LIMIT 1";
    tracing::debug!("To query: {}", q);
    let user: Option<User> = client.query_single(q, &(email,)).await?;
    Ok(user)
}

pub async fn get_all_posts_count(client: &Client) -> Result<usize, Error> {
    let q = "SELECT count(BlogPost)";
    tracing::debug!("To query: {}", q);
    let count: i64 = client.query_required_single(q, &()).await?;
    Ok(count.try_into().unwrap_or(0))
}
