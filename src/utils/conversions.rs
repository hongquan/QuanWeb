use minijinja::value::Value as MJValue;
use edgedb_protocol::model::Datetime as EDatetime;
use chrono::{DateTime, Utc};
use chrono_tz::Asia::Ho_Chi_Minh;

pub fn edge_datetime_to_jinja(dt: EDatetime) -> MJValue {
    let chrono: DateTime<Utc> = dt.into();
    chrono.with_timezone(&Ho_Chi_Minh).format("%+").to_string().into()
}
