use chrono::Utc;
use serial_test::serial;
use todo_api::db::models::{User, TodoModel, CreateUser, CreateTodo, UpdateTodo, UpdateTodoPartial};
use todo_api::handlers::todo::models::{CreateTodoRequest, PartialUpdateTodoRequest, UpdateTodoRequest};

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

fn create_test_todo_request() -> CreateTodoRequest {
    CreateTodoRequest {
        title: "Test Todo".to_string(),
        description: "Test description".to_string(),
    }
}

fn create_test_todo_model() -> TodoModel {
    TodoModel {
        id: 1,
        title: "Test Todo".to_string(),
        description: "Test description".to_string(),
        created: Utc::now().naive_utc(),
        updated: Utc::now().naive_utc(),
        user_id: 1,
    }
}

#[tokio::test]
#[serial]
async fn test_create_todo_request_validation() {
    let request = CreateTodoRequest {
        title: "Valid Title".to_string(),
        description: "Valid description".to_string(),
    };
    
    assert_eq!(request.title, "Valid Title");
    assert_eq!(request.description, "Valid description");
}

#[tokio::test]
#[serial]
async fn test_todo_update_request() {
    let request = UpdateTodoRequest {
        title: "Updated Title".to_string(),
        description: "Updated description".to_string(),
    };
    
    assert_eq!(request.title, "Updated Title");
    assert_eq!(request.description, "Updated description");
}

#[tokio::test]
#[serial]
async fn test_todo_partial_update_request() {
    let request = PartialUpdateTodoRequest {
        title: Some("Patched Title".to_string()),
        description: None,
    };
    
    assert_eq!(request.title, Some("Patched Title".to_string()));
    assert!(request.description.is_none());
}

#[tokio::test]
#[serial]
async fn test_todo_partial_update_request_empty() {
    let request = PartialUpdateTodoRequest {
        title: None,
        description: None,
    };
    
    assert!(request.title.is_none());
    assert!(request.description.is_none());
}

#[tokio::test]
#[serial]
async fn test_todo_title_edge_cases() {
    // Empty title
    let request = CreateTodoRequest {
        title: "".to_string(),
        description: "Test".to_string(),
    };
    assert_eq!(request.title, "");
    
    // Very long title
    let long_title = "a".repeat(1000);
    let request = CreateTodoRequest {
        title: long_title.clone(),
        description: "Test".to_string(),
    };
    assert_eq!(request.title, long_title);
    
    // Unicode title
    let unicode_title = "✓ Türkçe görev 🎯";
    let request = CreateTodoRequest {
        title: unicode_title.to_string(),
        description: "Test".to_string(),
    };
    assert_eq!(request.title, unicode_title);
}

#[tokio::test]
#[serial]
async fn test_todo_description_edge_cases() {
    // Very long description
    let long_description = "a".repeat(5000);
    let request = CreateTodoRequest {
        title: "Test".to_string(),
        description: long_description.clone(),
    };
    assert_eq!(request.description, long_description);
    
    // Multiline description
    let multiline_description = "Line 1\nLine 2\nLine 3";
    let request = CreateTodoRequest {
        title: "Test".to_string(),
        description: multiline_description.to_string(),
    };
    assert_eq!(request.description, multiline_description.to_string());
}

#[tokio::test]
#[serial]
async fn test_clone_derivations() {
    let request = create_test_todo_request();
    let cloned_request = request.clone();
    
    assert_eq!(request.title, cloned_request.title);
    assert_eq!(request.description, cloned_request.description);
}

#[tokio::test]
#[serial]
async fn test_user_model() {
    let user = create_test_user();
    assert_eq!(user.id, 1);
    assert_eq!(user.username, "testuser");
    assert!(!user.password.is_empty());
}

#[tokio::test]
#[serial]
async fn test_create_user_model() {
    let create_user = CreateUser {
        username: "newuser".to_string(),
        password: "password123".to_string(),
        email: Some("newuser@example.com".to_string()),
        display_name: Some("New User".to_string()),
    };
    
    assert_eq!(create_user.username, "newuser");
    assert_eq!(create_user.password, "password123");
}

#[tokio::test]
#[serial]
async fn test_todo_model() {
    let todo = create_test_todo_model();
    assert_eq!(todo.id, 1);
    assert_eq!(todo.title, "Test Todo");
    assert_eq!(todo.description, "Test description");
    assert_eq!(todo.user_id, 1);
}

#[tokio::test]
#[serial]
async fn test_create_todo_model() {
    let create_todo = CreateTodo {
        user_id: 1,
        title: "New Todo".to_string(),
        description: "New description".to_string(),
    };
    
    assert_eq!(create_todo.user_id, 1);
    assert_eq!(create_todo.title, "New Todo");
    assert_eq!(create_todo.description, "New description");
}

#[tokio::test]
#[serial]
async fn test_update_todo_model() {
    let update_todo = UpdateTodo {
        title: "Updated Todo".to_string(),
        description: "Updated description".to_string(),
    };
    
    assert_eq!(update_todo.title, "Updated Todo");
    assert_eq!(update_todo.description, "Updated description");
}

#[tokio::test]
#[serial]
async fn test_update_todo_partial_model() {
    let partial_update = UpdateTodoPartial {
        title: Some("Partial Title".to_string()),
        description: None,
    };
    
    assert_eq!(partial_update.title, Some("Partial Title".to_string()));
    assert!(partial_update.description.is_none());
}