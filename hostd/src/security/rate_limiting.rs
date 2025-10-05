use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

/// Rate limiting service
pub struct RateLimiter {
    entries: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    max_requests: u32,
    window_seconds: u64,
}

#[derive(Debug, Clone)]
struct RateLimitEntry {
    count: u32,
    window_start: DateTime<Utc>,
    last_request: DateTime<Utc>,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window_seconds,
        }
    }
    
    pub async fn check_rate_limit(&self, key: &str) -> Result<(), RateLimitError> {
        let now = Utc::now();
        let mut entries = self.entries.write().await;
        
        let entry = entries.get_mut(key);
        
        match entry {
            Some(entry) => {
                // Check if window has expired
                if now.signed_duration_since(entry.window_start).num_seconds() >= self.window_seconds as i64 {
                    // Reset window
                    entry.count = 1;
                    entry.window_start = now;
                    entry.last_request = now;
                } else {
                    // Check rate limit
                    if entry.count >= self.max_requests {
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
}
