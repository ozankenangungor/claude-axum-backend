use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::Deserialize;

use crate::{
    db::models::{UpdateUserProfile, UserProfile},
    handlers::models::Claims,
    AppState,
};

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

pub async fn get_profile(
    State(app_state): State<AppState>,
    Path(user_id): Path<i32>,
) -> Result<Json<UserProfile>, StatusCode> {
    match app_state.social_service.get_user_profile(user_id).await {
        Ok(Some(profile)) => Ok(Json(profile)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to get user profile: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_my_profile(
    State(app_state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<UserProfile>, StatusCode> {
    match app_state.social_service.get_user_profile(claims.sub).await {
        Ok(Some(profile)) => Ok(Json(profile)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to get user profile: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_profile(
    State(app_state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(update_profile): Json<UpdateUserProfile>,
) -> Result<Json<UserProfile>, StatusCode> {
    match app_state.social_service.update_user_profile(claims.sub, update_profile).await {
        Ok(Some(profile)) => Ok(Json(profile)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Failed to update user profile: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn search_users(
    State(app_state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<UserProfile>>, StatusCode> {
    match app_state.social_service.search_users(&query.q, query.limit, query.offset).await {
        Ok(users) => Ok(Json(users)),
        Err(e) => {
            eprintln!("Failed to search users: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}