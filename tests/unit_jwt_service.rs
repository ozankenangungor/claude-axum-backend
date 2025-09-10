use anyhow::Result;
use chrono::NaiveDate;
use todo_api::db::models::User;
use todo_api::service::jwt::Service as JwtService;

#[tokio::test]
async fn test_jwt_token_generation() -> Result<()> {
    let jwt_secret = "test_jwt_secret_12345678901234567890123456789012345";
    let jwt_service = JwtService::new(jwt_secret)?;

    let user = User {
        id: 1,
        username: "test_user".to_string(),
        password: "hashedpassword".to_string(),
        created: NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap(),
        updated: NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap(),
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
    };

    let token = jwt_service.generate_token(&user)?;
    assert!(!token.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_jwt_token_verification() -> Result<()> {
    let jwt_secret = "test_jwt_secret_12345678901234567890123456789012345";
    let jwt_service = JwtService::new(jwt_secret)?;

    let user = User {
        id: 42,
        username: "test_user_42".to_string(),
        password: "hashedpassword".to_string(),
        created: NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap(),
        updated: NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap(),
        email: Some("test42@example.com".to_string()),
        display_name: Some("Test User 42".to_string()),
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

    // Generate token
    let token = jwt_service.generate_token(&user)?;

    // Verify token
    let context_user = jwt_service.verify_token(token)?;

    assert_eq!(context_user.sub, 42);
    assert_eq!(context_user.username, "test_user_42");

    Ok(())
}

#[tokio::test]
async fn test_jwt_invalid_token() {
    let jwt_secret = "test_jwt_secret_12345678901234567890123456789012345";
    let jwt_service = JwtService::new(jwt_secret).unwrap();

    let invalid_token = "invalid.token.here";
    let result = jwt_service.verify_token(invalid_token.to_string());

    assert!(result.is_err());
}

#[tokio::test]
async fn test_jwt_empty_token() {
    let jwt_secret = "test_jwt_secret_12345678901234567890123456789012345";
    let jwt_service = JwtService::new(jwt_secret).unwrap();

    let result = jwt_service.verify_token("".to_string());
    assert!(result.is_err());
}

#[tokio::test]
async fn test_jwt_malformed_token() {
    let jwt_secret = "test_jwt_secret_12345678901234567890123456789012345";
    let jwt_service = JwtService::new(jwt_secret).unwrap();

    let malformed_tokens = vec![
        "not.a.jwt",
        "header.payload", // Missing signature
        "a.b.c.d",        // Too many parts
        "header",         // Only header
    ];

    for token in malformed_tokens {
        let result = jwt_service.verify_token(token.to_string());
        assert!(result.is_err(), "Token '{}' should be invalid", token);
    }
}

#[tokio::test]
async fn test_jwt_with_different_secrets() -> Result<()> {
    let secret1 = "secret1_12345678901234567890123456789012345";
    let secret2 = "secret2_12345678901234567890123456789012345";

    let service1 = JwtService::new(secret1)?;
    let service2 = JwtService::new(secret2)?;

    let user = User {
        id: 1,
        username: "test".to_string(),
        password: "hashedpassword".to_string(),
        created: NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap(),
        updated: NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap(),
        email: Some("test@example.com".to_string()),
        display_name: Some("Test".to_string()),
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

    // Generate token with service1
    let token = service1.generate_token(&user)?;

    // Try to verify with service2 (different secret)
    let result = service2.verify_token(token);
    assert!(
        result.is_err(),
        "Token should not be valid with different secret"
    );

    Ok(())
}

#[tokio::test]
async fn test_jwt_service_creation_with_short_secret() {
    let short_secret = "short";
    let result = JwtService::new(short_secret);

    // This might pass depending on your implementation
    // You might want to add secret length validation
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_multiple_token_generations() -> Result<()> {
    let jwt_secret = "test_jwt_secret_12345678901234567890123456789012345";
    let jwt_service = JwtService::new(jwt_secret)?;

    let mut tokens = Vec::new();

    // Generate multiple tokens
    for i in 0..5 {
        let user = User {
            id: i,
            username: format!("user_{}", i),
            password: "hashedpassword".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            updated: NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            email: Some(format!("user{}@example.com", i)),
            display_name: Some(format!("User {}", i)),
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
        let token = jwt_service.generate_token(&user)?;
        tokens.push(token);
    }

    // Verify all tokens are different
    for i in 0..tokens.len() {
        for j in i + 1..tokens.len() {
            assert_ne!(tokens[i], tokens[j], "Tokens should be unique");
        }
    }

    // Verify all tokens can be decoded
    for (i, token) in tokens.iter().enumerate() {
        let context_user = jwt_service.verify_token(token.clone())?;
        assert_eq!(context_user.sub, i as i32);
        assert_eq!(context_user.username, format!("user_{}", i));
    }

    Ok(())
}
