use time::OffsetDateTime;
use time::format_description::well_known::Iso8601;
use minijinja::value::Value;

use crate::consts::ISO8601_CONFIG;

pub fn debug_filter(value: Value) -> String {
    tracing::info!("Value: {:?}; Kind {}", value, value.kind());
    format!("{}", value)
}

pub fn to_datetime(value: String) -> String {
    match OffsetDateTime::parse(&value, &Iso8601::<{ ISO8601_CONFIG.encode() }>) {
        Ok(dt) => tracing::info!("Parsed datetime: {:?}", dt),
        Err(e) => tracing::error!("Failed to parse datetime: {:?}", e),
    };
    value
}

pub fn post_detail_url(slug: String, created_at: String) -> String {
    let x = OffsetDateTime::parse(&created_at, &Iso8601::<{ ISO8601_CONFIG.encode() }>).ok();
    match x {
        Some(dt) => {
            let y = dt.year();
            let m = dt.month() as u8;
            format!("/post/{y}/{m}/{slug}")
        },
        None => format!("/post/y/m/{}", slug),
    }
}
