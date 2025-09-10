use dotenvy::dotenv;
use std::env;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),
    #[error("Environment variable error: {0}")]
    EnvVarError(#[from] env::VarError),
}

#[derive(Debug, Clone)]
pub struct Config {
    #[allow(dead_code)] // Used through environment variables in db module
    pub database_url: String,
    pub hashing_secret_key: String,
    pub jwt_secret: String,
    pub server_port: u16,
    pub server_host: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingEnvVar("DATABASE_URL".to_string()))?;
        
        let hashing_secret_key = env::var("HASHING_SECRET_KEY")
            .map_err(|_| ConfigError::MissingEnvVar("HASHING_SECRET_KEY".to_string()))?;
        
        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| ConfigError::MissingEnvVar("JWT_SECRET".to_string()))?;
        
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "9999".to_string())
            .parse()
            .unwrap_or(9999);
        
        let server_host = env::var("SERVER_HOST")
            .unwrap_or_else(|_| "0.0.0.0".to_string());

        // Validate required secrets
        if hashing_secret_key.len() < 16 {
            return Err(ConfigError::MissingEnvVar("HASHING_SECRET_KEY must be at least 16 characters".to_string()));
        }
        
        if jwt_secret.len() < 32 {
            return Err(ConfigError::MissingEnvVar("JWT_SECRET must be at least 32 characters".to_string()));
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