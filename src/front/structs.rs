use std::num::NonZeroU16;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PostPageParams {
    pub cat: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct LaxPaging {
    pub page: Option<String>,
}

impl LaxPaging {
    /// Get the page as non-zero number (default to 1).
    /// The type is NonZeroU16 because our website is small enough for page number
    /// to be fit in u16.
    pub fn get_page_as_number(&self) -> NonZeroU16 {
        self.page.as_deref().map(|s| s.parse().ok()).flatten().unwrap_or(NonZeroU16::MIN)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetLangReq {
    pub lang: String,
}
