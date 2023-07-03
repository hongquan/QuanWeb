use garde::Validate;
use redact::Secret;
use serde::Deserialize;

use crate::utils::validation::has_some_chars;

#[derive(Debug, Deserialize, Validate)]
pub struct LoginReqData {
    #[garde(email)]
    pub email: String,
    #[garde(custom(has_some_chars))]
    pub password: Secret<String>,
}
