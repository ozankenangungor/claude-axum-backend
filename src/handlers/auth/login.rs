use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use validator::Validate;

use crate::{
    handlers::{
        auth::models::{LoginRequest, LoginResponse},
        models::{ErrorResponse, JsonResponse},
    },
    service::auth::Error,
    AppState,
};

pub async fn handler(
    State(AppState { auth_service, .. }): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> impl IntoResponse {
    if let Err(validation_errors) = request.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(JsonResponse::Error(ErrorResponse::new_from_str(&format!(
                "Validation error: {}",
                validation_errors
            )))),
        );
    }

    match auth_service.login(request).await {
        Ok(token) => (
            StatusCode::OK,
            Json(JsonResponse::Success(LoginResponse { token })),
        ),
        Err(error) => {
            let message = match error {
                Error::InvalidPassword => "Invalid password".to_string(),
                Error::UserNotFound => "User not found".to_string(),
                Error::UsernameAlreadyExists(_)
                | Error::ConnectionPool(_)
                | Error::Sqlx(_)
                | Error::Hashing(_)
                | Error::EnvVar(_)
                | Error::JwtService(_)
                | Error::WeakPassword(_) => {
                    tracing::error!("Login error: {:?}", error);
                    "Internal server error".to_string()
                }
            };

            (
                StatusCode::BAD_REQUEST,
                Json(JsonResponse::Error(ErrorResponse::new_from_str(&message))),
            )
        }
    }
}
