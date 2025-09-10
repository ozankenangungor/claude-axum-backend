use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};

use crate::{
    handlers::{
        models::{ErrorResponse, JsonResponse},
        todo::models::Todo,
    },
    service::{self, jwt::ContextUser},
    AppState,
};

pub async fn handler(
    State(AppState { todo_service, .. }): State<AppState>,
    Extension(user): Extension<ContextUser>,
    Path(id): Path<u64>,
) -> impl IntoResponse {
    println!("a");
    tracing::info!("TODO'yu getiriliyor: {}", id);
    match todo_service.get(user.user_id as i32, id as i32).await {
        Ok(result) => (
            StatusCode::OK,
            Json(JsonResponse::Success(Todo::from(result))),
        ),
        Err(error) => {
            if matches!(error, service::todo::Error::TodoNotFound) {
                return (
                    StatusCode::NOT_FOUND,
                    Json(JsonResponse::Error(ErrorResponse::from_str(
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
