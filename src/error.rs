use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::collections::HashMap;
use thiserror::Error;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Error severity levels for logging and alerting
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,      // Non-critical errors, expected failures
    Medium,   // Important errors that need attention
    High,     // Critical errors affecting service availability
    Critical, // System-wide failures requiring immediate action
}

/// Error context for better debugging and monitoring
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub correlation_id: String,
    pub user_id: Option<i32>,
    pub request_path: Option<String>,
    pub severity: ErrorSeverity,
    pub additional_data: HashMap<String, String>,
}

impl ErrorContext {
    pub fn new() -> Self {
        Self {
            correlation_id: Uuid::new_v4().to_string(),
            user_id: None,
            request_path: None,
            severity: ErrorSeverity::Medium,
            additional_data: HashMap::new(),
        }
    }

    pub fn with_user_id(mut self, user_id: i32) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_path(mut self, path: String) -> Self {
        self.request_path = Some(path);
        self
    }

    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_data(mut self, key: String, value: String) -> Self {
        self.additional_data.insert(key, value);
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Production-ready error types with context and severity
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {message}")]
    Database {
        message: String,
        #[source]
        source: sqlx::Error,
        context: ErrorContext,
    },

    #[error("Authentication failed: {message}")]
    Authentication {
        message: String,
        context: ErrorContext,
    },

    #[error("Authorization denied: {message}")]
    Authorization {
        message: String,
        context: ErrorContext,
    },

    #[error("Validation failed: {message}")]
    Validation {
        message: String,
        field_errors: Option<HashMap<String, Vec<String>>>,
        context: ErrorContext,
    },

    #[error("Resource not found: {resource}")]
    NotFound {
        resource: String,
        context: ErrorContext,
    },

    #[error("Resource conflict: {message}")]
    Conflict {
        message: String,
        context: ErrorContext,
    },

    #[error("Rate limit exceeded")]
    RateLimitExceeded {
        retry_after: Option<u64>,
        context: ErrorContext,
    },

    #[error("Internal server error: {message}")]
    Internal {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        context: ErrorContext,
    },

    #[error("Bad request: {message}")]
    BadRequest {
        message: String,
        context: ErrorContext,
    },

    #[error("Service temporarily unavailable: {message}")]
    ServiceUnavailable {
        message: String,
        retry_after: Option<u64>,
        context: ErrorContext,
    },

    #[error("External service error: {service}")]
    ExternalService {
        service: String,
        message: String,
        context: ErrorContext,
    },

    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
        context: ErrorContext,
    },
}

impl AppError {
    /// Get the error context
    pub fn context(&self) -> &ErrorContext {
        match self {
            AppError::Database { context, .. } => context,
            AppError::Authentication { context, .. } => context,
            AppError::Authorization { context, .. } => context,
            AppError::Validation { context, .. } => context,
            AppError::NotFound { context, .. } => context,
            AppError::Conflict { context, .. } => context,
            AppError::RateLimitExceeded { context, .. } => context,
            AppError::Internal { context, .. } => context,
            AppError::BadRequest { context, .. } => context,
            AppError::ServiceUnavailable { context, .. } => context,
            AppError::ExternalService { context, .. } => context,
            AppError::Configuration { context, .. } => context,
        }
    }

    /// Log the error with appropriate level based on severity
    pub fn log(&self) {
        let context = self.context();
        let correlation_id = &context.correlation_id;
        let error_msg = self.to_string();

        match context.severity {
            ErrorSeverity::Low => {
                info!(
                    correlation_id = %correlation_id,
                    user_id = ?context.user_id,
                    path = ?context.request_path,
                    error = %error_msg,
                    "Low severity error occurred"
                );
            }
            ErrorSeverity::Medium => {
                warn!(
                    correlation_id = %correlation_id,
                    user_id = ?context.user_id,
                    path = ?context.request_path,
                    error = %error_msg,
                    additional_data = ?context.additional_data,
                    "Medium severity error occurred"
                );
            }
            ErrorSeverity::High => {
                error!(
                    correlation_id = %correlation_id,
                    user_id = ?context.user_id,
                    path = ?context.request_path,
                    error = %error_msg,
                    additional_data = ?context.additional_data,
                    "High severity error occurred"
                );
            }
            ErrorSeverity::Critical => {
                error!(
                    correlation_id = %correlation_id,
                    user_id = ?context.user_id,
                    path = ?context.request_path,
                    error = %error_msg,
                    additional_data = ?context.additional_data,
                    "CRITICAL ERROR: Immediate attention required"
                );
                // In production, this could trigger alerts (Slack, PagerDuty, etc.)
            }
        }
    }

