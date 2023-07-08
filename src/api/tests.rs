use http::Uri;

use super::paging::gen_pagination_links;
use super::structs::Paging;

#[test]
fn gen_next_url_when_per_page_is_missing() {
    let uri = Uri::from_static("/api/categories");
    let links = gen_pagination_links(
        &Paging {
            page: Some(1),
            per_page: None,
        },
        13,
        uri.clone(),
    );
    assert!(links.next == Some("/api/categories?page=2".to_string()));
}
