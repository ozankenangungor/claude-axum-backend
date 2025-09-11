use axum::{http::StatusCode, response::IntoResponse};

pub async fn handler() -> impl IntoResponse {
    (StatusCode::OK, "VERSION 3 - DEPLOYMENT BAŞARILI")
}
