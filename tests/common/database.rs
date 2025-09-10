use anyhow::Result;
use std::env;

/// Test database utilities
pub struct TestDatabase;

impl TestDatabase {
    /// Setup test database connection
    pub async fn setup() -> Result<sqlx::PgPool> {
        // For now, use the existing database connection
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/todo_db".to_string());
        
        let pool = sqlx::PgPool::connect(&database_url).await?;
        
        Ok(pool)
    }

    /// Clean up test data
    pub async fn cleanup(pool: &sqlx::PgPool) -> Result<()> {
        // Clean up test data
        sqlx::query!("DELETE FROM todos WHERE user_id IN (SELECT id FROM users WHERE username LIKE 'test_%')")
            .execute(pool)
            .await?;
            
        sqlx::query!("DELETE FROM users WHERE username LIKE 'test_%'")
            .execute(pool)
            .await?;
            
        Ok(())
    }

    /// Seed test data
    pub async fn seed_test_data(pool: &sqlx::PgPool) -> Result<()> {
        // This can be expanded as needed
        Ok(())
    }
}

/// Test transaction helper
pub struct TestTransaction<'a> {
    pub tx: sqlx::Transaction<'a, sqlx::Postgres>,
}

impl<'a> TestTransaction<'a> {
    pub async fn new(pool: &'a sqlx::PgPool) -> Result<Self> {
        let tx = pool.begin().await?;
        Ok(Self { tx })
    }

    pub async fn rollback(self) -> Result<()> {
        self.tx.rollback().await?;
        Ok(())
    }
}