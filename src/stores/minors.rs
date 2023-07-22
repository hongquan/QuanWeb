use edgedb_tokio::{Client, Error};
use uuid::Uuid;

use crate::models::minors::{Book, BookAuthor, Presentation};

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

pub async fn get_presentations(
    offset: Option<i64>,
    limit: Option<i64>,
    client: &Client,
) -> Result<Vec<Presentation>, Error> {
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
    let q = "SELECT count(Presentation)";
    let count: i64 = client.query_required_single(q, &()).await?;
    Ok(count.try_into().unwrap_or(0))
}

pub async fn get_presentation(id: Uuid, client: &Client) -> Result<Option<Presentation>, Error> {
    let q = "SELECT Presentation { id, title, url, event } FILTER .id = <uuid>$0";
    let object = client.query_single(q, &(id,)).await?;
    Ok(object)
}

pub async fn get_book_authors(
    offset: Option<i64>,
    limit: Option<i64>,
    client: &Client,
) -> Result<Vec<BookAuthor>, Error> {
    let q = "
    SELECT BookAuthor {
        id,
        name,
    }
    ORDER BY .name ASC EMPTY FIRST OFFSET <optional int64>$0 LIMIT <optional int64>$1";
    client.query(q, &(offset, limit)).await
}

pub async fn get_all_book_authors_count(client: &Client) -> Result<usize, Error> {
    let q = "SELECT count(BookAuthor)";
    let count: i64 = client.query_required_single(q, &()).await?;
    Ok(count.try_into().unwrap_or(0))
}

pub async fn get_book_author(id: Uuid, client: &Client) -> Result<Option<BookAuthor>, Error> {
    let q = "SELECT BookAuthor { id, name } FILTER .id = <uuid>$0";
    let object = client.query_single(q, &(id,)).await?;
    Ok(object)
}

pub async fn get_books(
    offset: Option<i64>,
    limit: Option<i64>,
    client: &Client,
) -> Result<Vec<Book>, Error> {
    let q = "
    SELECT Book {
        id,
        title,
        download_url,
        author: {
            id,
            name,
        }
    } ORDER BY .title ASC EMPTY FIRST OFFSET <optional int64>$0 LIMIT <optional int64>$1";
    let books: Vec<Book> = client.query(q, &(offset, limit)).await?;
    Ok(books)
}

pub async fn get_all_books_count(client: &Client) -> Result<usize, Error> {
    let q = "SELECT count(Book)";
    let count: i64 = client.query_required_single(q, &()).await?;
    Ok(count.try_into().unwrap_or(0))
}

pub async fn get_book(id: Uuid, client: &Client) -> Result<Option<Book>, Error> {
    let q = "SELECT Book {
        id,
        title,
        download_url,
        author: {
            id,
            name,
        }
    } FILTER .id = <uuid>$0";
    let object = client.query_single(q, &(id,)).await?;
    Ok(object)
}
