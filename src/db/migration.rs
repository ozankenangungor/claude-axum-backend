use anyhow::anyhow;
use sqlx::{migrate::MigrateDatabase, Postgres};

#[allow(dead_code)] // Reserved for future SQLx migration usage
pub async fn migrate_db() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")?;

    // Create database if it doesn't exist
    if !Postgres::database_exists(&database_url)
        .await
        .unwrap_or(false)
    {
        Postgres::create_database(&database_url).await?;
    }

    let pool = super::connection_pool(&database_url).await?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|error| anyhow!("Failed to run DB migrations: {error}"))?;

    Ok(())
}
