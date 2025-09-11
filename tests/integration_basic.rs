use chrono::Utc;
use serial_test::serial;
use std::env;
use todo_api::db::models::{
    CreateTodo, CreateUser, TodoModel, UpdateTodo, UpdateTodoPartial, User,
};
use todo_api::handlers::auth::models::{LoginRequest, RegistrationRequest};
use todo_api::handlers::todo::models::{
    CreateTodoRequest, PartialUpdateTodoRequest, Todo, UpdateTodoRequest,
};
use todo_api::service::jwt::Service as JwtService;

fn create_jwt_service() -> JwtService {
    let secret = env::var("JWT_SECRET").expect("TEST_JWT_SECRET çevre değişkeni ayarlanmamış!");
    JwtService::new(&secret).expect("Çevre değişkenindeki secret ile JWT servisi oluşturulamadı")
}

fn create_test_user() -> User {
    User {
        id: 1,
        username: "integrationuser".to_string(),
        password: "hashed_password".to_string(),
        created: Utc::now().naive_utc(),
        updated: Utc::now().naive_utc(),
        email: Some("integration@example.com".to_string()),
        display_name: Some("Integration User".to_string()),
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

fn create_test_todo_model() -> TodoModel {
    TodoModel {
        id: 1,
        title: "Integration Test Todo".to_string(),
        description: "This is a test todo for integration testing".to_string(),
        created: Utc::now().naive_utc(),
        updated: Utc::now().naive_utc(),
        user_id: 1,
    }
}

#[tokio::test]
#[serial]
async fn test_user_auth_flow() {
    // Test registration request
    let registration_request = RegistrationRequest {
        username: "testuser".to_string(),
        password: "TestPassword123!".to_string(),
    };

    // Test login request with same credentials
    let login_request = LoginRequest {
        username: registration_request.username.clone(),
        password: registration_request.password.clone(),
    };

    assert_eq!(registration_request.username, login_request.username);
    assert_eq!(registration_request.password, login_request.password);
}

#[tokio::test]
#[serial]
async fn test_jwt_authentication_flow() {
    let jwt_service = create_jwt_service();
    let user = create_test_user();

    // Generate token for user
    let token = jwt_service
        .generate_token(&user)
        .expect("Failed to generate token");
    assert!(!token.is_empty());

    // Verify token and get context user
    let context_user = jwt_service
        .verify_token(token)
        .expect("Failed to verify token");

    // Verify context user matches original user
    assert_eq!(context_user.sub, user.id);
    assert_eq!(context_user.username, user.username);
}

#[tokio::test]
#[serial]
async fn test_todo_model_transformations() {
    let todo_model = create_test_todo_model();

    // Test conversion from TodoModel to Todo (response model)
    let todo_response = Todo::from(&todo_model);

    assert_eq!(todo_response.id, todo_model.id as u64);
    assert_eq!(todo_response.title, todo_model.title);
    assert_eq!(todo_response.description, todo_model.description);
}

#[tokio::test]
#[serial]
async fn test_todo_request_to_db_model() {
    let create_request = CreateTodoRequest {
        title: "New Todo".to_string(),
        description: "New todo description".to_string(),
    };

    // Simulate creating a DB model from request
    let create_todo = CreateTodo {
        user_id: 1,
        title: create_request.title.clone(),
        description: create_request.description.clone(),
    };

    assert_eq!(create_todo.title, create_request.title);
    assert_eq!(create_todo.description, create_request.description);
    assert_eq!(create_todo.user_id, 1);
}

#[tokio::test]
#[serial]
async fn test_todo_update_flow() {
    let update_request = UpdateTodoRequest {
        title: "Updated Title".to_string(),
        description: "Updated Description".to_string(),
    };

    // Convert to DB update model
    let update_todo = UpdateTodo::from(update_request.clone());

    assert_eq!(update_todo.title, update_request.title);
    assert_eq!(update_todo.description, update_request.description);
}

#[tokio::test]
#[serial]
async fn test_todo_partial_update_flow() {
    let partial_request = PartialUpdateTodoRequest {
        title: Some("Partially Updated Title".to_string()),
        description: None,
    };

    // Convert to DB partial update model
    let partial_update = UpdateTodoPartial::from(partial_request.clone());

    assert_eq!(partial_update.title, partial_request.title);
    assert_eq!(partial_update.description, partial_request.description);
}

#[tokio::test]
#[serial]
async fn test_user_creation_flow() {
    let create_user = CreateUser {
        username: "newuser".to_string(),
        password: "hashedpassword".to_string(),
        email: Some("newuser@example.com".to_string()),
        display_name: Some("New User".to_string()),
    };

    // Simulate creating a User model after DB insertion
    let user = User {
        id: 1,
        username: create_user.username.clone(),
        password: create_user.password.clone(),
        created: Utc::now().naive_utc(),
        updated: Utc::now().naive_utc(),
        email: Some("newuser@example.com".to_string()),
        display_name: Some("New User".to_string()),
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

    assert_eq!(user.username, create_user.username);
    assert_eq!(user.password, create_user.password);
}

#[tokio::test]
#[serial]
async fn test_complete_todo_lifecycle() {
    let jwt_service = create_jwt_service();
    let user = create_test_user();

    // 1. Generate JWT token for user
    let token = jwt_service.generate_token(&user).unwrap();

    // 2. Verify token to get context user
    let context_user = jwt_service
        .verify_token(token)
        .expect("Failed to verify JWT for context");
    assert_eq!(context_user.sub, user.id);
    assert_eq!(context_user.username, user.username);

    // 3. Create todo request
    let create_request = CreateTodoRequest {
        title: "Lifecycle Test Todo".to_string(),
        description: "Testing complete todo lifecycle".to_string(),
    };

    // 4. Simulate todo creation
    let todo_model = TodoModel {
        id: 1,
        title: create_request.title.clone(),
        description: create_request.description.clone(),
        created: Utc::now().naive_utc(),
        updated: Utc::now().naive_utc(),
        user_id: context_user.sub,
    };

    // 5. Convert to response model
    let todo_response = Todo::from(todo_model);

    assert_eq!(todo_response.title, create_request.title);
    assert_eq!(todo_response.description, create_request.description);
}

#[tokio::test]
#[serial]
async fn test_error_handling_flows() {
    let jwt_service = create_jwt_service();

    // Test invalid token
    let invalid_token_result = jwt_service.verify_token("invalid_token".to_string());
    assert!(invalid_token_result.is_err());

    // Test empty token
    let empty_token_result = jwt_service.verify_token("".to_string());
    assert!(empty_token_result.is_err());

    // Test malformed token
    let malformed_token_result = jwt_service.verify_token("malformed.token".to_string());
    assert!(malformed_token_result.is_err());
}

#[tokio::test]
#[serial]
async fn test_concurrent_operations() {
    let jwt_service = create_jwt_service();

    let users = vec![
        User {
            id: 1,
            username: "user1".to_string(),
            password: "pass1".to_string(),
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
            password: "pass2".to_string(),
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
        User {
            id: 3,
            username: "user3".to_string(),
            password: "pass3".to_string(),
            created: Utc::now().naive_utc(),
            updated: Utc::now().naive_utc(),
            email: Some("user3@example.com".to_string()),
            display_name: Some("User 3".to_string()),
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

    let mut handles = vec![];

    for user in users {
        let service = jwt_service.clone();
        handles.push(tokio::spawn(async move {
            let token = service.generate_token(&user)?;
            let context_user = service.verify_token(token)?;
            assert_eq!(context_user.sub, user.id);
            assert_eq!(context_user.username, user.username);
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        }));
    }

    for handle in handles {
        handle
            .await
            .expect("Task failed")
            .expect("JWT operation failed");
    }
}
