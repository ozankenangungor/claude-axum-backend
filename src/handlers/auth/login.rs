use axum::{extract::State, response::IntoResponse, Json};
use validator::Validate;

use crate::{
    error::{AppError, AppResult, ErrorSeverity},
    handlers::auth::models::LoginRequest,
    AppState,
};

pub async fn handler(
    State(AppState { auth_service, .. }): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> AppResult<impl IntoResponse> {
    // Validate request
    if let Err(validation_errors) = request.validate() {
        return Err(AppError::from(validation_errors));
    }

    // Attempt login with proper error context
    let token = auth_service.login(request).await.map_err(|e| {
        let mut error = AppError::from(e);
        // Add specific context for login failures
        if let AppError::Authentication {
            ref mut context, ..
        } = error
        {
            context
                .additional_data
                .insert("operation".to_string(), "user_login".to_string());
            context.severity = ErrorSeverity::Medium;
        }
        error
    })?;

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "token": token
        }
    })))
}
