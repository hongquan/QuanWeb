use chrono::DateTime;

pub fn post_detail_url(slug: String, created_at: String) -> String {
    match DateTime::parse_from_rfc3339(&created_at) {
        Ok(x) => {
            format!("/post/{}/{}", x.format("%Y/%m"), slug)
        },
        Err(e) => {
            tracing::error!("Failed to parse datetime: {:?}", e);
            format!("/post/y/m/{}", slug)
        }
    }
}
