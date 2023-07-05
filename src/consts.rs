use time::{OffsetDateTime, macros::datetime};

pub const DB_NAME: &str = "quanweb";
pub const DEFAULT_PAGE_SIZE: u8 = 10;
pub const STATIC_URL: &str = "/static";
pub const POSTGRES_EPOCH: OffsetDateTime = datetime!(2000-01-01 0:00 UTC);
