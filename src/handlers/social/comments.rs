use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::Deserialize;

use crate::{
    db::models::{Comment, CreateComment, UpdateComment},
    handlers::models::Claims,
    AppState,
};

#[derive(Deserialize)]
pub struct CommentQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

pub async fn create_comment(
    State(app_state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(create_comment): Json<CreateComment>,
) -> Result<(StatusCode, Json<Comment>), StatusCode> {
    match app_state.social_service.create_comment(claims.sub, create_comment).await {
        Ok(comment) => Ok((StatusCode::CREATED, Json(comment))),
        Err(e) => {
            eprintln!("Failed to create comment: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_post_comments(
    State(app_state): State<AppState>,
    Path(post_id): Path<i32>,
    Query(query): Query<CommentQuery>,
) -> Result<Json<Vec<Comment>>, StatusCode> {
    match app_state.social_service.get_post_comments(post_id, query.limit, query.offset).await {
        Ok(comments) => Ok(Json(comments)),
        Err(e) => {
            eprintln!("Failed to get comments: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_comment(
    State(app_state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(comment_id): Path<i32>,
    Json(update_comment): Json<UpdateComment>,
) -> Result<Json<Comment>, StatusCode> {
    match app_state.social_service.update_comment(comment_id, claims.sub, update_comment).await {
        Ok(Some(comment)) => Ok(Json(comment)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to update comment: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_comment(
    State(app_state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(comment_id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    match app_state.social_service.delete_comment(comment_id, claims.sub).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to delete comment: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}