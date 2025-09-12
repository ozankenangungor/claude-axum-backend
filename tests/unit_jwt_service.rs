use anyhow::Result;
use chrono::NaiveDate;
use std::env;
use todo_api::db::models::User;
use todo_api::service::jwt::Service as JwtService;

// Testlerin çoğunun kullanacağı yardımcı fonksiyon.
// JWT_SECRET'ı YAML dosyasındaki çevre değişkeninden okur.
fn get_jwt_service_from_env() -> JwtService {
    let secret = env::var("JWT_SECRET").expect("TEST_JWT_SECRET environment variable not set!");
    JwtService::new(&secret).expect("Failed to create JWT service from environment secret")
}

#[tokio::test]
async fn test_jwt_token_generation() -> Result<()> {
    let jwt_service = get_jwt_service_from_env();

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
    let jwt_service = get_jwt_service_from_env();

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

    let token = jwt_service.generate_token(&user)?;
    let context_user = jwt_service.verify_token(token)?;

    assert_eq!(context_user.sub, 42);
    assert_eq!(context_user.username, "test_user_42");

    Ok(())
}

#[tokio::test]
async fn test_jwt_invalid_token() {
    let jwt_service = get_jwt_service_from_env();
    let invalid_token = "invalid.token.here";
    let result = jwt_service.verify_token(invalid_token.to_string());
    assert!(result.is_err());
}

#[tokio::test]
async fn test_jwt_empty_token() {
    let jwt_service = get_jwt_service_from_env();
    let result = jwt_service.verify_token("".to_string());
    assert!(result.is_err());
}

#[tokio::test]
async fn test_jwt_malformed_token() {
    let jwt_service = get_jwt_service_from_env();
    let malformed_tokens = vec!["not.a.jwt", "header.payload", "a.b.c.d", "header"];
    for token in malformed_tokens {
        let result = jwt_service.verify_token(token.to_string());
        assert!(result.is_err(), "Token '{}' should be invalid", token);
    }
}

#[tokio::test]
async fn test_jwt_with_different_secrets() -> Result<()> {
    // Bu testin amacı farklı secret'ları denemek olduğu için
    // çevre değişkeni yerine geçerli Base64 değerleri kullanıyoruz.
    let secret1 = "dGhpcyBpcyB0ZXN0IHNlY3JldCBudW1iZXIgb25lISE=";
    let secret2 = "dGhpcyBpcyB0ZXN0IHNlY3JldCBudW1iZXIgdHdvISE=";

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

    let token = service1.generate_token(&user)?;
    let result = service2.verify_token(token);
    assert!(
        result.is_err(),
        "Token should not be valid with different secret"
    );

    Ok(())
}

#[tokio::test]
async fn test_jwt_service_creation_with_short_secret() {
    // "short" kelimesinin Base64 hali
    let short_secret = "c2hvcnQ=";
    let result = JwtService::new(short_secret);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multiple_token_generations() -> Result<()> {
    let jwt_service = get_jwt_service_from_env();
    let mut tokens = Vec::new();

    for i in 0..5 {
        let user = User {
            id: i,
            username: format!("user_{}", i),
            // ... (diğer alanlar aynı)
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

    for i in 0..tokens.len() {
        for j in i + 1..tokens.len() {
            assert_ne!(tokens[i], tokens[j], "Tokens should be unique");
        }
    }

    for (i, token) in tokens.iter().enumerate() {
        let context_user = jwt_service.verify_token(token.clone())?;
        assert_eq!(context_user.sub, i as i32);
        assert_eq!(context_user.username, format!("user_{}", i));
    }

    Ok(())
}
