use std::sync::Arc;

use todo_api::{AppState, create_app_router, config::Config, db::connection_pool, service};

#[derive(Clone)]
pub struct AppState {
    pub todo_service: Arc<service::todo::Service>,
    pub auth_service: Arc<service::auth::Service>,
    pub jwt_service: Arc<service::jwt::Service>,
}

/// Create app router for testing and production
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

    Router::new()
        // TODO routes with authentication - RESTful endpoints
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
        // Apply authentication middleware to all TODO routes
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ))
        // Authentication routes (no middleware needed)
        .route("/auth/register", post(handlers::auth::registration::handler))
        .route("/auth/login", post(handlers::auth::login::handler))
        // Health check endpoint for Cloud Run
        .route("/health", get(handlers::health::handler))
        // Add security and performance layers
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(RequestBodyLimitLayer::new(1024 * 1024)) // 1MB limit
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

async fn auth_middleware(
    State(AppState { jwt_service, .. }): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(auth_header) = req.headers().get(http::header::AUTHORIZATION) {
        let auth_header_content = auth_header.to_str().map_err(|_| StatusCode::UNAUTHORIZED)?;
        if !auth_header_content.starts_with("Bearer ") {
            return Err(StatusCode::UNAUTHORIZED);
        }
        let auth_token = auth_header_content.replace("Bearer ", "");
        let context_user = jwt_service
            .verify_token(auth_token)
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        req.extensions_mut().insert(context_user);

        return Ok(next.run(req).await);
    }

    Err(StatusCode::UNAUTHORIZED)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Cloud Run için JSON formatında loglama başlat
    tracing_subscriber::fmt()
        .json() // JSON formatında loglama için
        .init();

    tracing::info!("Yapılandırma yükleniyor...");
    // Load configuration
    let config = config::Config::from_env()?;

    // Initialize database connection pool
    let db_pool = connection_pool().await?;

    // Schema initialization - manuel schema kurulumu yapıyoruz
    tracing::info!("Veritabanı şeması başlatılıyor...");
    db::schema::initialize_schema(&db_pool).await?;
    tracing::info!("Veritabanı şeması başarıyla başlatıldı.");

    // Initialize all services with single connection pool instance
    let todo_service = Arc::new(service::todo::Service::new(db_pool.clone())?);
    let jwt_service = Arc::new(service::jwt::Service::new(&config.jwt_secret)?);
    let auth_service = Arc::new(service::auth::Service::new(
        jwt_service.clone(),
        db_pool.clone(),
        config.hashing_secret_key.clone(),
    )?);

    // Create application state
    let app_state = AppState {
        todo_service,
        auth_service,
        jwt_service: jwt_service.clone(),
    };

    // Create router
    let router = create_app_router(app_state);

    // Cloud Run için portu ortam değişkeninden oku, yoksa config'den al
    let port = std::env::var("PORT").unwrap_or_else(|_| config.server_port.to_string());
    let host = &config.server_host; // Config'den host bilgisini al

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;
    tracing::info!("Sunucu {}:{} adresinde dinlemede...", host, port);
    axum::serve(listener, router).await?;
    Ok(())
}
