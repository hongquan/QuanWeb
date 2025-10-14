use std::collections::HashSet;
use std::sync::LazyLock;

use ammonia::Builder;
use minijinja::Environment;
use serde::ser::Serialize;

pub use crate::errors::PageError;

pub fn strip_tags(html: &str) -> String {
    let builder: LazyLock<Builder> = LazyLock::new(|| {
        let mut b = Builder::new();
        b.tags(HashSet::new());
        b
    });
    builder.clean(html).to_string()
}

pub fn render_with<S: Serialize>(
    template_name: &str,
    context: S,
    engine: Environment,
) -> Result<String, PageError> {
    let tpl = engine.get_template(template_name)?;
    let content = tpl.render(context)?;
    Ok(content)
}
