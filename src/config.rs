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
    /// Asynchronously fetches application configuration from Google Secret Manager
    /// using the official Google Cloud Rust SDK.
    /// This is the recommended way to load secrets in a production environment.
    pub async fn from_gcp_secrets() -> Result<Self> {
        // The GCP Project ID should be provided as an environment variable,
        // which we will set in our deployment configuration.
        let project_id = std::env::var("GCP_PROJECT_ID");

        // For local development, fallback to environment variables if GCP_PROJECT_ID is not set
        if project_id.is_err() || project_id.as_ref().unwrap().is_empty() {
            return Self::from_env_fallback();
        }

        let project_id = project_id.unwrap();

        // Create Google Cloud Secret Manager client with default configuration
        // This automatically handles authentication via Application Default Credentials (ADC)
        let client = SecretManagerService::builder().build().await?;

        // Define futures to fetch all required secrets concurrently using the official SDK
        let db_url_fut = fetch_secret_with_sdk(&client, &project_id, "database-url");
        let jwt_secret_fut = fetch_secret_with_sdk(&client, &project_id, "jwt-secret");
        let hashing_key_fut = fetch_secret_with_sdk(&client, &project_id, "hashing-secret");

        // Await all futures to complete
        let (database_url, jwt_secret, hashing_secret_key) =
            tokio::try_join!(db_url_fut, jwt_secret_fut, hashing_key_fut)?;

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

        // Construct and return the final Config object
        Ok(Config {
            database_url,
            jwt_secret,
            hashing_secret_key,
            server_port,
            server_host,
        })
    }

    /// Fallback configuration loader for local development
    fn from_env_fallback() -> Result<Self> {
        // Try to load from .env file for local development
        dotenvy::dotenv().ok();

        let database_url = std::env::var("DATABASE_URL")
            .context("DATABASE_URL environment variable is required")?;

        let hashing_secret_key = std::env::var("HASHING_SECRET_KEY")
            .context("HASHING_SECRET_KEY environment variable is required")?;

        let jwt_secret =
            std::env::var("JWT_SECRET").context("JWT_SECRET environment variable is required")?;

        let server_port = std::env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .context("Invalid SERVER_PORT value")?;

        let server_host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

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
