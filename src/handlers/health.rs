use axum::{
    extract::State,
    response::{IntoResponse, Json},
};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    error::{AppError, AppResult, ErrorContext, ErrorSeverity},
    AppState,
};

pub async fn handler(State(app_state): State<AppState>) -> AppResult<impl IntoResponse> {
    // Check database connectivity with proper error handling
    match crate::db::health_check(&app_state.todo_service.get_pool()).await {
        Ok(_) => {
            // Return healthy status
            Ok(Json(json!({
                "status": "healthy",
                "timestamp": SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                "service": "todo_api",
                "version": env!("CARGO_PKG_VERSION"),
                "database": "healthy",
                "uptime": "running"
            })))
        }
        Err(e) => {
            // Return service unavailable error with proper context
            Err(AppError::ServiceUnavailable {
                message: "Database health check failed".to_string(),
                retry_after: Some(30), // Retry after 30 seconds
                context: ErrorContext::new()
                    .with_severity(ErrorSeverity::Critical)
                    .with_data("database_error".to_string(), e.to_string())
                    .with_data("check_type".to_string(), "health_check".to_string()),
            })
        }
    }
}
