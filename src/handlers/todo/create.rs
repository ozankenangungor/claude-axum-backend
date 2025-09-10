use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use validator::Validate;

use crate::{
    handlers::{
        models::{ErrorResponse, JsonResponse},
        todo::models::{CreateTodoRequest, Todo},
    },
    service::jwt::ContextUser,
    AppState,
};

pub async fn handler(
    State(AppState { todo_service, .. }): State<AppState>,
    Extension(user): Extension<ContextUser>,
    Json(request): Json<CreateTodoRequest>,
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

    match todo_service.create(user.user_id as i32, request).await {
        Ok(result) => (
            StatusCode::OK,
            Json(JsonResponse::Success(Todo::from(result))),
        ),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(JsonResponse::Error(ErrorResponse::from_error(error))),
        ),
    }
}
