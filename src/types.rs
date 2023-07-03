use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use edgedb_protocol::model::Datetime as EDatetime;
use edgedb_tokio::Client;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use edgedb_protocol::codec::ShapeElement;
use edgedb_protocol::common::Cardinality;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorShape {
    pub message: String,
    pub fields: Option<HashMap<String, String>>,
    pub code: Option<String>,
}

impl Default for ApiErrorShape {
    fn default() -> Self {
        Self {
            message: "Some error".to_string(),
            fields: None,
            code: None,
        }
    }
}

impl From<String> for ApiErrorShape {
    fn from(message: String) -> Self {
        Self {
            message,
            ..Default::default()
        }
    }
}

impl From<HashMap<String, String>> for ApiErrorShape {
    fn from(fields: HashMap<String, String>) -> Self {
        Self {
            fields: Some(fields),
            ..Default::default()
        }
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
