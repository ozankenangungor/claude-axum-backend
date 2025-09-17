use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

/// Neon Serverless PostgreSQL için optimize edilmiş bağlantı ayarları
/// Neon'un serverless yapısına uygun olarak tasarlandı
pub struct NeonConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
    pub statement_timeout: Duration,
}

impl Default for NeonConfig {
    fn default() -> Self {
        Self {
            // Neon serverless için optimum değerler
            max_connections: 15, // Neon'un connection limitine uygun
            min_connections: 2,  // Serverless için minimum
            acquire_timeout: Duration::from_secs(10), // Neon cold start için yeterli
            idle_timeout: Duration::from_secs(180), // 3 dakika - Neon'da connection pooling için ideal
            max_lifetime: Duration::from_secs(900), // 15 dakika - Neon serverless döngüsü
            statement_timeout: Duration::from_secs(30), // Query timeout
        }
    }
}

/// Neon Serverless PostgreSQL için özel connection pool oluşturma
pub async fn create_neon_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let config = NeonConfig::default();

    tracing::info!("Creating Neon-optimized database connection pool");
    tracing::info!(
        "Max connections: {}, Min connections: {}",
        config.max_connections,
        config.min_connections
    );

    let pool = PgPoolOptions::new()
        // Neon Serverless için optimize edilmiş ayarlar
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        // Timeout ayarları - Neon'un cold start süresini göz önünde bulundurur
        .acquire_timeout(config.acquire_timeout)
        .idle_timeout(config.idle_timeout)
        .max_lifetime(config.max_lifetime)
        // Neon için önemli: Connection'ları test et
        .test_before_acquire(true)
        // Neon SSL desteği
        .connect_lazy(database_url)?;

    // Neon health check with specific timeout
    test_neon_connection(&pool).await?;

    tracing::info!("Neon database connection pool ready");
    Ok(pool)
}

/// Neon'a özel sağlık kontrolü
/// Cold start durumlarını handle eder
async fn test_neon_connection(pool: &PgPool) -> Result<(), sqlx::Error> {
    use tokio::time::timeout;

    let health_check = async {
        sqlx::query("SELECT 1 as neon_health_check, NOW() as timestamp")
            .fetch_one(pool)
            .await
    };

    // Neon cold start için 15 saniye timeout
    match timeout(Duration::from_secs(15), health_check).await {
        Ok(result) => {
            result?;
            tracing::info!("Neon database health check passed");
            Ok(())
        }
        Err(_) => {
            tracing::error!("Neon database health check timed out (cold start suspected)");
            Err(sqlx::Error::Configuration("Neon connection timeout".into()))
        }
    }
}

/// Neon-specific connection string builder
pub fn build_neon_connection_string(
    host: &str,
    database: &str,
    username: &str,
    password: &str,
    options: Option<&str>,
) -> String {
    let base_url = format!("postgres://{}:{}@{}/{}", username, password, host, database);

    // Neon için optimize edilmiş connection parametreleri
    let neon_params = "?sslmode=require&connect_timeout=10&statement_timeout=30000&idle_in_transaction_session_timeout=60000";

    match options {
        Some(opts) if !opts.is_empty() => format!("{}&{}", base_url, opts),
        _ => format!("{}{}", base_url, neon_params),
    }
}

/// Neon branching desteği için utility fonksiyonlar
#[derive(Debug, Clone)]
pub struct NeonBranch {
    pub name: String,
    pub host: String,
    pub is_primary: bool,
}

impl NeonBranch {
    pub fn primary(host: String) -> Self {
        Self {
            name: "main".to_string(),
            host,
            is_primary: true,
        }
    }

    pub fn preview(name: String, host: String) -> Self {
        Self {
            name,
            host,
            is_primary: false,
        }
    }
}

/// Environment-aware Neon configuration
pub fn get_neon_config_for_env() -> NeonConfig {
    match std::env::var("RUST_ENV").as_deref() {
        Ok("production") => NeonConfig {
            max_connections: 20, // Production için daha yüksek
            min_connections: 5,
            acquire_timeout: Duration::from_secs(8),
            idle_timeout: Duration::from_secs(300),
            max_lifetime: Duration::from_secs(1800),
            statement_timeout: Duration::from_secs(30),
        },
        Ok("staging") => NeonConfig {
            max_connections: 10,
            min_connections: 2,
            acquire_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(180),
            max_lifetime: Duration::from_secs(600),
            statement_timeout: Duration::from_secs(30),
        },
        _ => NeonConfig::default(), // Development
    }
}
