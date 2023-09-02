use std::borrow::Cow;

use serde::Deserialize;
use validator::{Validate, ValidationError};

use crate::utils::validators::valid_alphanumeric_name;

#[derive(Debug, Deserialize, Validate)]
pub struct NewSessionDto {
    #[validate(
        length(min = 1, max = 30, message = "Wrong id length", code = "wrong_length"),
        custom = "validate_session_id"
    )]
    pub id: String,
    #[serde(default)]
    pub public: Option<bool>,
}

fn validate_session_id(session_id: &str) -> Result<(), ValidationError> {
    let session_id = session_id.trim();

    let validation_error = |msg: &str, code: &str| ValidationError {
        message: Some(Cow::from(msg.to_owned())),
        code: Cow::from(code.to_owned()),
        params: std::collections::HashMap::new(),
    };

    if let Some((msg, code)) = valid_alphanumeric_name(session_id, "Session id") {
        return Err(validation_error(&msg, &code));
    }

    Ok(())
}
