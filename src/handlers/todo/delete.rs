use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};

use crate::{
    handlers::models::{ErrorResponse, JsonResponse},
    service::{self, jwt::ContextUser},
    AppState,
};

pub async fn handler(
    State(AppState { todo_service, .. }): State<AppState>,
    Extension(user): Extension<ContextUser>,
    Path(id): Path<u64>,
) -> impl IntoResponse {
    match todo_service.delete(user.user_id, id as i32).await {
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
