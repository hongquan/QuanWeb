use std::collections::HashMap;

use chrono::{DateTime, Utc};
use chrono_tz::Asia::Ho_Chi_Minh;
use edgedb_protocol::codec::ObjectShape;
use edgedb_protocol::common::Cardinality;
use edgedb_protocol::model::Datetime as EDatetime;
use edgedb_protocol::value::Value as EValue;
use serde::ser::Serializer;
use serde::Serialize;
use minijinja::value::{Kwargs, Value as MJValue, ValueKind as MJValueKind};
use fluent_bundle::FluentValue;

use super::create_shape_element;

/* Serde serializers to serialize EdgeDB's Datetime type */
pub fn serialize_edge_datetime<Se>(edt: &EDatetime, serializer: Se) -> Result<Se::Ok, Se::Error>
where
    Se: Serializer,
{
    let chrono: DateTime<Utc> = edt.into();
    let dt_string = chrono.with_timezone(&Ho_Chi_Minh).format("%+").to_string();
    dt_string.serialize(serializer)
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

// Ref: https://github.com/edgedb/edgedb-rust/blob/master/edgedb-protocol/src/value.rs#L100
pub fn edge_object_from_simple_pairs<N, V>(iter: impl IntoIterator<Item = (N, V)>) -> EValue
where
    N: ToString,
    V: Into<Option<EValue>>,
{
    let mut elements = Vec::new();
    let mut fields: Vec<Option<EValue>> = Vec::new();
    for (key, val) in iter.into_iter() {
        elements.push(create_shape_element(key, Cardinality::One));
        fields.push(val.into());
    }
    EValue::Object {
        shape: ObjectShape::new(elements),
        fields,
    }
}

pub fn edge_object_from_pairs<N, V>(iter: impl IntoIterator<Item = (N, (V, Cardinality))>) -> EValue
where
    N: ToString,
    V: Into<Option<EValue>>,
{
    let mut elements = Vec::new();
    let mut fields: Vec<Option<EValue>> = Vec::new();
    for (key, (val, cardinality)) in iter.into_iter() {
        elements.push(create_shape_element(key, cardinality));
        fields.push(val.into());
    }
    EValue::Object {
        shape: ObjectShape::new(elements),
        fields,
    }
}

pub fn jinja_value_to_fluent_value(value: MJValue) -> FluentValue<'static> {
    match value.kind() {
        MJValueKind::Number => {
            let n: f64 = value.try_into().unwrap_or(0.0);
            FluentValue::from(n)
        },
        MJValueKind::String => {
            let s: String = value.try_into().unwrap_or_default();
            FluentValue::from(s)
        },
        MJValueKind::Bool => {
            let b: u8 = value.try_into().unwrap_or(0);
            FluentValue::from(b)
        },
        _ => FluentValue::None,
    }
}

pub fn jinja_kwargs_to_fluent_args(kwargs: Kwargs) -> Option<HashMap<String, FluentValue<'static>>> {
    let mj_value: MJValue = kwargs.into();
    let mut hm: HashMap<String, FluentValue<'static>> = HashMap::new();
    let iter = mj_value.try_iter().ok()?;
    for key in iter {
        let skey = match key.as_str() {
            Some(sk) => sk.to_string(),
            None => continue,
        };
        let value = match mj_value.get_item(&key) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let fv = jinja_value_to_fluent_value(value);
        hm.insert(skey, fv);
    }
    Some(hm)
}
