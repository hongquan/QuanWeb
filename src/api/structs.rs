
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Paging {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}
