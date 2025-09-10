use chrono::Utc;
use serial_test::serial;
use todo_api::db::models::User;
use todo_api::handlers::auth::models::{LoginRequest, RegistrationRequest};
use todo_api::service::jwt::Service as JwtService;

#[derive(Clone)]
struct MockConfig {
    jwt_secret: String,
    #[allow(dead_code)] // Reserved for future hash testing scenarios
    hashing_secret: String,
}

impl MockConfig {
    fn new() -> Self {
        Self {
            jwt_secret: "test_secret_key_for_jwt_that_is_long_enough_for_testing".to_string(),
            hashing_secret: "test_hashing_secret".to_string(),
        }
    }
}

fn create_jwt_service() -> JwtService {
    let config = MockConfig::new();
    JwtService::new(&config.jwt_secret).expect("Failed to create JWT service")
}

fn create_test_user() -> User {
    User {
        id: 1,
        username: "testuser".to_string(),
        password: "hashed_password".to_string(),
        created: Utc::now().naive_utc(),
        updated: Utc::now().naive_utc(),
        email: Some("test@example.com".to_string()),
        display_name: Some("Test User".to_string()),
        bio: None,
        avatar_url: None,
        location: None,
        website: None,
        is_verified: Some(false),
        is_private: Some(false),
        follower_count: Some(0),
        following_count: Some(0),
        post_count: Some(0),
    }
}

#[tokio::test]
#[serial]
async fn test_jwt_service_creation() {
    let jwt_service = create_jwt_service();
    // Test that service is created successfully
    let user = create_test_user();
    let token_result = jwt_service.generate_token(&user);
    assert!(token_result.is_ok());
}

#[tokio::test]
#[serial]
async fn test_registration_request_validation() {
    let request = RegistrationRequest {
        username: "testuser".to_string(),
        password: "TestPassword123!".to_string(),
    };

    assert_eq!(request.username, "testuser");
    assert_eq!(request.password, "TestPassword123!");
}

#[tokio::test]
#[serial]
async fn test_login_request_validation() {
    let request = LoginRequest {
        username: "testuser".to_string(),
        password: "password123".to_string(),
    };

    assert_eq!(request.username, "testuser");
    assert_eq!(request.password, "password123");
}

#[tokio::test]
#[serial]
async fn test_user_model_structure() {
    let user = create_test_user();

    assert_eq!(user.id, 1);
    assert_eq!(user.username, "testuser");
    assert!(!user.password.is_empty());
    assert!(user.created <= Utc::now().naive_utc());
    assert!(user.updated <= Utc::now().naive_utc());
}

#[tokio::test]
#[serial]
async fn test_jwt_token_generation_and_verification() {
    let jwt_service = create_jwt_service();
    let user = create_test_user();

    // Generate token
    let token_result = jwt_service.generate_token(&user);
    assert!(token_result.is_ok());

    let token = token_result.unwrap();
    assert!(!token.is_empty());

    // Verify token
    let verification_result = jwt_service.verify_token(token);
    assert!(verification_result.is_ok());

    let context_user = verification_result.unwrap();
    assert_eq!(context_user.sub, user.id);
    assert_eq!(context_user.username, user.username);
}

#[tokio::test]
#[serial]
async fn test_jwt_invalid_token() {
    let jwt_service = create_jwt_service();

    let invalid_token = "invalid.token.here".to_string();
    let verification_result = jwt_service.verify_token(invalid_token);
    assert!(verification_result.is_err());
}

#[tokio::test]
#[serial]
async fn test_jwt_empty_token() {
    let jwt_service = create_jwt_service();

    let verification_result = jwt_service.verify_token("".to_string());
    assert!(verification_result.is_err());
}

#[tokio::test]
#[serial]
async fn test_jwt_malformed_token() {
    let jwt_service = create_jwt_service();

    let malformed_tokens = vec![
        "not.a.jwt",
        "header.payload",
        "header..signature",
        ".payload.signature",
        "header.payload.",
    ];

    for token in malformed_tokens {
        let verification_result = jwt_service.verify_token(token.to_string());
        assert!(
            verification_result.is_err(),
            "Token should be invalid: {}",
            token
        );
    }
}

