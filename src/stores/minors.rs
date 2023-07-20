use edgedb_tokio::{Client, Error};

use crate::models::minors::Presentation;

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
