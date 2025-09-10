use criterion::{black_box, criterion_group, criterion_main, Criterion};
use chrono::Utc;
use todo_api::db::models::User;
use todo_api::service::jwt::Service as JwtService;
use todo_api::handlers::todo::models::CreateTodoRequest;

fn create_jwt_service() -> JwtService {
    JwtService::new("benchmark_secret_key_for_jwt_testing").expect("Failed to create JWT service")
}

fn create_test_user() -> User {
    User {
        id: 1,
        username: "testuser".to_string(),
        password: "password".to_string(),
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

fn jwt_token_generation_benchmark(c: &mut Criterion) {
    let jwt_service = create_jwt_service();
    let user = create_test_user();
    
    c.bench_function("jwt_token_generation", |b| {
        b.iter(|| {
            jwt_service.generate_token(black_box(&user)).unwrap()
        })
    });
}

fn jwt_token_verification_benchmark(c: &mut Criterion) {
    let jwt_service = create_jwt_service();
    let user = create_test_user();
    let token = jwt_service.generate_token(&user).unwrap();
    
    c.bench_function("jwt_token_verification", |b| {
        b.iter(|| {
            jwt_service.verify_token(black_box(token.clone())).unwrap()
        })
    });
}

fn todo_request_creation_benchmark(c: &mut Criterion) {
    c.bench_function("todo_request_creation", |b| {
        b.iter(|| {
            CreateTodoRequest {
                title: black_box("Benchmark Todo".to_string()),
                description: black_box("This is a benchmark todo description".to_string()),
            }
        })
    });
}

fn user_model_creation_benchmark(c: &mut Criterion) {
    c.bench_function("user_model_creation", |b| {
        b.iter(|| {
            User {
                id: black_box(1),
                username: black_box("benchuser".to_string()),
                password: black_box("hashed_password".to_string()),
                created: black_box(Utc::now().naive_utc()),
                updated: black_box(Utc::now().naive_utc()),
                email: Some("bench@example.com".to_string()),
                display_name: Some("Bench User".to_string()),
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
        })
    });
}

criterion_group!(
    benches,
    jwt_token_generation_benchmark,
    jwt_token_verification_benchmark,
    todo_request_creation_benchmark,
    user_model_creation_benchmark
);
criterion_main!(benches);