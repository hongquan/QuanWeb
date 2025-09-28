use std::collections::HashSet;
use std::sync::LazyLock;

use ammonia::Builder;

pub fn strip_tags(html: &str) -> String {
    let builder: LazyLock<Builder> = LazyLock::new(|| {
        let mut b = Builder::new();
        b.tags(HashSet::new());
        b
    });
    builder.clean(html).to_string()
}
