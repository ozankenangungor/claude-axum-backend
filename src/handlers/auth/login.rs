use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use validator::Validate;

use crate::{
    handlers::{auth::models::{LoginRequest, LoginResponse}, models::{ErrorResponse, JsonResponse}},
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
            Json(JsonResponse::Error(ErrorResponse::from_str(&format!(
                "Validation error: {}",
                validation_errors
            )))),
        );
    }

    match auth_service.login(request).await {
        Ok(token) => (StatusCode::OK, Json(JsonResponse::Success(LoginResponse { token }))),
        Err(error) => {
            if matches!(error, Error::InvalidPassword) {
                return (StatusCode::UNAUTHORIZED, Json(JsonResponse::Error(ErrorResponse::from_error(error))));
            }

            if matches!(error, Error::UserNotFound) {
                return (StatusCode::UNAUTHORIZED, Json(JsonResponse::Error(ErrorResponse::from_str("Invalid username!"))));
            }

            (StatusCode::INTERNAL_SERVER_ERROR, Json(JsonResponse::Error(ErrorResponse::from_error(error))))
        },
    }
}
