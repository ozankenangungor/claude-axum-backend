use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

pub async fn handler() -> impl IntoResponse {
    (StatusCode::OK, "VERSION 3 - DEPLOYMENT BAŞARILI")
}
