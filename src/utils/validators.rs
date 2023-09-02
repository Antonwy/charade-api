pub fn valid_alphanumeric_name(value: &str, validation_obj_name: &str) -> Option<(String, String)> {
    for c in value.chars() {
        if c == '-' || c == '_' {
            continue;
        }

        if c.is_whitespace() {
            return Some((
                format!("{} must not contain any whitespace", validation_obj_name),
                "wrong_format".to_string(),
            ));
        }

        if !c.is_ascii_alphanumeric() {
            return Some((
                format!(
                    "{} must only contain alphanumeric characters or -/_",
                    validation_obj_name
                ),
                "wrong_format".to_string(),
            ));
        }

        if c.is_uppercase() {
            return Some((
                format!(
                    "{} must only contain lowercase characters",
                    validation_obj_name
                ),
                "wrong_format".to_string(),
            ));
        }
    }

    None
}
