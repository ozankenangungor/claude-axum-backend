use base64::{engine::general_purpose, Engine as _};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::env::VarError;
use thiserror::Error;

use crate::{db::models::User, handlers::models::Claims};

#[derive(Clone)]
pub struct ContextUser {
    pub user_id: i32,
    pub username: String,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Missing environment variable: {0}")]
    EvnVar(#[from] VarError),
    #[error("JWT error: {0}")]
    JWT(#[from] jsonwebtoken::errors::Error),
    #[error("Failed to parse int from string: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("Base64 Decode Error: {0}")]
    Base64Decode(#[from] base64::DecodeError),
}

#[derive(Clone)]
pub struct Service {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl Service {
    // Bu fonksiyon artık Base64 formatında bir secret bekliyor
    pub fn new(jwt_secret_base64: &str) -> Result<Self, Error> {
        // Gelen Base64 string'ini byte dizisine çeviriyoruz.
        // Çökme (panic) yerine artık Result döndürüyor, çok daha güvenli.
        let secret_bytes = general_purpose::STANDARD.decode(jwt_secret_base64)?;

        Ok(Self {
            encoding_key: EncodingKey::from_secret(&secret_bytes),
            decoding_key: DecodingKey::from_secret(&secret_bytes),
        })
    }

    pub fn generate_token(&self, user: &User) -> Result<String, Error> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::hours(24))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user.id,
            username: user.username.clone(),
            exp: expiration as usize,
        };

        let token = encode(&Header::default(), &claims, &self.encoding_key)?;
        Ok(token)
    }

    pub fn verify_token(&self, token: String) -> Result<Claims, Error> {
        let token_data = decode::<Claims>(&token, &self.decoding_key, &Validation::default())?;
        Ok(token_data.claims)
    }
}
