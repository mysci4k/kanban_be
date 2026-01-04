use validator::ValidationError;

pub fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !has_uppercase {
        return Err(ValidationError::new("password_uppercase")
            .with_message("Password must contain at least one uppercase letter".into()));
    }

    if !has_lowercase {
        return Err(ValidationError::new("password_lowercase")
            .with_message("Password must contain at least one lowercase letter".into()));
    }

    if !has_digit {
        return Err(ValidationError::new("password_digit")
            .with_message("Password must contain at least one digit".into()));
    }

    if !has_special {
        return Err(ValidationError::new("password_special")
            .with_message("Password must contain at least one special character".into()));
    }

    Ok(())
}
