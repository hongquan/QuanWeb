use syntect::html::ClassStyle;


pub const DB_NAME: &str = "quanweb";
pub const DEFAULT_PAGE_SIZE: u8 = 10;
pub const STATIC_URL: &str = "/static";
pub const UNCATEGORIZED_URL: &str = "/category/_uncategorized/";
pub const TEMPLATE_DIR: &str = "minijinja";
pub const SYNTECT_CLASS_STYLE: ClassStyle = ClassStyle::SpacedPrefixed { prefix: "st-" };
