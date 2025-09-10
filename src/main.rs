use anyhow::Context;
use std::sync::Arc;
use todo_api::{config::Config, create_app_router, db, service, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize structured logging (JSON format for Cloud Logging)
    tracing_subscriber::fmt()
        .json() // JSON formatında loglama için
        .init();

    tracing::info!("Starting application...");

    tracing::info!("Loading configuration from Google Secret Manager...");
    // --- The key change is here ---
    let config = Config::from_gcp_secrets().await?;
    // ----------------------------
    tracing::info!("Configuration successfully loaded.");

    tracing::info!("Initializing database connection pool for Neon DB...");
    // The connection_pool function now accepts the database_url as an argument.
    let db_pool = db::connection_pool(&config.database_url)
        .await
        .context("Failed to create database connection pool for Neon")?;
    tracing::info!("Database connection pool successfully created.");

    // Schema initialization - manuel schema kurulumu yapıyoruz
    tracing::info!("Initializing database schema...");
    db::schema::initialize_schema(&db_pool).await?;
    tracing::info!("Database schema successfully initialized.");

    // Initialize all services with single connection pool instance
    let todo_service = Arc::new(service::todo::Service::new(db_pool.clone())?);
    let jwt_service = Arc::new(service::jwt::Service::new(&config.jwt_secret)?);
    let auth_service = Arc::new(service::auth::Service::new(
        jwt_service.clone(),
        db_pool.clone(),
        config.hashing_secret_key.clone(),
    )?);
    let social_service = Arc::new(service::social::SocialService::new(db_pool.clone()));

    // Create application state
    let app_state = AppState {
        todo_service,
        auth_service,
        jwt_service: jwt_service.clone(),
        social_service,
    };

    // Create router
    let router = create_app_router(app_state);

    // Cloud Run için portu ortam değişkeninden oku, yoksa config'den al
    let port = std::env::var("PORT").unwrap_or_else(|_| config.server_port.to_string());
    // Cloud Run için host her zaman 0.0.0.0 olmalı
    let host = "0.0.0.0";

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;
    tracing::info!("Server starting to listen on {}:{}...", host, port);
    axum::serve(listener, router).await?;
    Ok(())
}
