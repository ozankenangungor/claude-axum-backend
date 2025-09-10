use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use validator::Validate;

use crate::{handlers::{auth::models::RegistrationRequest, models::{ErrorResponse, JsonResponse}}, AppState};

pub async fn handler(
    State(AppState { auth_service, .. }): State<AppState>,
    Json(request): Json<RegistrationRequest>,
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

    match auth_service.register(request).await {
        Ok(_) => (StatusCode::OK, Json(JsonResponse::Success(true))),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, Json(JsonResponse::Error(ErrorResponse::from_error(error)))),
    }
}
