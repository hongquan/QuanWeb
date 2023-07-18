
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PostPageParams {
    pub cat: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct LaxPaging {
    pub page: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetLangReq {
    pub lang: String,
}
