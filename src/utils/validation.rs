
use redact::Secret;

pub fn has_some_chars(value: &Secret<String>, _ctx: &()) -> garde::Result {
    if value.expose_secret().len() >= 1 {
        return Ok(());
    }
    Err(garde::Error::new("Too short"))
}
