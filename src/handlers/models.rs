use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JsonResponse<T: Serialize> {
    Success(T),
    Error(ErrorResponse),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub message: String,
}

impl ErrorResponse {
    pub fn from_error(error: impl std::error::Error) -> Self {
        Self {
            message: error.to_string(),
        }
    }

    pub fn new_from_str(error: &str) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

// JWT Claims struct for authentication
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i32, // user_id
    pub username: String,
    pub exp: usize,
}
