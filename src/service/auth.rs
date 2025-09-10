use std::{env::VarError, sync::Arc};

use argonautica::Verifier;
use regex::Regex;
use sqlx::PgPool;
use thiserror::Error;

use crate::{
    db::{models::User, DbConnectionPoolError},
    handlers::auth::models::{LoginRequest, RegistrationRequest},
    service,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Connection pool init error: {0}")]
    ConnectionPool(#[from] DbConnectionPoolError),
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Hashing error: {0}")]
    Hashing(argonautica::Error),
    #[error("Failed to get environment variable: {0}")]
    EnvVar(#[from] VarError),
    #[error("Username already exists: {0}")]
    UsernameAlreadyExists(String),
    #[error("Invalid password")]
    InvalidPassword,
    #[error("JWT service error: {0}")]
    JwtService(#[from] service::jwt::Error),
    #[error("User not found")]
    UserNotFound,
    #[error("Weak password: {0}")]
    WeakPassword(String),
}

pub struct Service {
    jwt_service: Arc<service::jwt::Service>,
    db_pool: PgPool,
    hashing_secret: String,
}

impl Service {
    pub fn new(
        jwt_service: Arc<service::jwt::Service>,
        db_pool: PgPool,
        hashing_secret: String,
    ) -> Result<Self, Error> {
        Ok(Self {
            jwt_service,
            db_pool,
            hashing_secret,
        })
    }

    fn validate_password(password: &str) -> Result<(), Error> {
        if password.len() < 8 {
            return Err(Error::WeakPassword(
                "Password must be at least 8 characters long".to_string(),
            ));
        }

        let has_uppercase = Regex::new(r"[A-Z]").unwrap().is_match(password);
        let has_lowercase = Regex::new(r"[a-z]").unwrap().is_match(password);
        let has_digit = Regex::new(r"\d").unwrap().is_match(password);
        let has_special = Regex::new(r"[!@#$%^&*()_+\-=\[\]{};':.,<>/?]")
            .unwrap()
            .is_match(password);

        if !has_uppercase {
            return Err(Error::WeakPassword(
                "Password must contain at least one uppercase letter".to_string(),
            ));
        }
        if !has_lowercase {
            return Err(Error::WeakPassword(
                "Password must contain at least one lowercase letter".to_string(),
            ));
        }
        if !has_digit {
            return Err(Error::WeakPassword(
                "Password must contain at least one digit".to_string(),
            ));
        }
        if !has_special {
            return Err(Error::WeakPassword(
                "Password must contain at least one special character".to_string(),
            ));
        }

        Ok(())
    }

    pub async fn login(&self, request: LoginRequest) -> Result<String, Error> {
        // Fetch user with basic fields only (social media fields will be added via migration)
        let found_user_basic = sqlx::query!(
            r#"
            SELECT id, username, password, created, updated
            FROM users
            WHERE username = $1
            "#,
            request.username
        )
        .fetch_optional(&self.db_pool)
        .await?;

        let found_user_basic = match found_user_basic {
            Some(user) => user,
            None => return Err(Error::UserNotFound),
        };

        // Create User struct with defaults for new fields
        let found_user = User {
            id: found_user_basic.id,
            username: found_user_basic.username,
            password: found_user_basic.password,
            created: found_user_basic.created,
            updated: found_user_basic.updated,
            email: None,
            display_name: None,
            bio: None,
            avatar_url: None,
            location: None,
            website: None,
            is_verified: Some(false),
            is_private: Some(false),
            follower_count: Some(0),
            following_count: Some(0),
            post_count: Some(0),
        };

        let mut password_hash_verifier = Verifier::default();
        let pass_valid = password_hash_verifier
            .with_hash(&found_user.password)
            .with_password(request.password)
            .with_secret_key(&self.hashing_secret)
            .verify()
            .map_err(|error| Error::Hashing(error))?;

        if !pass_valid {
            return Err(Error::InvalidPassword);
        }

        let token = self.jwt_service.generate_token(&found_user)?;
        Ok(token)
    }

    pub async fn register(&self, request: RegistrationRequest) -> Result<(), Error> {
        // Validate password complexity
        Self::validate_password(&request.password)?;

        let existing_user_count = sqlx::query!(
            "SELECT COUNT(*) as count FROM users WHERE username = $1",
            request.username
        )
        .fetch_one(&self.db_pool)
        .await?;

        if existing_user_count.count.unwrap_or(0) > 0 {
            return Err(Error::UsernameAlreadyExists(request.username));
        }

        let password_hash = argonautica::Hasher::default()
            .with_password(request.password)
            .with_secret_key(&self.hashing_secret)
            .hash()
            .map_err(|error| Error::Hashing(error))?;

        sqlx::query!(
            r#"
            INSERT INTO users (username, password)
            VALUES ($1, $2)
            "#,
            request.username,
            password_hash
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}
