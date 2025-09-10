use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};

use crate::{
    handlers::{
        models::{ErrorResponse, JsonResponse},
        todo::models::Todo,
    },
    service::jwt::ContextUser,
    AppState,
};

pub async fn handler(
    State(AppState { todo_service, .. }): State<AppState>,
    Extension(user): Extension<ContextUser>,
) -> impl IntoResponse {
    match todo_service.list(user.user_id).await {
        Ok(result) => (
            StatusCode::OK,
            Json(JsonResponse::Success(
                result
                    .iter()
                    .map(|value| value.into())
                    .collect::<Vec<Todo>>(),
            )),
        ),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(JsonResponse::Error(ErrorResponse::from_error(error))),
        ),
    }
}
