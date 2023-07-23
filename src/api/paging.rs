use std::num::NonZeroU16;

use axum::http::Uri;

use super::structs::{PaginationLinks, NPaging};
use crate::consts::DEFAULT_PAGE_SIZE;
use crate::utils::urls::update_entry_in_query;

pub fn gen_pagination_links(paging: &NPaging, total: usize, original_uri: Uri) -> PaginationLinks {
    let current_page = paging.page.unwrap_or(NonZeroU16::MIN).get();
    let per_page = paging.per_page.unwrap_or(DEFAULT_PAGE_SIZE) as usize;
    let prev_page = if current_page > 1 {
        Some(current_page - 1)
    } else {
        None
    };
    let next_page = if (current_page as usize * per_page) < total {
        Some(current_page + 1)
    } else {
        None
    };
    let next_url = next_page.map(|p| update_entry_in_query("page", p, &original_uri));
    let next_url = next_url.map(|u| paging.per_page.map(|size| update_entry_in_query("per_page", size, &u)).unwrap_or(u));
    let prev_url = prev_page.map(|p| update_entry_in_query("page", p, &original_uri));
    let prev_url = prev_url.map(|u| paging.per_page.map(|size| update_entry_in_query("per_page", size, &u)).unwrap_or(u));
    PaginationLinks {
        prev: prev_url.map(|u| u.to_string()),
        next: next_url.map(|u| u.to_string()),
    }
}
