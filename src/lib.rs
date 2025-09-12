use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{Request, State},
    http::{self, header::CONTENT_TYPE, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Router,
};
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer,
};
use tracing::{error, info};
pub mod config;
pub mod db;
pub mod handlers;
pub mod service;

#[derive(Clone)]
pub struct AppState {
    pub todo_service: Arc<service::todo::Service>,
    pub auth_service: Arc<service::auth::Service>,
    pub jwt_service: Arc<service::jwt::Service>,
    pub social_service: Arc<service::social::SocialService>,
}

async fn auth_middleware(
    State(AppState { jwt_service, .. }): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    info!(">>> Auth middleware çalıştı! Token kontrol ediliyor...");
    println!(">>> Auth middleware çalıştı! Token kontrol ediliyor...");
    if let Some(auth_header) = req.headers().get(http::header::AUTHORIZATION) {
        let auth_header_content = auth_header.to_str().map_err(|_| StatusCode::UNAUTHORIZED)?;
        if !auth_header_content.starts_with("Bearer ") {
            return Err(StatusCode::UNAUTHORIZED);
        }
        let auth_token = auth_header_content.replace("Bearer ", "");

        let verification_result = jwt_service.verify_token(auth_token);

        // 2. Eğer sonuç bir hata ise (Err), hatanın içeriğini log'lara yazdır.
        if let Err(e) = &verification_result {
            println!("!!! TOKEN DOĞRULAMA HATASI: {:?}", e);

            error!("!!! TOKEN DOĞRULAMA HATASI: {:?}", e);
        }

        let context_user = verification_result.map_err(|_| StatusCode::UNAUTHORIZED)?;

        req.extensions_mut().insert(context_user);

        return Ok(next.run(req).await);
    }

    Err(StatusCode::UNAUTHORIZED)
}

pub fn create_app_router(app_state: AppState) -> Router {
    let origin = std::env::var("FRONTEND_URL").unwrap_or_else(|_| "*".to_string());
    let allowed_origin = match origin.parse::<HeaderValue>() {
        Ok(header_value) => header_value,
        Err(_) => {
            tracing::error!(
                "Geçersiz FRONTEND_URL değeri: '{}'. Sunucu başlatılamadı.",
                origin
            );
            panic!("Geçersiz FRONTEND_URL yapılandırması.");
        }
    };

    let protected_routes = Router::new()
        .route(
            "/todos",
            get(handlers::todo::list::handler).post(handlers::todo::create::handler),
        )
        .route(
            "/todos/{id}",
            get(handlers::todo::get::handler)
                .put(handlers::todo::update::handler)
                .patch(handlers::todo::partial_update::handler)
                .delete(handlers::todo::delete::handler),
        )
        .route(
            "/posts",
            get(handlers::social::posts::get_feed).post(handlers::social::posts::create_post),
        )
        .route(
            "/posts/{id}",
            get(handlers::social::posts::get_post)
                .put(handlers::social::posts::update_post)
                .delete(handlers::social::posts::delete_post),
        )
        .route(
            "/users/{id}/posts",
            get(handlers::social::posts::get_user_posts),
        )
        .route(
            "/users/{id}/follow",
            post(handlers::social::follows::follow_user)
                .delete(handlers::social::follows::unfollow_user),
        )
        .route(
            "/users/{id}/following-status",
            get(handlers::social::follows::check_following),
        )
        .route(
            "/users/{id}/followers",
            get(handlers::social::follows::get_followers),
        )
        .route(
            "/users/{id}/following",
            get(handlers::social::follows::get_following),
        )
        .route(
            "/posts/{id}/like",
            post(handlers::social::likes::like_post).delete(handlers::social::likes::unlike_post),
        )
        .route(
            "/posts/{id}/liked",
            get(handlers::social::likes::check_liked),
        )
        .route(
            "/posts/{id}/comments",
            get(handlers::social::comments::get_post_comments)
                .post(handlers::social::comments::create_comment),
        )
        .route("/profile", get(handlers::social::profile::get_my_profile))
        .route(
            "/users/{id}/profile",
            get(handlers::social::profile::get_profile),
        )
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    let public_routes = Router::new()
        .route(
            "/auth/register",
            post(handlers::auth::registration::handler),
        )
        .route("/auth/login", post(handlers::auth::login::handler))
        .route("/health", get(handlers::health::handler));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        .layer(
            CorsLayer::new()
                .allow_origin(allowed_origin)
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::PUT,
                    axum::http::Method::PATCH,
                    axum::http::Method::DELETE,
                ])
                .allow_headers([CONTENT_TYPE]),
        )
        .with_state(app_state)
}
