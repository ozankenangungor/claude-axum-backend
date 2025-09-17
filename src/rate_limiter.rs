// Simple rate limiting using tower-http built-in features
// For production, consider using external services like Redis for distributed rate limiting

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tracing::{info, warn};

#[derive(Clone)]
pub struct SimpleRateLimiter {
    requests: Arc<Mutex<HashMap<IpAddr, Vec<Instant>>>>,
    max_requests: usize,
    window_duration: Duration,
}

impl SimpleRateLimiter {
    pub fn new(max_requests: usize, window_duration: Duration) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window_duration,
        }
    }

    fn is_allowed(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let mut requests = match self.requests.lock() {
            Ok(guard) => guard,
            Err(_) => {
                warn!("Failed to acquire rate limiter lock for IP: {}", ip);
                return true; // Fail open for resilience
            }
        };

        let ip_requests = requests.entry(ip).or_insert_with(Vec::new);
        
        // Remove old requests outside the window
        ip_requests.retain(|&timestamp| now.duration_since(timestamp) < self.window_duration);

        // Check if rate limit is exceeded
        if ip_requests.len() >= self.max_requests {
            false
        } else {
            ip_requests.push(now);
            true
        }
    }
}

/// Global rate limiting middleware - 100 requests per minute per IP  
/// Cloud Run compatible version using headers for IP detection
pub async fn global_rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    static RATE_LIMITER: std::sync::OnceLock<SimpleRateLimiter> = std::sync::OnceLock::new();
    let limiter = RATE_LIMITER.get_or_init(|| {
        SimpleRateLimiter::new(100, Duration::from_secs(60))
    });

    // Extract IP address from headers (Cloud Run uses X-Forwarded-For)
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or_else(|| {
            info!("No client IP found in headers, using localhost fallback");
            std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
        });
    
    if !limiter.is_allowed(client_ip) {
        warn!("Global rate limit exceeded for IP: {}", client_ip);
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    info!("Request allowed for IP: {}", client_ip);
    Ok(next.run(request).await)
}

/// Auth rate limiting middleware - 10 requests per 15 minutes per IP
/// Cloud Run compatible version using headers for IP detection  
pub async fn auth_rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    static AUTH_RATE_LIMITER: std::sync::OnceLock<SimpleRateLimiter> = std::sync::OnceLock::new();
    let limiter = AUTH_RATE_LIMITER.get_or_init(|| {
        SimpleRateLimiter::new(10, Duration::from_secs(900)) // 15 minutes
    });

    // Extract IP address from headers (Cloud Run uses X-Forwarded-For)
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or_else(|| {
            info!("No client IP found in auth headers, using localhost fallback");
            std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
        });
    
    if !limiter.is_allowed(client_ip) {
        warn!("Auth rate limit exceeded for IP: {}", client_ip);
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    info!("Auth request allowed for IP: {}", client_ip);
    Ok(next.run(request).await)
}