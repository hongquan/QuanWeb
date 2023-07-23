use http::Uri;
use nonzero::nonzero as nz;

use super::paging::gen_pagination_links;
use super::structs::NPaging;

#[test]
fn gen_next_url_when_per_page_is_missing() {
    let uri = Uri::from_static("/api/categories");
    let links = gen_pagination_links(
        &NPaging {
            page: Some(nz!(1u16)),
            per_page: None,
        },
        13,
        uri.clone(),
    );
    assert!(links.next == Some("/api/categories?page=2".to_string()));
}
