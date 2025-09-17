use sqlx::PgPool;
use std::env::VarError;
use thiserror::Error;

pub mod migration;
pub mod models;
pub mod neon_config;
pub mod schema;

#[derive(Error, Debug)]
pub enum DbConnectionPoolError {
    #[error("Missing environment variable: {0}")]
    EnvVar(#[from] VarError),

    #[error("SQLx pool error: {0}")]
    SqlxPool(#[from] sqlx::Error),
}

/// Getting an optimized connection pool for database (PostgreSQL)
/// Auto-detects Neon and applies serverless-optimized settings
pub async fn connection_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;

    // Detect if we're using Neon based on the connection string
    let is_neon = database_url.contains("neon.tech")
        || database_url.contains("neon.")
        || std::env::var("NEON_BRANCH_NAME").is_ok();

    if is_neon {
        tracing::info!("Detected Neon PostgreSQL - using serverless-optimized settings");
        return neon_config::create_neon_pool(database_url).await;
    }

    // Fallback to standard PostgreSQL configuration
    println!("Creating optimized database connection pool...");
    tracing::info!("Initializing database connection pool with production settings");

    let pool = PgPoolOptions::new()
        // Connection pool settings for production
        .max_connections(20) // Maximum number of connections in the pool
        .min_connections(5) // Minimum number of connections to maintain
        .acquire_timeout(Duration::from_secs(8)) // Maximum time to wait for a connection
        .idle_timeout(Duration::from_secs(300)) // Close connections idle for 5 minutes
        .max_lifetime(Duration::from_secs(1800)) // Close connections after 30 minutes
        // Connection testing
        .test_before_acquire(true) // Test connections before using them
        .connect(database_url)
        .await?;

    // Test the connection with a simple query
    sqlx::query("SELECT 1").execute(&pool).await.map_err(|e| {
        tracing::error!("Database health check failed: {}", e);
        e
    })?;

    tracing::info!("Database connection pool initialized successfully");
    println!("Database connection pool ready with {} max connections", 20);

    Ok(pool)
}

/// Health check for database connection
pub async fn health_check(pool: &PgPool) -> Result<(), sqlx::Error> {
    let start = std::time::Instant::now();

    sqlx::query("SELECT 1 as health_check")
        .fetch_one(pool)
        .await?;

    let duration = start.elapsed();
    tracing::info!("Database health check passed in {:?}", duration);

    // Log the operation
    crate::monitoring::log_db_query("health_check", "system", duration, true);

    Ok(())
}
