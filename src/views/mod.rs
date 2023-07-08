pub mod front;
pub mod routes;
pub mod structs;
pub mod old_urls;

use serde::ser::Serialize;
use minijinja::Environment;

pub use front::{home, static_handler};
pub use crate::errors::PageError;

pub fn render_with<S: Serialize>(template_name: &str, context: S, engine: Environment) -> Result<String, PageError> {
    let tpl = engine.get_template(template_name)?;
    let content = tpl.render(context)?;
    Ok(content)
}
