use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;
use tracing::{info, info_span, Instrument};

/// Request tracing middleware with performance logging
pub async fn request_metrics_middleware(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = normalize_path(uri.path());

    // Create a span for this request
    let span = info_span!(
        "http_request",
        method = %method,
        path = %path,
        version = ?request.version(),
    );

    async move {
        info!(
            method = %method,
            path = %path,
            "Processing request"
        );

        let response = next.run(request).await;
        let status = response.status();
        let duration = start.elapsed();

        info!(
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            "Request completed"
        );

        response
    }
    .instrument(span)
    .await
}

/// Normalize request path for better grouping in metrics
/// E.g., /todos/123 becomes /todos/{id}
fn normalize_path(path: &str) -> String {
    let segments: Vec<&str> = path.split('/').collect();
    let mut normalized = Vec::new();

    for segment in segments {
        if segment.is_empty() {
            continue;
        }

        // Check if segment looks like an ID (numeric or UUID-like)
        if segment.chars().all(|c| c.is_ascii_digit())
            || (segment.len() >= 8
                && segment
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-'))
        {
            normalized.push("{id}");
        } else {
            normalized.push(segment);
        }
    }

    if normalized.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", normalized.join("/"))
    }
}

/// Database query logging helper
pub fn log_db_query(query_type: &str, table: &str, duration: std::time::Duration, success: bool) {
    if success {
        info!(
            query_type = %query_type,
            table = %table,
            duration_ms = %duration.as_millis(),
            "Database query completed successfully"
        );
    } else {
        tracing::warn!(
            query_type = %query_type,
            table = %table,
            duration_ms = %duration.as_millis(),
            "Database query failed"
        );
    }
}

/// Error tracking middleware - tracks error patterns and frequencies
pub async fn error_tracking_middleware(request: Request, next: Next) -> Response {
    let start = std::time::Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = normalize_path(uri.path());

    let response = next.run(request).await;
    let status = response.status();
    let duration = start.elapsed();

    // Track error responses for monitoring
    if status.is_client_error() || status.is_server_error() {
        tracing::warn!(
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            "Error response"
        );

        // In production, you could also:
        // - Increment error counters by endpoint
        // - Track error patterns for alerting
        // - Send metrics to external monitoring systems
    }

    response
}

/// JWT operation logging helper
pub fn log_jwt_operation(operation: &str, success: bool, duration: std::time::Duration) {
    info!(
        operation = %operation,
        success = %success,
        duration_ms = %duration.as_millis(),
        "JWT operation completed"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("/todos/123"), "/todos/{id}");
        assert_eq!(normalize_path("/users/456/posts"), "/users/{id}/posts");
        assert_eq!(normalize_path("/auth/login"), "/auth/login");
        assert_eq!(normalize_path("/"), "/");
        assert_eq!(normalize_path(""), "/");
        assert_eq!(
            normalize_path("/users/abc-123-def-456/profile"),
            "/users/{id}/profile"
        );
    }
}
