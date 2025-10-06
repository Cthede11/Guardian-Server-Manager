use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_limit: u32,
    pub window_size: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            burst_limit: 10,
            window_size: Duration::from_secs(60),
        }
    }
}

/// Rate limiter entry
#[derive(Debug, Clone)]
struct RateLimitEntry {
    requests: Vec<Instant>,
    last_cleanup: Instant,
}

impl RateLimitEntry {
    fn new() -> Self {
        Self {
            requests: Vec::new(),
            last_cleanup: Instant::now(),
        }
    }

    /// Check if request is allowed
    fn is_allowed(&mut self, config: &RateLimitConfig) -> bool {
        let now = Instant::now();
        
        // Clean up old requests
        if now.duration_since(self.last_cleanup) > Duration::from_secs(30) {
            self.requests.retain(|&time| now.duration_since(time) < config.window_size);
            self.last_cleanup = now;
        }
        
        // Check burst limit
        if self.requests.len() >= config.burst_limit as usize {
            return false;
        }
        
        // Check rate limit
        let recent_requests = self.requests
            .iter()
            .filter(|&&time| now.duration_since(time) < config.window_size)
            .count();
        
        if recent_requests >= config.requests_per_minute as usize {
            return false;
        }
        
        // Add current request
        self.requests.push(now);
        true
    }
}

/// Rate limiter service
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

    /// Check if request is allowed for given key
    pub async fn is_allowed(&self, key: &str) -> bool {
        let mut entries = self.entries.write().await;
        let entry = entries.entry(key.to_string()).or_insert_with(RateLimitEntry::new);
        entry.is_allowed(&self.config)
    }

    /// Get remaining requests for given key
    pub async fn get_remaining(&self, key: &str) -> u32 {
        let entries = self.entries.read().await;
        if let Some(entry) = entries.get(key) {
            let now = Instant::now();
            let recent_requests = entry.requests
                .iter()
                .filter(|&&time| now.duration_since(time) < self.config.window_size)
                .count();
            
            self.config.requests_per_minute.saturating_sub(recent_requests as u32)
        } else {
            self.config.requests_per_minute
        }
    }

    /// Reset rate limit for given key
    pub async fn reset(&self, key: &str) {
        let mut entries = self.entries.write().await;
        entries.remove(key);
    }

    /// Clean up old entries
    pub async fn cleanup(&self) {
        let mut entries = self.entries.write().await;
        let now = Instant::now();
        
        entries.retain(|_, entry| {
            entry.requests.retain(|&time| now.duration_since(time) < self.config.window_size);
            !entry.requests.is_empty()
        });
    }
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    State(rate_limiter): State<RateLimiter>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract client IP or user ID for rate limiting
    let client_key = extract_client_key(&request);
    
    if !rate_limiter.is_allowed(&client_key).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    
    Ok(next.run(request).await)
}

/// Extract client key for rate limiting
fn extract_client_key(request: &Request) -> String {
    // Try to get user ID from headers first
    if let Some(user_id) = request.headers().get("X-User-ID") {
        if let Ok(user_id) = user_id.to_str() {
            return format!("user:{}", user_id);
        }
    }
    
    // Fall back to IP address
    if let Some(ip) = request.headers().get("X-Forwarded-For") {
        if let Ok(ip) = ip.to_str() {
            return format!("ip:{}", ip);
        }
    }
    
    if let Some(ip) = request.headers().get("X-Real-IP") {
        if let Ok(ip) = ip.to_str() {
            return format!("ip:{}", ip);
        }
    }
    
    // Default to a generic key
    "anonymous".to_string()
}

/// Rate limit response headers
pub fn add_rate_limit_headers(
    response: &mut Response,
    rate_limiter: &RateLimiter,
    client_key: &str,
) {
    // This would be implemented in the actual middleware
    // For now, we'll just add the headers conceptually
    let remaining = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(rate_limiter.get_remaining(client_key))
    });
    
    // Add rate limit headers
    response.headers_mut().insert(
        "X-RateLimit-Limit",
        rate_limiter.config.requests_per_minute.to_string().parse().unwrap(),
    );
    
    response.headers_mut().insert(
        "X-RateLimit-Remaining",
        remaining.to_string().parse().unwrap(),
    );
    
    response.headers_mut().insert(
        "X-RateLimit-Reset",
        (chrono::Utc::now().timestamp() + 60).to_string().parse().unwrap(),
    );
}

/// Advanced rate limiting with different limits per endpoint
pub struct AdvancedRateLimiter {
    global_limiter: RateLimiter,
    endpoint_limiters: HashMap<String, RateLimiter>,
}

impl AdvancedRateLimiter {
    pub fn new() -> Self {
        let mut endpoint_limiters = HashMap::new();
        
        // Different limits for different endpoints
        endpoint_limiters.insert(
            "/api/servers".to_string(),
            RateLimiter::new(RateLimitConfig {
                requests_per_minute: 30,
                burst_limit: 5,
                window_size: Duration::from_secs(60),
            }),
        );
        
        endpoint_limiters.insert(
            "/api/auth/login".to_string(),
            RateLimiter::new(RateLimitConfig {
                requests_per_minute: 5,
                burst_limit: 2,
                window_size: Duration::from_secs(60),
            }),
        );
        
        Self {
            global_limiter: RateLimiter::new(RateLimitConfig::default()),
            endpoint_limiters,
        }
    }
    
    pub async fn is_allowed(&self, key: &str, endpoint: &str) -> bool {
        // Check global rate limit
        if !self.global_limiter.is_allowed(key).await {
            return false;
        }
        
        // Check endpoint-specific rate limit
        if let Some(endpoint_limiter) = self.endpoint_limiters.get(endpoint) {
            endpoint_limiter.is_allowed(key).await
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_rate_limiter() {
        let config = RateLimitConfig {
            requests_per_minute: 2,
            burst_limit: 2,
            window_size: Duration::from_secs(60),
        };
        
        let rate_limiter = RateLimiter::new(config);
        
        // First request should be allowed
        assert!(rate_limiter.is_allowed("test_key").await);
        
        // Second request should be allowed
        assert!(rate_limiter.is_allowed("test_key").await);
        
        // Third request should be blocked
        assert!(!rate_limiter.is_allowed("test_key").await);
    }

    #[tokio::test]
    async fn test_rate_limiter_reset() {
        let config = RateLimitConfig {
            requests_per_minute: 1,
            burst_limit: 1,
            window_size: Duration::from_secs(1),
        };
        
        let rate_limiter = RateLimiter::new(config);
        
        // First request should be allowed
        assert!(rate_limiter.is_allowed("test_key").await);
        
        // Second request should be blocked
        assert!(!rate_limiter.is_allowed("test_key").await);
        
        // Wait for window to reset
        sleep(Duration::from_secs(2)).await;
        
        // Request should be allowed again
        assert!(rate_limiter.is_allowed("test_key").await);
    }

    #[tokio::test]
    async fn test_advanced_rate_limiter() {
        let limiter = AdvancedRateLimiter::new();
        
        // Test global rate limit
        assert!(limiter.is_allowed("test_key", "/api/servers").await);
        assert!(limiter.is_allowed("test_key", "/api/servers").await);
        
        // Test endpoint-specific rate limit
        assert!(limiter.is_allowed("test_key", "/api/auth/login").await);
        assert!(!limiter.is_allowed("test_key", "/api/auth/login").await);
    }
}