use std::collections::HashSet;

use ammonia::Builder;
use once_cell::sync::Lazy;

pub fn strip_tags(html: &str) -> String {
    let builder: Lazy<Builder> = Lazy::new(|| {
        let mut b = Builder::new();
        b.tags(HashSet::new());
        b
    });
    builder.clean(html).to_string()
}
