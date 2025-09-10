use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};

use crate::{
    db::models::Like,
    handlers::models::Claims,
    AppState,
};

pub async fn like_post(
    State(app_state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(post_id): Path<i32>,
) -> Result<(StatusCode, Json<Like>), StatusCode> {
    match app_state.social_service.like_post(claims.sub, post_id).await {
        Ok(like) => Ok((StatusCode::CREATED, Json(like))),
        Err(e) => {
            eprintln!("Failed to like post: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn unlike_post(
    State(app_state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(post_id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    match app_state.social_service.unlike_post(claims.sub, post_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to unlike post: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn check_liked(
    State(app_state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(post_id): Path<i32>,
) -> Result<Json<bool>, StatusCode> {
    match app_state.social_service.is_liked(claims.sub, post_id).await {
        Ok(is_liked) => Ok(Json(is_liked)),
        Err(e) => {
            eprintln!("Failed to check like status: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}