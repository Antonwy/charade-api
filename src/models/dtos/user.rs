use std::borrow::Cow;

use serde::Deserialize;
use validator::{Validate, ValidationError};

use crate::utils::validators::valid_alphanumeric_name;

#[derive(Deserialize, Validate)]
pub struct NewUserDto {
    #[validate(
        length(min = 5, max = 20, message = "Wrong name length"),
        custom = "validate_username"
    )]
    pub name: Option<String>,
}

fn validate_username(session_id: &str) -> Result<(), ValidationError> {
    let session_id = session_id.trim();

    let validation_error = |msg: &str, code: &str| ValidationError {
        message: Some(Cow::from(msg.to_owned())),
        code: Cow::from(code.to_owned()),
        params: std::collections::HashMap::new(),
    };

    if let Some((msg, code)) = valid_alphanumeric_name(session_id, "Username") {
        return Err(validation_error(&msg, &code));
    }

    Ok(())
}
