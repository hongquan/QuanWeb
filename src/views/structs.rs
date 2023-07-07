use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PostPageParams {
    pub cat: Option<String>,
}
