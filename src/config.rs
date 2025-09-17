use anyhow::{Context, Result};
// use google_cloud_secretmanager_v1::{
//     client::SecretManagerService, model::AccessSecretVersionRequest,
// }; // Temporarily disabled due to edition2024 requirement

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub hashing_secret_key: String,
    pub jwt_secret: String,
    pub server_port: u16,
    pub server_host: String,
}

impl Config {
    /// Auto-detect environment and load configuration appropriately
    /// Currently uses environment variables (Secret Manager temporarily disabled)
    pub async fn auto_load() -> Result<Self> {
        tracing::info!("Loading configuration from environment variables");
        tracing::warn!("Google Secret Manager temporarily disabled due to edition2024 requirement");
        Self::from_env()
    }

    /// Load configuration from environment variables (development only)
    /// DİKKAT: Bu method sadece local development için kullanılmalı
    /// Production'da Google Secret Manager kullanılır
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok(); // Load .env file for local development

        tracing::warn!("Loading configuration from environment variables - development mode only!");
        tracing::warn!("Production deployment should use Google Secret Manager");

        let database_url = std::env::var("DATABASE_URL")
            .context("DATABASE_URL environment variable is required")?;

        let hashing_secret_key = std::env::var("HASHING_SECRET_KEY")
            .context("HASHING_SECRET_KEY environment variable is required")?;

        let jwt_secret =
            std::env::var("JWT_SECRET").context("JWT_SECRET environment variable is required")?;

        let server_port = std::env::var("PORT")
            .or_else(|_| std::env::var("SERVER_PORT"))
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .context("Invalid PORT/SERVER_PORT value")?;

        let server_host = std::env::var("HOST")
            .or_else(|_| std::env::var("SERVER_HOST"))
            .unwrap_or_else(|_| "0.0.0.0".to_string());

        // Validate required secrets
        if hashing_secret_key.len() < 16 {
            anyhow::bail!("HASHING_SECRET_KEY must be at least 16 characters");
        }

        if jwt_secret.len() < 32 {
            anyhow::bail!("JWT_SECRET must be at least 32 characters");
        }

        Ok(Config {
            database_url,
            hashing_secret_key,
            jwt_secret,
            server_port,
            server_host,
        })
    }

    // /// Asynchronously fetches application configuration from Google Secret Manager
    // /// using the official Google Cloud Rust SDK.
    // /// This is the recommended way to load secrets in a production environment.
    // /// KULLANIM: Google Cloud Run + Neon PostgreSQL production deployment için
    // /// TEMPORARILY DISABLED: Edition2024 requirement conflicts with current Rust version
    // pub async fn from_gcp_secrets() -> Result<Self> {
}

// /// Fetch a single secret from Google Secret Manager using the official SDK
// /// This is much cleaner and more reliable than manual HTTP requests
// /// TEMPORARILY DISABLED: Edition2024 requirement conflicts with current Rust version
// async fn fetch_secret_with_sdk(
//     client: &SecretManagerService,
//     project_id: &str,
//     secret_name: &str,
// ) -> Result<String> {
