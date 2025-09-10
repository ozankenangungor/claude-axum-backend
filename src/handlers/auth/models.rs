use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use validator::Validate;

static VALID_USERNAME: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r"^[a-zA-Z0-9_]+$").unwrap());

fn validate_username(username: &str) -> Result<(), validator::ValidationError> {
    if !VALID_USERNAME.is_match(username) {
        return Err(validator::ValidationError::new("invalid_username"));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegistrationRequest {
    #[validate(length(
        min = 3,
        max = 50,
        message = "Username must be between 3 and 50 characters"
    ))]
    #[validate(custom(function = "validate_username"))]
    pub username: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1, message = "Username is required"))]
    pub username: String,
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
}
