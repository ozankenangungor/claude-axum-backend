use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::Deserialize;

use crate::{
    db::models::{CreatePost, Post, UpdatePost},
    handlers::models::Claims,
    AppState,
};

#[derive(Deserialize)]
pub struct PostQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

pub async fn create_post(
    State(app_state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(create_post): Json<CreatePost>,
) -> Result<(StatusCode, Json<Post>), StatusCode> {
    match app_state
        .social_service
        .create_post(claims.sub, create_post)
        .await
    {
        Ok(post) => Ok((StatusCode::CREATED, Json(post))),
        Err(e) => {
            eprintln!("Failed to create post: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_post(
    State(app_state): State<AppState>,
    Path(post_id): Path<i32>,
) -> Result<Json<Post>, StatusCode> {
    match app_state.social_service.get_post(post_id).await {
        Ok(Some(post)) => Ok(Json(post)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to get post: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_user_posts(
    State(app_state): State<AppState>,
    Path(user_id): Path<i32>,
    Query(query): Query<PostQuery>,
) -> Result<Json<Vec<Post>>, StatusCode> {
    match app_state
        .social_service
        .get_user_posts(user_id, query.limit, query.offset)
        .await
    {
        Ok(posts) => Ok(Json(posts)),
        Err(e) => {
            eprintln!("Failed to get user posts: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_feed(
    State(app_state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<PostQuery>,
) -> Result<Json<Vec<Post>>, StatusCode> {
    match app_state
        .social_service
        .get_feed_posts(claims.sub, query.limit, query.offset)
        .await
    {
        Ok(posts) => Ok(Json(posts)),
        Err(e) => {
            eprintln!("Failed to get feed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_post(
    State(app_state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(post_id): Path<i32>,
    Json(update_post): Json<UpdatePost>,
) -> Result<Json<Post>, StatusCode> {
    match app_state
        .social_service
        .update_post(post_id, claims.sub, update_post)
        .await
    {
        Ok(Some(post)) => Ok(Json(post)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to update post: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_post(
    State(app_state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(post_id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    match app_state
        .social_service
        .delete_post(post_id, claims.sub)
        .await
    {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to delete post: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
