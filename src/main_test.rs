use axum::{response::Json, routing::get, Router};
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Cloud Run için JSON formatında loglama başlat
    tracing_subscriber::fmt()
        .json() // JSON formatında loglama için
        .init();

    tracing::info!("Basit test sunucusu başlatılıyor...");

    // Basit health check endpoint'i
    let app = Router::new().route("/health", get(health_check));

    // Cloud Run için portu ortam değişkeninden oku
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let host = "0.0.0.0";

    tracing::info!("Port ortam değişkeni: {}", port);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;
    tracing::info!("Test sunucusu {}:{} adresinde dinlemede...", host, port);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "message": "Test server is running"
    }))
}
