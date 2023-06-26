use http::Uri;
use querystring_tiny::QueryString;

pub fn update_entry_in_query(name: &str, value: usize, original_uri: &Uri) -> Uri {
    let mut query = original_uri
        .query()
        .map(|s| QueryString::decode(s.as_bytes()).ok())
        .flatten()
        .unwrap_or(QueryString::new());
    query.set(name, format!("{value}"));
    let path = original_uri.path();
    let path_and_query = match String::from_utf8(query.encode()).ok() {
        Some(query) => format!("{path}?{query}"),
        None => path.to_string(),
    };
    let new_uri = Uri::builder().path_and_query(&path_and_query).build().unwrap_or(original_uri.clone());
    new_uri
}
