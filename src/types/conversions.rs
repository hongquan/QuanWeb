use chrono::{DateTime, Utc};
use chrono_tz::Asia::Ho_Chi_Minh;
use edgedb_protocol::model::Datetime as EDatetime;
use minijinja::value::Value as MJValue;
use serde::ser::Serializer;
use serde::Serialize;

pub fn edge_datetime_to_jinja(dt: EDatetime) -> MJValue {
    let chrono: DateTime<Utc> = dt.into();
    chrono
        .with_timezone(&Ho_Chi_Minh)
        .format("%+")
        .to_string()
        .into()
}

/* Serde serializers to serialize EdgeDB's Datetime type */
pub fn serialize_edge_datetime<Se>(edt: &EDatetime, serializer: Se) -> Result<Se::Ok, Se::Error>
where
    Se: Serializer,
{
    let cdt: DateTime<Utc> = edt.into();
    cdt.serialize(serializer)
}

pub fn serialize_optional_edge_datetime<Se>(
    edt: &Option<EDatetime>,
    serializer: Se,
) -> Result<Se::Ok, Se::Error>
where
    Se: Serializer,
{
    match edt {
        Some(edt) => serialize_edge_datetime(edt, serializer),
        None => serializer.serialize_none(),
    }
}
