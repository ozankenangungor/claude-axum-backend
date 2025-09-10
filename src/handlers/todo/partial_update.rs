use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use validator::Validate;

use crate::{
    handlers::{
        models::{ErrorResponse, JsonResponse},
        todo::models::PartialUpdateTodoRequest,
    },
    service::{self, jwt::ContextUser},
    AppState,
};

pub async fn handler(
    State(AppState { todo_service, .. }): State<AppState>,
    Extension(user): Extension<ContextUser>,
    Path(id): Path<u64>,
    Json(request): Json<PartialUpdateTodoRequest>,
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

    match todo_service
        .partial_update(user.user_id, id as i32, request.into())
        .await
    {
        Ok(_) => (StatusCode::OK, Json(JsonResponse::Success(true))),
        Err(error) => {
            if matches!(error, service::todo::Error::TodoNotFound) {
                return (
                    StatusCode::NOT_FOUND,
                    Json(JsonResponse::Error(ErrorResponse::new_from_str(
                        "TODO not found!",
                    ))),
                );
            }
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JsonResponse::Error(ErrorResponse::from_error(error))),
            )
        }
    }
}
