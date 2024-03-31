use syntect::html::ClassStyle;

pub const DB_NAME: &str = "quanweb";
pub const DEFAULT_PAGE_SIZE: u8 = 10;
pub const STATIC_URL: &str = "/static";
pub const UNCATEGORIZED_URL: &str = "/category/_uncategorized/";
#[allow(dead_code)]
pub const SYNTECT_CLASS_STYLE: ClassStyle = ClassStyle::SpacedPrefixed { prefix: "st-" };
pub const KEY_LANG: &str = "lang";
pub const DEFAULT_LANG: &str = "en";
pub const ALPINE_HIGHLIGHTING_APP: &str = "need_highlight";
pub const ALPINE_ORIG_CODE_ELM: &str = "orig_code";
// Given by comrak
pub const ATTR_CODEFENCE_EXTRA: &str = "data-meta";
