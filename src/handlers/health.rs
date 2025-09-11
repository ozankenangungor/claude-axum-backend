use axum::{http::StatusCode, response::IntoResponse, Json};

pub async fn handler() -> impl IntoResponse {
    (StatusCode::OK, "VERSION 3 - DEPLOYMENT BAŞARILI")
}
