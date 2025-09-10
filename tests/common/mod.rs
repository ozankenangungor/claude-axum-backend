use todo_api::AppState;
use anyhow::Result;

/// Test modules
pub mod database;
pub mod fixtures;
pub mod auth_helpers;
pub mod test_client;

/// Test configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub hashing_secret_key: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/todo_db".to_string()),
            jwt_secret: "test_jwt_secret_12345678901234567890123456789012345".to_string(),
            hashing_secret_key: "test_hashing_secret_key_1234567890123456789012345".to_string(),
        }
    }
}

/// Main test context for integration tests
pub struct TestContext {
    pub config: TestConfig,
    pub app_state: AppState,
    pub db_pool: sqlx::PgPool,
}

impl TestContext {
    /// Setup test context with database and services
    pub async fn new() -> Result<Self> {
        // Load environment variables
        dotenvy::dotenv().ok();
        
        let config = TestConfig::default();
        
        // Setup database
        let db_pool = database::TestDatabase::setup().await?;
        
        // Clean any existing test data
        database::TestDatabase::cleanup(&db_pool).await?;
        
        // Create JWT service
        let jwt_service = std::sync::Arc::new(
            todo_api::service::jwt::Service::new(&config.jwt_secret)?
        );
        
        // Create auth service
        let auth_service = std::sync::Arc::new(
            todo_api::service::auth::Service::new(
                jwt_service.clone(),
                db_pool.clone(), 
                config.hashing_secret_key.clone(),
            )?
        );
        
        // Create todo service
        let todo_service = std::sync::Arc::new(
            todo_api::service::todo::Service::new(db_pool.clone())?
        );
        
        let app_state = AppState {
            jwt_service,
            auth_service,
            todo_service,
        };
        
        Ok(Self {
            config,
            app_state,
            db_pool,
        })
    }

    /// Clean up test data
    pub async fn cleanup(&self) -> Result<()> {
        database::TestDatabase::cleanup(&self.db_pool).await
    }
}

/// Test result type for consistent error handling
pub type TestResult<T = ()> = Result<T>;