    /// Create a standardized error response
    fn error_response(&self) -> (StatusCode, Json<serde_json::Value>) {
        let context = self.context();
        let (status, code, user_message) = match self {
            AppError::Database { .. } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                "A database error occurred. Please try again later.",
            ),
            AppError::Authentication { message, .. } => (
                StatusCode::UNAUTHORIZED,
                "AUTHENTICATION_ERROR",
                message.as_str(),
            ),
            AppError::Authorization { message, .. } => (
                StatusCode::FORBIDDEN,
                "AUTHORIZATION_ERROR",
                message.as_str(),
            ),
            AppError::Validation {
                message,
                field_errors,
                ..
            } => {
                let response = if let Some(errors) = field_errors {
                    json!({
                        "error": {
                            "code": "VALIDATION_ERROR",
                            "message": message,
                            "correlation_id": context.correlation_id,
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                            "field_errors": errors
                        }
                    })
                } else {
                    json!({
                        "error": {
                            "code": "VALIDATION_ERROR",
                            "message": message,
                            "correlation_id": context.correlation_id,
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        }
                    })
                };
                return (StatusCode::BAD_REQUEST, Json(response));
            }
            AppError::NotFound { .. } => (StatusCode::NOT_FOUND, "NOT_FOUND", "Resource not found"),
            AppError::Conflict { message, .. } => {
                (StatusCode::CONFLICT, "CONFLICT", message.as_str())
            }
            AppError::RateLimitExceeded { retry_after, .. } => {
                let mut response = json!({
                    "error": {
                        "code": "RATE_LIMIT_EXCEEDED",
                        "message": "Rate limit exceeded. Please try again later.",
                        "correlation_id": context.correlation_id,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }
                });

                if let Some(retry) = retry_after {
                    response["error"]["retry_after_seconds"] = json!(retry);
                }

                return (StatusCode::TOO_MANY_REQUESTS, Json(response));
            }
            AppError::Internal { .. } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "An internal error occurred. Please try again later.",
            ),
            AppError::BadRequest { message, .. } => {
                (StatusCode::BAD_REQUEST, "BAD_REQUEST", message.as_str())
            }
            AppError::ServiceUnavailable {
                message,
                retry_after,
                ..
            } => {
                let mut response = json!({
                    "error": {
                        "code": "SERVICE_UNAVAILABLE",
                        "message": message,
                        "correlation_id": context.correlation_id,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }
                });

                if let Some(retry) = retry_after {
                    response["error"]["retry_after_seconds"] = json!(retry);
                }

                return (StatusCode::SERVICE_UNAVAILABLE, Json(response));
            }
            AppError::ExternalService {
                service: _,
                message,
                ..
            } => (
                StatusCode::BAD_GATEWAY,
                "EXTERNAL_SERVICE_ERROR",
                message.as_str(),
            ),
            AppError::Configuration { .. } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "CONFIGURATION_ERROR",
                "Service configuration error",
            ),
        };

        let body = Json(json!({
            "error": {
                "code": code,
                "message": user_message,
                "correlation_id": context.correlation_id,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        }));

        (status, body)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Log the error
        self.log();

        // Return the response
        let (status, body) = self.error_response();
        (status, body).into_response()
    }
}

// Error creation helpers for common cases
impl AppError {
    pub fn database(source: sqlx::Error) -> Self {
        Self::Database {
            message: "Database operation failed".to_string(),
            source,
            context: ErrorContext::new().with_severity(ErrorSeverity::High),
        }
    }

    pub fn auth_failed(message: &str) -> Self {
        Self::Authentication {
            message: message.to_string(),
            context: ErrorContext::new().with_severity(ErrorSeverity::Medium),
        }
    }

    pub fn forbidden(message: &str) -> Self {
        Self::Authorization {
            message: message.to_string(),
            context: ErrorContext::new().with_severity(ErrorSeverity::Medium),
        }
    }

    pub fn not_found(resource: &str) -> Self {
        Self::NotFound {
            resource: resource.to_string(),
            context: ErrorContext::new().with_severity(ErrorSeverity::Low),
        }
    }

