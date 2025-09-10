use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::Deserialize;

use crate::{
    db::models::{Follow, UserProfile},
    handlers::models::Claims,
    AppState,
};

#[derive(Deserialize)]
pub struct FollowQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

pub async fn follow_user(
    State(app_state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(following_id): Path<i32>,
) -> Result<(StatusCode, Json<Follow>), StatusCode> {
    // Check if trying to follow themselves
    if claims.sub == following_id {
        return Err(StatusCode::BAD_REQUEST);
    }

    match app_state
        .social_service
        .follow_user(claims.sub, following_id)
        .await
    {
        Ok(follow) => Ok((StatusCode::CREATED, Json(follow))),
        Err(e) => {
            eprintln!("Failed to follow user: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn unfollow_user(
    State(app_state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(following_id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    match app_state
        .social_service
        .unfollow_user(claims.sub, following_id)
        .await
    {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to unfollow user: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn check_following(
    State(app_state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(following_id): Path<i32>,
) -> Result<Json<bool>, StatusCode> {
    match app_state
        .social_service
        .is_following(claims.sub, following_id)
        .await
    {
        Ok(is_following) => Ok(Json(is_following)),
        Err(e) => {
            eprintln!("Failed to check following status: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_followers(
    State(app_state): State<AppState>,
    Path(user_id): Path<i32>,
    Query(query): Query<FollowQuery>,
) -> Result<Json<Vec<UserProfile>>, StatusCode> {
    match app_state
        .social_service
        .get_followers(user_id, query.limit, query.offset)
        .await
    {
        Ok(followers) => Ok(Json(followers)),
        Err(e) => {
            eprintln!("Failed to get followers: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_following(
    State(app_state): State<AppState>,
    Path(user_id): Path<i32>,
    Query(query): Query<FollowQuery>,
) -> Result<Json<Vec<UserProfile>>, StatusCode> {
    match app_state
        .social_service
        .get_following(user_id, query.limit, query.offset)
        .await
    {
        Ok(following) => Ok(Json(following)),
        Err(e) => {
            eprintln!("Failed to get following: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
