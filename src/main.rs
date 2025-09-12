// src/main.rs (GÜNCELLENMİŞ VE TEŞHİS KODU EKLENMİŞ HALİ)

use anyhow::Context;
use std::env; // <-- Bu satırı ekledik
use std::sync::Arc;
use todo_api::{config::Config, create_app_router, db, service, AppState};
use tracing::{error, info}; // <-- error ve info'yu ekledik

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Loglamayı başlat
    tracing_subscriber::fmt().json().init();

    info!("Uygulama başlatılıyor...");

    // --- TEŞHİS KODU BAŞLANGICI ---
    // JWT_SECRET'ı çevre değişkenlerinden manuel olarak oku
    let jwt_secret_from_env = match env::var("JWT_SECRET") {
        Ok(secret) => {
            // Secret'ı logla (güvenlik için sadece bir kısmını)
            info!(
                "TEŞHİS: JWT_SECRET başarıyla okundu. Uzunluk: {}, Başlangıcı: '{}...'",
                secret.len(),
                secret.chars().take(4).collect::<String>()
            );
            secret
        }
        Err(e) => {
            // Eğer secret bulunamazsa, kritik bir hata logu bas ve çık
            error!("KRİTİK HATA: JWT_SECRET çevre değişkeni okunamadı! Hata: {:?}. Uygulama başlatılamıyor.", e);
            // Bu satırın production loglarında görünmesi, Secret Manager bağlantısının KESİNLİKLE çalışmadığını kanıtlar.
            std::process::exit(1);
        }
    };
    // --- TEŞHİS KODU BİTİŞİ ---

    info!("Genel yapılandırma (Config) yükleniyor...");
    let config = Config::from_gcp_secrets().await?;
    info!("Genel yapılandırma başarıyla yüklendi.");

    info!("Veritabanı bağlantı havuzu oluşturuluyor...");
    let db_pool = db::connection_pool(&config.database_url)
        .await
        .context("Veritabanı bağlantı havuzu oluşturulamadı")?;
    info!("Veritabanı bağlantı havuzu başarıyla oluşturuldu.");

    info!("Veritabanı şeması başlatılıyor...");
    db::schema::initialize_schema(&db_pool).await?;
    info!("Veritabanı şeması başarıyla başlatıldı.");

    // Servisleri oluştur
    let todo_service = Arc::new(service::todo::Service::new(db_pool.clone())?);

    // --- DEĞİŞTİRİLMİŞ JWT SERVİSİ OLUŞTURMA ---
    // JWT Servisini, config'den gelen yerine, manuel olarak okuduğumuz ve logladığımız secret ile oluştur.
    let jwt_service = Arc::new(
        service::jwt::Service::new(&jwt_secret_from_env)
            .context("TEŞHİS: Manuel olarak okunan JWT_SECRET ile JWT servisi oluşturulamadı!")?,
    );
    info!("JWT servisi başarıyla oluşturuldu.");
    // --- DEĞİŞİKLİK BİTİŞİ ---

    let auth_service = Arc::new(service::auth::Service::new(
        jwt_service.clone(),
        db_pool.clone(),
        config.hashing_secret_key.clone(),
    )?);
    let social_service = Arc::new(service::social::SocialService::new(db_pool.clone()));

    let app_state = AppState {
        todo_service,
        auth_service,
        jwt_service: jwt_service.clone(),
        social_service,
    };

    let router = create_app_router(app_state);

    let port = std::env::var("PORT").unwrap_or_else(|_| config.server_port.to_string());
    let host = "0.0.0.0";

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;
    info!("Sunucu {}:{} adresinde dinleniyor...", host, port);
    axum::serve(listener, router).await?;
    Ok(())
}