    pub fn conflict(message: &str) -> Self {
        Self::Conflict {
            message: message.to_string(),
            context: ErrorContext::new().with_severity(ErrorSeverity::Medium),
        }
    }

    pub fn validation(message: &str) -> Self {
        Self::Validation {
            message: message.to_string(),
            field_errors: None,
            context: ErrorContext::new().with_severity(ErrorSeverity::Low),
        }
    }

    pub fn validation_with_fields(
        message: &str,
        field_errors: HashMap<String, Vec<String>>,
    ) -> Self {
        Self::Validation {
            message: message.to_string(),
            field_errors: Some(field_errors),
            context: ErrorContext::new().with_severity(ErrorSeverity::Low),
        }
    }

    pub fn internal(message: &str) -> Self {
        Self::Internal {
            message: message.to_string(),
            source: None,
            context: ErrorContext::new().with_severity(ErrorSeverity::High),
        }
    }

    pub fn rate_limit() -> Self {
        Self::RateLimitExceeded {
            retry_after: Some(60), // Default 60 seconds
            context: ErrorContext::new().with_severity(ErrorSeverity::Medium),
        }
    }
}

// Convert various error types to AppError
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::database(err)
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let mut field_errors = HashMap::new();

        for (field, field_error_list) in errors.field_errors() {
            let messages: Vec<String> = field_error_list
                .iter()
                .map(|error| {
                    error
                        .message
                        .as_ref()
                        .map(|msg| msg.to_string())
                        .unwrap_or_else(|| format!("Invalid value for field '{}'", field))
                })
                .collect();

            field_errors.insert(field.to_string(), messages);
        }

        AppError::validation_with_fields("Validation failed", field_errors)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::internal(&err.to_string())
    }
}

impl From<crate::service::jwt::Error> for AppError {
    fn from(err: crate::service::jwt::Error) -> Self {
        match err {
            crate::service::jwt::Error::JWT(_) => AppError::auth_failed("Invalid or expired token"),
            _ => AppError::internal(&err.to_string()),
        }
    }
}

impl From<crate::service::auth::Error> for AppError {
    fn from(err: crate::service::auth::Error) -> Self {
        match err {
            crate::service::auth::Error::UsernameAlreadyExists(username) => {
                AppError::conflict(&format!("Username '{}' already exists", username))
            }
            crate::service::auth::Error::UserNotFound => {
                AppError::auth_failed("Invalid credentials")
            }
            crate::service::auth::Error::InvalidPassword => {
                AppError::auth_failed("Invalid credentials")
            }
            crate::service::auth::Error::WeakPassword(msg) => AppError::validation(&msg),
            crate::service::auth::Error::Sqlx(e) => AppError::database(e),
            _ => AppError::internal(&err.to_string()),
        }
    }
}

/// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;

/// Error correlation middleware to add correlation IDs to requests
pub async fn error_correlation_middleware(mut request: Request, next: Next) -> Response {
    let correlation_id = Uuid::new_v4().to_string();

    // Add correlation ID to request headers for downstream services
    request.headers_mut().insert(
        "x-correlation-id",
        correlation_id
            .parse()
            .unwrap_or_else(|_| "invalid".parse().unwrap()),
    );

    let response = next.run(request).await;

    // Add correlation ID to response headers
    let mut response = response;
    response.headers_mut().insert(
        "x-correlation-id",
        correlation_id
            .parse()
            .unwrap_or_else(|_| "invalid".parse().unwrap()),
    );

    response
}

/// Helper trait for adding context to errors
pub trait WithErrorContext<T> {
    fn with_context(self, context: ErrorContext) -> Result<T, AppError>;
    fn with_user_context(self, user_id: i32, path: &str) -> Result<T, AppError>;
}

impl<T, E> WithErrorContext<T> for Result<T, E>
where
    E: Into<AppError>,
{
    fn with_context(self, _context: ErrorContext) -> Result<T, AppError> {
        self.map_err(|e| {
            let error = e.into();
            // Update context (simplified - in real implementation you'd merge contexts)
            error
        })
    }

    fn with_user_context(self, user_id: i32, path: &str) -> Result<T, AppError> {
        let context = ErrorContext::new()
            .with_user_id(user_id)
            .with_path(path.to_string());
        self.with_context(context)
    }
}
