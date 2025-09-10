use fake::{Fake, Faker};
use fake::faker::internet::en::Username;
use fake::faker::lorem::en::{Sentence, Paragraph};
use fake::faker::name::en::Name;
use serde_json::{json, Value};
use todo_api::handlers::auth::models::{LoginRequest, RegistrationRequest};
use todo_api::handlers::todo::models::{CreateTodoRequest, UpdateTodoRequest, PartialUpdateTodoRequest};

/// Generate test user data
pub struct UserFixture;

impl UserFixture {
    pub fn valid_registration_request() -> RegistrationRequest {
        RegistrationRequest {
            username: Username().fake::<String>().chars().take(20).collect(),
            password: "test_password123".to_string(),
        }
    }

    pub fn valid_login_request() -> LoginRequest {
        LoginRequest {
            username: "test_user".to_string(),
            password: "test_password123".to_string(),
        }
    }

    pub fn invalid_short_username() -> RegistrationRequest {
        RegistrationRequest {
            username: "ab".to_string(), // Too short
            password: "test_password123".to_string(),
        }
    }

    pub fn invalid_short_password() -> RegistrationRequest {
        RegistrationRequest {
            username: "valid_user".to_string(),
            password: "short".to_string(), // Too short
        }
    }

    pub fn invalid_username_chars() -> RegistrationRequest {
        RegistrationRequest {
            username: "user@invalid.com".to_string(), // Contains invalid chars
            password: "test_password123".to_string(),
        }
    }
}

/// Generate test todo data
pub struct TodoFixture;

impl TodoFixture {
    pub fn valid_create_request() -> CreateTodoRequest {
        CreateTodoRequest {
            title: Sentence(3..5).fake(),
            description: Paragraph(2..4).fake(),
        }
    }

    pub fn valid_update_request() -> UpdateTodoRequest {
        UpdateTodoRequest {
            title: Sentence(3..5).fake(),
            description: Paragraph(2..4).fake(),
        }
    }

    pub fn valid_partial_update_request() -> PartialUpdateTodoRequest {
        PartialUpdateTodoRequest {
            title: Some(Sentence(3..5).fake()),
            description: None,
        }
    }

    pub fn empty_title_request() -> CreateTodoRequest {
        CreateTodoRequest {
            title: "".to_string(),
            description: Paragraph(2..4).fake(),
        }
    }

    pub fn empty_description_request() -> CreateTodoRequest {
        CreateTodoRequest {
            title: Sentence(3..5).fake(),
            description: "".to_string(),
        }
    }

    pub fn too_long_title_request() -> CreateTodoRequest {
        CreateTodoRequest {
            title: "a".repeat(256), // Assuming 255 char limit
            description: Paragraph(2..4).fake(),
        }
    }

    pub fn multiple_todos(count: usize) -> Vec<CreateTodoRequest> {
        (0..count).map(|_| Self::valid_create_request()).collect()
    }
}

/// API request/response fixtures
pub struct ApiFixture;

impl ApiFixture {
    pub fn auth_header(token: &str) -> (&'static str, String) {
        ("Authorization", format!("Bearer {}", token))
    }

    pub fn invalid_auth_header() -> (&'static str, &'static str) {
        ("Authorization", "Invalid token format")
    }

    pub fn expired_token() -> String {
        // This would be a JWT with exp in the past
        "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwibmFtZSI6InRlc3QiLCJleHAiOjE2MDk0NTkyMDB9.invalid".to_string()
    }

    pub fn malformed_json() -> Value {
        json!({
            "title": 123, // Should be string
            "description": null // Should be string
        })
    }

    pub fn missing_required_fields() -> Value {
        json!({
            "description": "Missing title field"
        })
    }
}

/// Database fixtures for integration tests
pub struct DatabaseFixture;

impl DatabaseFixture {
    pub fn sample_user_data() -> (String, String) {
        ("test_user_123".to_string(), "hashed_password_123".to_string())
    }

    pub fn sample_todo_data(user_id: i32) -> (String, String, i32) {
        (
            "Sample Todo Title".to_string(),
            "Sample todo description".to_string(),
            user_id,
        )
    }
}

/// Performance test fixtures
pub struct PerformanceFixture;

impl PerformanceFixture {
    pub fn load_test_requests(count: usize) -> Vec<CreateTodoRequest> {
        TodoFixture::multiple_todos(count)
    }

    pub fn concurrent_users(count: usize) -> Vec<RegistrationRequest> {
        (0..count).map(|i| RegistrationRequest {
            username: format!("user_{}", i),
            password: "test_password123".to_string(),
        }).collect()
    }
}

/// Security test fixtures
pub struct SecurityFixture;

impl SecurityFixture {
    pub fn sql_injection_payloads() -> Vec<String> {
        vec![
            "'; DROP TABLE users; --".to_string(),
            "' OR '1'='1".to_string(),
            "'; UPDATE users SET password = 'hacked'; --".to_string(),
            "' UNION SELECT * FROM users --".to_string(),
        ]
    }

    pub fn xss_payloads() -> Vec<String> {
        vec![
            "<script>alert('xss')</script>".to_string(),
            "javascript:alert('xss')".to_string(),
            "<img src=x onerror=alert('xss')>".to_string(),
        ]
    }

    pub fn large_payload_request() -> CreateTodoRequest {
        CreateTodoRequest {
            title: "A".repeat(10000),
            description: "B".repeat(100000),
        }
    }
}