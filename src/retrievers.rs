use uuid::Uuid;
use edgedb_tokio::{Client, Error};
use crate::models::{User, RawBlogPost, BlogCategory};

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

pub async fn get_blogpost(post_id: Uuid, client: &Client) -> Result<Option<RawBlogPost>, Error> {
    let q = "
    SELECT BlogPost {
        id,
        title,
        slug,
        is_published,
        published_at,
        created_at,
        updated_at,
        categories: {
            id,
            title,
            slug,
        },
    }
    FILTER .id = <uuid>$0";
    tracing::debug!("To query: {}", q);
    let post: Option<RawBlogPost> = client.query_single(q, &(post_id,)).await?;
    Ok(post)
}

pub async fn get_blogposts(offset: Option<i64>, limit: Option<i64>, client: &Client) -> Result<Vec<RawBlogPost>, Error> {
    let q = "
    SELECT BlogPost {
        id,
        title,
        slug,
        is_published,
        published_at,
        created_at,
        updated_at,
        categories: {
            id,
            title,
            slug,
        },
    }
    ORDER BY .created_at DESC EMPTY FIRST OFFSET <optional int64>$0 LIMIT <optional int64>$1";
    let posts: Vec<RawBlogPost> = client.query(q, &(offset, limit)).await?;
    Ok(posts)
}

pub async fn get_blogcategories(offset: Option<i64>, limit: Option<i64>, client: &Client) -> Result<Vec<BlogCategory>, Error> {
    let q = "
    SELECT BlogCategory {
        id,
        title,
        slug
    } ORDER BY .title OFFSET <optional int64>$0 LIMIT <optional int64>$1";
    let categories: Vec<BlogCategory> = client.query(q, &(offset, limit)).await?;
    Ok(categories)
}


pub async fn get_all_categories_count(client: &Client) -> Result<usize, Error> {
    let q = "SELECT count(BlogCategory)";
    tracing::debug!("To query: {}", q);
    let count: i64 = client.query_required_single(q, &()).await?;
    Ok(count.try_into().unwrap_or(0))
}
