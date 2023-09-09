use redact::{Secret, expose_secret};
use serde::{Serialize, Deserialize};
use validify::{ValidationError, ValidationErrors};


#[derive(Debug, Deserialize, Serialize)]
pub struct LoginReqData {
    pub email: String,
    #[serde(serialize_with = "expose_secret")]
    pub password: Secret<String>,
    pub remember_me: bool,
}

// The `#[derive(Validate)]` macro failed to generate code due to the type of `password` field.
// So we have to manually do it.
impl validify::Validate for LoginReqData {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        if !validify::validate_email(&self.email) {
            let mut err = ValidationError::new_field("email", "email");
            err.add_param("value", &&self.email);
            err.set_location("email");
            errors.add(err);
        }
        match validate_password(&self.password) {
            Ok(()) => (),
            Err(mut err) => {
                let field = err.field_name().unwrap().to_string();
                err.set_location(field);
                err.add_param("value", &"**redacted**");
                errors.add(err);
            }
        };
        errors.is_empty().then_some(()).ok_or(errors)
    }
}

pub fn validate_password(value: &Secret<String>) -> Result<(), ValidationError> {
    (value.expose_secret().len() >= 8).then_some(()).ok_or_else(|| {
        ValidationError::new_field("password", "too-short")
    })
}
