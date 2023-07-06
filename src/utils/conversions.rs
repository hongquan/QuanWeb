use time::OffsetDateTime;
use minijinja::value::Value as MJValue;
use time::format_description::well_known::Iso8601;
use time_tz::OffsetDateTimeExt;

use crate::consts::{ISO8601_CONFIG, TZ_VN};

pub fn time_rs_to_iso8601(dt: OffsetDateTime) -> Option<String> {
    dt.to_timezone(TZ_VN).format(&Iso8601::<{ ISO8601_CONFIG.encode() }>).map_err(|e| {
        tracing::error!("Failed to format to ISO8601: {:?}", e);
        e
    }).ok()
}

pub fn datetime_to_jinja(dt: OffsetDateTime) -> Option<MJValue> {
    time_rs_to_iso8601(dt).map(MJValue::from)
}
