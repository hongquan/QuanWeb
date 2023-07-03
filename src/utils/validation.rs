use std::collections::HashMap;

use redact::Secret;

pub fn flatten_garde_errors(errors: garde::Errors) -> HashMap<String, String> {
    errors
        .flatten()
        .into_iter()
        .map(|(k, v)| (k, v.message.to_string()))
        .collect()
}

pub fn has_some_chars(value: &Secret<String>, _ctx: &()) -> garde::Result {
    if value.expose_secret().len() >= 1 {
        return Ok(());
    }
    Err(garde::Error::new("Too short"))
}
