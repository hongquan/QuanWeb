use url::Url;

use crate::consts::DEFAULT_PAGE_SIZE;
use super::structs::{Paging, PaginationLinks};


pub fn gen_pagination_links(paging: &Paging, total: usize, mut base_url: Url) -> PaginationLinks {
    let current_page = paging.page.unwrap_or(1);
    let per_page = paging.per_page.unwrap_or(DEFAULT_PAGE_SIZE);
    let prev_page = if current_page > 1 {
        Some(current_page - 1)
    } else {
        None
    };
    let next_page = if current_page * per_page < total {
        Some(current_page + 1)
    } else {
        None
    };
    let mut query_pairs = base_url.query_pairs_mut();
    let next_url = next_page.map(|p| {
        query_pairs.append_pair("page", p.to_string().as_str()).finish().to_string()
    });
    let prev_url = prev_page.map(|p| {
        query_pairs.append_pair("page", p.to_string().as_str()).finish().to_string()
    });
    PaginationLinks {
        prev: prev_url,
        next: next_url,
    }
}
