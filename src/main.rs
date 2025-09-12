// src/main.rs (GEÇİCİ VE SON TEŞHİS İÇİN)

use anyhow::Context;
use std::env;
use std::sync::Arc;
use todo_api::{create_app_router, db, service, AppState};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().json().init();
    info!("--- NÜKLEER TEST BAŞLADI ---");

    // 1. JWT SECRET'I DOĞRUDAN KODA YAZIYORUZ. ARTIK HİÇBİR DIŞ ETKEN YOK.
    let hardcoded_jwt_secret = "A1B2C3D4E5F6G7H8A1B2C3D4E5F6G7H8A1B2C3D4E5F6G7H8";
    info!(
        "Hard-coded JWT Secret kullanılıyor. Uzunluk: {}",
        hardcoded_jwt_secret.len()
    );

    // 2. DİĞER SECRET'LARI DOĞRUDAN ORTAMDAN OKUYORUZ (CONFIG'İ BYPASS EDİYORUZ)
    // Not: Bu değişkenler zaten multi-env-deploy.yml tarafından Secret Manager'dan alınıp ortama set ediliyor.
    let database_url =
        env::var("DATABASE_URL").context("DATABASE_URL çevre değişkeni bulunamadı!")?;
    let hashing_secret_key =
        env::var("HASHING_SECRET_KEY").context("HASHING_SECRET_KEY çevre değişkeni bulunamadı!")?;

    // 3. SERVİSLERİ OLUŞTURUYORUZ
    info!("Veritabanı bağlantısı kuruluyor...");
    let db_pool = db::connection_pool(&database_url).await?;
    db::schema::initialize_schema(&db_pool).await?;
    info!("Veritabanı bağlantısı başarılı.");

    let todo_service = Arc::new(service::todo::Service::new(db_pool.clone())?);

    // JWT Servisini sabit kodlanmış secret ile oluşturuyoruz.
    let jwt_service = Arc::new(service::jwt::Service::new(hardcoded_jwt_secret)?);

    let auth_service = Arc::new(service::auth::Service::new(
        jwt_service.clone(),
        db_pool.clone(),
        hashing_secret_key,
    )?);

    let social_service = Arc::new(service::social::SocialService::new(db_pool.clone()));

    info!("Tüm servisler başarıyla oluşturuldu.");

    // 4. UYGULAMAYI BAŞLATIYORUZ
    let app_state = AppState {
        todo_service,
        auth_service,
        jwt_service: jwt_service.clone(),
        social_service,
    };

    let router = create_app_router(app_state);

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let host = "0.0.0.0";

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;
    info!("Sunucu {}:{} adresinde dinleniyor...", host, port);
    axum::serve(listener, router).await?;
    Ok(())
}
