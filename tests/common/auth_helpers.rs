use anyhow::Result;
use todo_api::service::jwt::{Service as JwtService, ContextUser};
use todo_api::db::models::User;
use todo_api::handlers::auth::models::{LoginRequest, RegistrationRequest};
use todo_api::AppState;
use chrono::NaiveDate;

/// Authentication test helpers
pub struct AuthTestHelper;

impl AuthTestHelper {
    /// Create a valid JWT token for testing
    pub fn create_test_token(jwt_service: &JwtService, user_id: i32, username: &str) -> Result<String> {
        let user = User {
            id: user_id,
            username: username.to_string(),
            password: "hashedpassword".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap(),
        };
        
        jwt_service.generate_token(&user).map_err(|e| anyhow::anyhow!("JWT generation failed: {:?}", e))
    }

    /// Create an expired JWT token for testing
    pub fn create_expired_token() -> String {
        // A token that's already expired (you'll need to implement this in jwt service)
        "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwibmFtZSI6InRlc3QiLCJleHAiOjE2MDk0NTkyMDB9.test".to_string()
    }

    /// Create a malformed JWT token
    pub fn create_malformed_token() -> String {
        "invalid.jwt.token".to_string()
    }

    /// Register and login a test user, return the JWT token
    pub async fn register_and_login_user(
        app_state: &AppState,
        username: &str,
        password: &str,
    ) -> Result<String> {
        // Register user
        let registration_request = RegistrationRequest {
            username: username.to_string(),
            password: password.to_string(),
        };

        app_state.auth_service.register(registration_request).await?;

        // Login user
        let login_request = LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        };

        let token = app_state.auth_service.login(login_request).await?;
        Ok(token)
    }

    /// Create multiple test users and return their tokens
    pub async fn create_test_users(
        app_state: &AppState,
        count: usize,
    ) -> Result<Vec<(String, String)>> { // (username, token)
        let mut users = Vec::new();

        for i in 0..count {
            let username = format!("test_user_{}", i);
            let password = "test_password123".to_string();

            let token = Self::register_and_login_user(app_state, &username, &password).await?;
            users.push((username, token));
        }

        Ok(users)
    }

    /// Verify JWT token and extract user info
    pub fn verify_token(jwt_service: &JwtService, token: &str) -> Result<ContextUser> {
        jwt_service.verify_token(token.to_string())
            .map_err(|e| anyhow::anyhow!("Token verification failed: {:?}", e))
    }

    /// Create authorization header with Bearer token
    pub fn bearer_header(token: &str) -> (&'static str, String) {
        ("Authorization", format!("Bearer {}", token))
    }

    /// Invalid authorization header formats for testing
    pub fn invalid_auth_headers() -> Vec<(&'static str, &'static str)> {
        vec![
            ("Authorization", "Token invalid"),  // Wrong prefix
            ("Authorization", "Bearer"),         // Missing token
            ("Authorization", ""),               // Empty
            ("Auth", "Bearer valid_token"),      // Wrong header name
        ]
    }
}

/// Test user context for consistent testing
#[derive(Debug, Clone)]
pub struct TestUser {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub token: String,
}

impl TestUser {
    pub async fn create(app_state: &AppState, username: &str, password: &str) -> Result<Self> {
        let token = AuthTestHelper::register_and_login_user(app_state, username, password).await?;
        
        // Extract user ID from token (you'll need to implement this)
        let context_user = AuthTestHelper::verify_token(&app_state.jwt_service, &token)?;

        Ok(Self {
            id: context_user.user_id,
            username: context_user.username,
            password: password.to_string(),
            token,
        })
    }

    pub fn auth_header(&self) -> (&'static str, String) {
        AuthTestHelper::bearer_header(&self.token)
    }
}

/// Mock authentication states for testing
pub struct MockAuthStates;

impl MockAuthStates {
    /// Simulate authenticated request
    pub fn authenticated_request(user_id: i32, username: &str) -> ContextUser {
        ContextUser {
            user_id,
            username: username.to_string(),
        }
    }

    /// Simulate unauthenticated request (missing auth)
    pub fn unauthenticated_request() -> Option<ContextUser> {
        None
    }

    /// Simulate invalid authentication
    pub fn invalid_auth_request() -> String {
        "invalid_token".to_string()
    }
}