use serde::Deserialize;
use garde::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct LoginReqData {
    #[garde(email)]
    pub email: String,
    #[garde(length(min=1))]
    pub password: String,
}
