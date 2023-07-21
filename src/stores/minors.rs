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
