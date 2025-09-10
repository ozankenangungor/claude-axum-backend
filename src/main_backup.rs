use std::sync::Arc;
use todo_api::{config::Config, create_app_router, db, service, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Cloud Run için JSON formatında loglama başlat
    tracing_subscriber::fmt()
        .json() // JSON formatında loglama için
        .init();

    tracing::info!("Yapılandırma yükleniyor...");
    // Load configuration
    let config = Config::from_env()?;

    // Initialize database connection pool
    let db_pool = db::connection_pool().await?;

    // Schema initialization - manuel schema kurulumu yapıyoruz
    tracing::info!("Veritabanı şeması başlatılıyor...");
    db::schema::initialize_schema(&db_pool).await?;
    tracing::info!("Veritabanı şeması başarıyla başlatıldı.");

    // Initialize all services with single connection pool instance
    let todo_service = Arc::new(service::todo::Service::new(db_pool.clone())?);
    let jwt_service = Arc::new(service::jwt::Service::new(&config.jwt_secret)?);
    let auth_service = Arc::new(service::auth::Service::new(
        jwt_service.clone(),
        db_pool.clone(),
        config.hashing_secret_key.clone(),
    )?);

    // Create application state
    let app_state = AppState {
        todo_service,
        auth_service,
        jwt_service: jwt_service.clone(),
    };

    // Create router
    let router = create_app_router(app_state);

    // Cloud Run için portu ortam değişkeninden oku, yoksa config'den al
    let port = std::env::var("PORT").unwrap_or_else(|_| config.server_port.to_string());
    // Cloud Run için host her zaman 0.0.0.0 olmalı
    let host = "0.0.0.0";

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;
    tracing::info!("Sunucu {}:{} adresinde dinlemede...", host, port);
    axum::serve(listener, router).await?;
    Ok(())
}
