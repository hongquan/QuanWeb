pub mod urls;
pub mod markdown;
pub mod html;
pub mod jinja_extra;

pub fn split_search_query(query: Option<&str>) -> Option<Vec<&str>> {
    let tokens: Option<Vec<&str>> = query.map(|s| s.split_whitespace().filter(|&s| !s.is_empty()).collect());
    tokens.and_then(|v| if v.is_empty() { None } else { Some(v) })
}
