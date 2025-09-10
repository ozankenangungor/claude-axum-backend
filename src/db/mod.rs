use sqlx::PgPool;
use std::env::VarError;
use thiserror::Error;

pub mod migration;
pub mod models;
pub mod schema;

#[derive(Error, Debug)]
pub enum DbConnectionPoolError {
    #[error("Missing environment variable: {0}")]
    EnvVar(#[from] VarError),

    #[error("SQLx pool error: {0}")]
    SqlxPool(#[from] sqlx::Error),
}

/// Getting a connection pool for database (PostgreSQL)
/// Now accepts database_url as parameter instead of reading from environment
pub async fn connection_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    println!("Attempting to connect to database...");

    PgPool::connect(database_url).await
}
