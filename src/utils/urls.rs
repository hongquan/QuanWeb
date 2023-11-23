use http::Uri;
use querystring_tiny::QueryString;
use std::fmt::Display;

pub fn update_entry_in_query<T: Display>(name: &str, value: T, original_uri: &Uri) -> Uri {
    let mut query = original_uri
        .query()
        .and_then(|s| QueryString::decode(s.as_bytes()).ok())
        .unwrap_or_default();
    query.set(name, format!("{value}"));
    let path = original_uri.path();
    let path_and_query = match String::from_utf8(query.encode()).ok() {
        Some(query) => format!("{path}?{query}"),
        None => path.to_string(),
    };
    Uri::builder()
        .path_and_query(&path_and_query)
        .build()
        .unwrap_or(original_uri.clone())
}
