pub mod auth;
pub mod validation;
pub mod rate_limiting;
pub mod input_sanitization;
pub mod encryption;

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
    http::StatusCode,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests: u32,
    pub window_seconds: u64,
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_seconds: 60,
            burst_size: 10,
        }
    }
}

/// Rate limiter entry
#[derive(Debug, Clone)]
struct RateLimitEntry {
    count: u32,
    window_start: DateTime<Utc>,
    last_request: DateTime<Utc>,
}

/// Rate limiter
#[derive(Debug)]
pub struct RateLimiter {
    entries: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub async fn check_rate_limit(&self, key: &str) -> Result<(), RateLimitError> {
        let now = Utc::now();
        let mut entries = self.entries.write().await;
        
        let entry = entries.get_mut(key);
        
        match entry {
            Some(entry) => {
                // Check if window has expired
                if now.signed_duration_since(entry.window_start).num_seconds() >= self.config.window_seconds as i64 {
                    // Reset window
                    entry.count = 1;
                    entry.window_start = now;
                    entry.last_request = now;
                } else {
                    // Check burst limit
                    if now.signed_duration_since(entry.last_request).num_seconds() < 1 {
                        if entry.count >= self.config.burst_size {
                            return Err(RateLimitError::BurstLimitExceeded);
                        }
                    }
                    
                    // Check rate limit
                    if entry.count >= self.config.max_requests {
                        return Err(RateLimitError::RateLimitExceeded);
                    }
                    
                    entry.count += 1;
                    entry.last_request = now;
                }
            }
            None => {
                // First request
                entries.insert(key.to_string(), RateLimitEntry {
                    count: 1,
                    window_start: now,
                    last_request: now,
                });
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Burst limit exceeded")]
    BurstLimitExceeded,
}

/// Input sanitization
pub struct InputSanitizer;

impl InputSanitizer {
    pub fn sanitize_string(input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || ".,!?@#$%^&*()_+-=[]{}|;':\",./<>?~`".contains(*c))
            .collect()
    }
    
    pub fn sanitize_filename(input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '_' || *c == '-')
            .collect()
    }
    
    pub fn sanitize_path(input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '/' || *c == '.' || *c == '_' || *c == '-')
            .collect()
    }
}

/// Security middleware
pub async fn security_middleware(
    State(rate_limiter): State<Arc<RateLimiter>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract client IP
    let client_ip = request
        .headers()
        .get("x-forwarded-for")
        .or_else(|| request.headers().get("x-real-ip"))
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");
    
    // Check rate limit
    if let Err(_) = rate_limiter.check_rate_limit(client_ip).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    
    // Continue with request
    Ok(next.run(request).await)
}
