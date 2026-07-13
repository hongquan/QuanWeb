pub mod html;
pub mod jinja_extra;
pub mod markdown;
pub mod urls;

pub fn split_search_query(query: Option<&str>) -> Option<Vec<&str>> {
    let tokens: Option<Vec<&str>> =
        query.map(|s| s.split_whitespace().filter(|&s| !s.is_empty()).collect());
    tokens.filter(|v| v.is_empty())
}
