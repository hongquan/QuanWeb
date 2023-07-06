use time::{OffsetDateTime, macros::datetime};
use time::format_description::well_known::iso8601::{Config as Iso8601Config, TimePrecision};
use time_tz::{timezones, Tz};

pub const DB_NAME: &str = "quanweb";
pub const DEFAULT_PAGE_SIZE: u8 = 10;
pub const STATIC_URL: &str = "/static";
pub const POSTGRES_EPOCH: OffsetDateTime = datetime!(2000-01-01 0:00 UTC);
pub const ISO8601_CONFIG: Iso8601Config = Iso8601Config::DEFAULT.set_time_precision(TimePrecision::Second { decimal_digits: None });
pub const TZ_VN: &Tz = timezones::db::asia::HO_CHI_MINH;
