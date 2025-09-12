use anyhow::Context;
use std::env;
use std::sync::Arc;
use todo_api::{create_app_router, db, service, AppState};
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().json().init();

    println!("[PRINTLN] --- NÜKLEER TEST BAŞLADI (vFinal) ---");
    info!("[TRACING] --- NÜKLEER TEST BAŞLADI (vFinal) ---");

    let hardcoded_jwt_secret = "A1B2C3D4E5F6G7H8A1B2C3D4E5F6G7H8A1B2C3D4E5F6G7H8";
    println!(
        "[PRINTLN] Hard-coded JWT Secret kullanılıyor. Uzunluk: {}",
        hardcoded_jwt_secret.len()
    );
    info!(
        "[TRACING] Hard-coded JWT Secret kullanılıyor. Uzunluk: {}",
        hardcoded_jwt_secret.len()
    );

    println!("[PRINTLN] Diğer secret'lar ortamdan okunuyor...");
    info!("[TRACING] Diğer secret'lar ortamdan okunuyor...");
    let database_url =
        env::var("DATABASE_URL").context("DATABASE_URL çevre değişkeni bulunamadı!")?;
    let hashing_secret_key =
        env::var("HASHING_SECRET_KEY").context("HASHING_SECRET_KEY çevre değişkeni bulunamadı!")?;
    println!("[PRINTLN] Diğer secret'lar başarıyla okundu.");
    info!("[TRACING] Diğer secret'lar başarıyla okundu.");

    println!("[PRINTLN] Veritabanı bağlantısı kuruluyor...");
    info!("[TRACING] Veritabanı bağlantısı kuruluyor...");
    let db_pool = db::connection_pool(&database_url).await?;
    db::schema::initialize_schema(&db_pool).await?;
    println!("[PRINTLN] Veritabanı bağlantısı başarılı.");
    info!("[TRACING] Veritabanı bağlantısı başarılı.");

    let todo_service = Arc::new(service::todo::Service::new(db_pool.clone())?);

    let jwt_service = Arc::new(service::jwt::Service::new(hardcoded_jwt_secret)?);

    let auth_service = Arc::new(service::auth::Service::new(
        jwt_service.clone(),
        db_pool.clone(),
        hashing_secret_key,
    )?);

    let social_service = Arc::new(service::social::SocialService::new(db_pool.clone()));

    println!("[PRINTLN] Tüm servisler başarıyla oluşturuldu.");
    info!("[TRACING] Tüm servisler başarıyla oluşturuldu.");

    let app_state = AppState {
        todo_service,
        auth_service,
        jwt_service: jwt_service.clone(),
        social_service,
    };

    let router = create_app_router(app_state);

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let host = "0.0.0.0";
    let addr_str = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr_str).await?;

    println!("[PRINTLN] Sunucu {} adresinde dinleniyor...", addr_str);
    info!("[TRACING] Sunucu {} adresinde dinleniyor...", addr_str);

    axum::serve(listener, router).await?;

    Ok(())
}
