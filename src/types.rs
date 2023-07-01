use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use edgedb_protocol::model::Datetime as EDatetime;
use edgedb_tokio::Client;
use serde::de::Deserializer;
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};
use serde_value::Value;
use edgedb_protocol::codec::ShapeElement;
use edgedb_protocol::common::Cardinality;

// Ref: https://github.com/sirgallifrey/serde_either/blob/main/src/enums.rs
#[derive(Debug, PartialEq)]
pub enum ApiErrorDetail {
    String(String),
    HashMap(HashMap<String, String>),
}

impl Default for ApiErrorDetail {
    fn default() -> Self {
        Self::String("".to_string())
    }
}

impl Clone for ApiErrorDetail {
    fn clone(&self) -> Self {
        match self {
            Self::String(as_single) => Self::String(as_single.clone()),
            Self::HashMap(as_hashmap) => Self::HashMap(as_hashmap.clone()),
        }
    }
}

impl Serialize for ApiErrorDetail {
    fn serialize<Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
    where
        Se: Serializer,
    {
        match self {
            Self::String(as_string) => serializer.serialize_str(as_string),
            Self::HashMap(as_hashmap) => {
                let mut map = serializer.serialize_map(Some(as_hashmap.len()))?;
                for (k, v) in as_hashmap {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for ApiErrorDetail {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::String(as_string) => Ok(Self::String(as_string)),
            Value::Map(as_map) => {
                let hm: HashMap<String, String> = as_map
                    .into_iter()
                    .flat_map(|(k, v)| {
                        let key = String::deserialize(k).ok()?;
                        let value = String::deserialize(v).ok()?;
                        Some((key, value))
                    })
                    .collect();
                Ok(Self::HashMap(hm))
            }
            _ => Err(serde::de::Error::custom("expected a string or a map")),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ApiErrorShape {
    pub detail: ApiErrorDetail,
    pub origin: Option<String>,
}

impl From<String> for ApiErrorShape {
    fn from(detail: String) -> Self {
        Self {
            detail: ApiErrorDetail::String(detail),
            ..Default::default()
        }
    }
}

impl From<HashMap<String, String>> for ApiErrorShape {
    fn from(detail: HashMap<String, String>) -> Self {
        Self {
            detail: ApiErrorDetail::HashMap(detail),
            ..Default::default()
        }
    }
}

impl Into<ApiErrorDetail> for String {
    fn into(self) -> ApiErrorDetail {
        ApiErrorDetail::String(self)
    }
}

pub struct AppState {
    pub db: Client,
}

pub type SharedState = Arc<AppState>;

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

pub fn create_shape_element(name: &str, cardinality: Cardinality) -> ShapeElement {
    ShapeElement {
        name: name.to_string(),
        cardinality: Some(cardinality),
        flag_link: false,
        flag_link_property: false,
        flag_implicit: false,
    }
}
