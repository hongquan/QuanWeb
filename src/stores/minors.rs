use edgedb_tokio::{Client, Error};

use crate::models::minors::{Presentation, Book};

pub async fn get_all_talks(client: &Client) -> Result<Vec<Presentation>, Error> {
    let q = "
    SELECT Presentation {
        id,
        title,
        url,
        event,
    }";
    client.query(q, &()).await
}

pub async fn get_all_books(client: &Client) -> Result<Vec<Book>, Error> {
    let q = "
    SELECT Book {
        id,
        title,
        download_url,
        author: {
            id,
            name,
        }
    }";
    client.query(q, &()).await
}

pub async fn get_presentations(offset: Option<i64>, limit: Option<i64>, client: &Client) -> Result<Vec<Presentation>, Error> {
    let q = "
    SELECT Presentation {
        id,
        title,
        url,
        event,
    }
    ORDER BY .title DESC EMPTY FIRST OFFSET <optional int64>$0 LIMIT <optional int64>$1";
    let presentations: Vec<Presentation> = client.query(q, &(offset, limit)).await?;
    Ok(presentations)
}

pub async fn get_all_presentations_count(client: &Client) -> Result<u16, Error> {
    let q = "
    SELECT count(Presentation)";
    let count: i64 = client.query_required_single(q, &()).await?;
    Ok(count.try_into().unwrap_or(0))
}