#[tokio::test]
#[serial]
async fn test_multiple_users_jwt() {
    let jwt_service = create_jwt_service();

    let users = vec![
        User {
            id: 1,
            username: "user1".to_string(),
            password: "password1".to_string(),
            created: Utc::now().naive_utc(),
            updated: Utc::now().naive_utc(),
            email: Some("user1@example.com".to_string()),
            display_name: Some("User 1".to_string()),
            bio: None,
            avatar_url: None,
            location: None,
            website: None,
            is_verified: Some(false),
            is_private: Some(false),
            follower_count: Some(0),
            following_count: Some(0),
            post_count: Some(0),
        },
        User {
            id: 2,
            username: "user2".to_string(),
            password: "password2".to_string(),
            created: Utc::now().naive_utc(),
            updated: Utc::now().naive_utc(),
            email: Some("user2@example.com".to_string()),
            display_name: Some("User 2".to_string()),
            bio: None,
            avatar_url: None,
            location: None,
            website: None,
            is_verified: Some(false),
            is_private: Some(false),
            follower_count: Some(0),
            following_count: Some(0),
            post_count: Some(0),
        },
    ];

    for user in users {
        let token_result = jwt_service.generate_token(&user);
        assert!(token_result.is_ok());

        let token = token_result.unwrap();
        let verification_result = jwt_service.verify_token(token);
        assert!(verification_result.is_ok());

        let context_user = verification_result.unwrap();
        assert_eq!(context_user.sub, user.id);
        assert_eq!(context_user.username, user.username);
    }
}

#[tokio::test]
#[serial]
async fn test_jwt_token_contains_correct_data() {
    let jwt_service = create_jwt_service();
    let user = User {
        id: 42,
        username: "specialuser".to_string(),
        password: "hashedpass".to_string(),
        created: Utc::now().naive_utc(),
        updated: Utc::now().naive_utc(),
        email: Some("special@example.com".to_string()),
        display_name: Some("Special User".to_string()),
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

    let token = jwt_service.generate_token(&user).unwrap();
    let context_user = jwt_service.verify_token(token).unwrap();

    assert_eq!(context_user.sub, 42);
    assert_eq!(context_user.username, "specialuser");
}

#[tokio::test]
#[serial]
async fn test_password_strength_requirements() {
    // Test various password patterns
    let weak_passwords = vec![
        "",          // Empty
        "1234567",   // Too short
        "password",  // No uppercase/digits/special
        "PASSWORD",  // No lowercase/digits/special
        "Password",  // No digits/special
        "Password1", // No special characters
    ];

    // These are just string validations, not actual auth service tests
    for password in weak_passwords {
        assert!(
            password.len() < 8
                || !password.chars().any(|c| c.is_uppercase())
                || !password.chars().any(|c| c.is_lowercase())
                || !password.chars().any(|c| c.is_numeric())
                || !password
                    .chars()
                    .any(|c| "!@#$%^&*()_+-=[]{}|;':.,<>/?".contains(c))
        );
    }
}

#[tokio::test]
#[serial]
async fn test_clone_operations() {
    let request = RegistrationRequest {
        username: "testuser".to_string(),
        password: "TestPassword123!".to_string(),
    };

    // Clone should work for request structs
    let login_request = LoginRequest {
        username: request.username.clone(),
        password: request.password.clone(),
    };

    assert_eq!(request.username, login_request.username);
    assert_eq!(request.password, login_request.password);
}

#[tokio::test]
#[serial]
async fn test_jwt_service_with_different_secrets() {
    let jwt_service1 = JwtService::new("secret1").expect("Failed to create JWT service 1");
    let jwt_service2 = JwtService::new("secret2").expect("Failed to create JWT service 2");

    let user = create_test_user();

    // Token generated with service1
    let token1 = jwt_service1.generate_token(&user).unwrap();

    // Should be verifiable with service1
    assert!(jwt_service1.verify_token(token1.clone()).is_ok());

    // Should NOT be verifiable with service2 (different secret)
    assert!(jwt_service2.verify_token(token1).is_err());
}
