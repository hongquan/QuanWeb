use garde::Validate;
use redact::Secret;
use serde::Deserialize;

#[derive(Debug, Deserialize, Validate)]
pub struct LoginReqData {
    #[garde(email)]
    pub email: String,
    #[garde(custom(has_some_chars))]
    pub password: Secret<String>,
}

fn has_some_chars(value: &Secret<String>, _ctx: &()) -> garde::Result {
    if value.expose_secret().len() >= 1 {
        return Ok(());
    }
    Err(garde::Error::new("Too short"))
}
