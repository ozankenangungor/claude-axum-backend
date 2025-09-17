use anyhow::{Context, Result};
use google_cloud_secretmanager_v1::{
    client::SecretManagerService, model::AccessSecretVersionRequest,
};

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
    /// Uses Google Secret Manager for production (Cloud Run + Neon)
    /// Uses environment variables for development
    pub async fn auto_load() -> Result<Self> {
        // Cloud Run ve production ortamını tespit et
        let is_production = std::env::var("RUST_ENV").as_deref() == Ok("production");
        let has_gcp_project = std::env::var("GCP_PROJECT_ID").is_ok();
        let is_cloud_run = std::env::var("K_SERVICE").is_ok(); // Cloud Run indicator

        if (is_production || is_cloud_run) && has_gcp_project {
            tracing::info!(
                "Production/Cloud Run environment detected - using Google Secret Manager"
            );
            Self::from_gcp_secrets().await
        } else {
            tracing::info!("Development environment detected - using environment variables");
            Self::from_env()
        }
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

    /// Asynchronously fetches application configuration from Google Secret Manager
    /// using the official Google Cloud Rust SDK.
    /// This is the recommended way to load secrets in a production environment.
    /// KULLANIM: Google Cloud Run + Neon PostgreSQL production deployment için
    pub async fn from_gcp_secrets() -> Result<Self> {
        // GCP Project ID zorunlu - Cloud Run deployment için
        let project_id = std::env::var("GCP_PROJECT_ID")
            .context("GCP_PROJECT_ID environment variable is required for Secret Manager")?;

        if project_id.is_empty() {
            anyhow::bail!("GCP_PROJECT_ID cannot be empty");
        }

        tracing::info!(
            "Connecting to Google Secret Manager for project: {}",
            project_id
        );

        // Create Google Cloud Secret Manager client with default configuration
        // This automatically handles authentication via Application Default Credentials (ADC)
        let client = SecretManagerService::builder()
            .build()
            .await
            .context("Failed to create Google Cloud Secret Manager client")?;

        // Define futures to fetch all required secrets concurrently using the official SDK
        let db_url_fut = fetch_secret_with_sdk(&client, &project_id, "database-url");
        let jwt_secret_fut = fetch_secret_with_sdk(&client, &project_id, "jwt-secret");
        let hashing_key_fut = fetch_secret_with_sdk(&client, &project_id, "hashing-secret");

        tracing::info!("Fetching secrets from Google Secret Manager...");

        // Await all futures to complete
        let (database_url, jwt_secret, hashing_secret_key) =
            tokio::try_join!(db_url_fut, jwt_secret_fut, hashing_key_fut)
                .context("Failed to fetch one or more secrets from Google Secret Manager")?;

        // Server configuration from environment variables (these are safe to be public)
        let server_port = std::env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .context("Invalid PORT value")?;

        let server_host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        // Validate secrets
        if hashing_secret_key.len() < 16 {
            anyhow::bail!("HASHING_SECRET_KEY must be at least 16 characters");
        }

        if jwt_secret.len() < 32 {
            anyhow::bail!("JWT_SECRET must be at least 32 characters");
        }

        tracing::info!("Successfully loaded configuration from Google Secret Manager");
        tracing::info!("Database: Neon PostgreSQL (serverless)");
        tracing::info!("JWT and hashing secrets loaded from Secret Manager");

        // Construct and return the final Config object
        Ok(Config {
            database_url,
            jwt_secret,
            hashing_secret_key,
            server_port,
            server_host,
        })
    }
}

/// Fetch a single secret from Google Secret Manager using the official SDK
/// This is much cleaner and more reliable than manual HTTP requests
async fn fetch_secret_with_sdk(
    client: &SecretManagerService,
    project_id: &str,
    secret_name: &str,
) -> Result<String> {
    let secret_path = format!(
        "projects/{}/secrets/{}/versions/latest",
        project_id, secret_name
    );

    let mut request = AccessSecretVersionRequest::default();
    request.name = secret_path;

    let response = client
        .access_secret_version()
        .with_request(request)
        .send()
        .await
        .with_context(|| {
            format!(
                "Failed to access the latest version of secret '{}' using Google Cloud SDK",
                secret_name
            )
        })?;

    // Extract the secret data from the response
    let secret_data = response
        .payload
        .ok_or_else(|| anyhow::anyhow!("Secret '{}' has no payload", secret_name))?
        .data;

    String::from_utf8(secret_data.to_vec())
        .with_context(|| format!("The secret '{}' was not valid UTF-8", secret_name))
        .map(|s| s.trim().to_string())
}
