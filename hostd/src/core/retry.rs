use std::time::Duration;
use tokio::time::sleep;
use tracing::{warn, error, info};
use crate::core::error_handler::AppError;

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 1000,
            max_delay_ms: 10000,
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration
    pub fn new(max_attempts: u32, base_delay_ms: u64) -> Self {
        Self {
            max_attempts,
            base_delay_ms,
            max_delay_ms: base_delay_ms * 10,
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
    
    /// Create a configuration for database operations
    pub fn database() -> Self {
        Self {
            max_attempts: 5,
            base_delay_ms: 500,
            max_delay_ms: 5000,
            backoff_multiplier: 1.5,
            jitter: true,
        }
    }
    
    /// Create a configuration for network operations
    pub fn network() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 1000,
            max_delay_ms: 10000,
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
    
    /// Create a configuration for external service calls
    pub fn external_service() -> Self {
        Self {
            max_attempts: 2,
            base_delay_ms: 2000,
            max_delay_ms: 15000,
            backoff_multiplier: 2.5,
            jitter: true,
        }
    }
    
    /// Create a configuration for file operations
    pub fn file_system() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 100,
            max_delay_ms: 2000,
            backoff_multiplier: 2.0,
            jitter: false,
        }
    }
}

/// Retry manager for handling retryable operations
#[derive(Clone)]
pub struct RetryManager {
    config: RetryConfig,
}

impl RetryManager {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }
    
    /// Execute a function with retry logic
    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>> + Send + Sync,
        E: Clone + std::fmt::Debug,
    {
        let mut attempt = 1;
        let mut last_error = None;
        
        while attempt <= self.config.max_attempts {
            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        info!("Operation succeeded on attempt {}", attempt);
                    }
                    return Ok(result);
                }
                Err(error) => {
                    last_error = Some(error.clone());
                    
                    if attempt == self.config.max_attempts {
                        error!("Operation failed after {} attempts", self.config.max_attempts);
                        break;
                    }
                    
                    let delay = self.calculate_delay(attempt);
                    warn!("Operation failed on attempt {} (of {}), retrying in {}ms: {:?}", 
                          attempt, self.config.max_attempts, delay.as_millis(), error);
                    
                    sleep(delay).await;
                    attempt += 1;
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| panic!("No errors collected but retry failed")))
    }
    
    /// Execute a function with retry logic, checking if the error is retryable
    pub async fn execute_with_retry_check<F, T>(&self, operation: F) -> Result<T, AppError>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, AppError>> + Send>> + Send + Sync,
    {
        let mut attempt = 1;
        let mut last_error = None;
        
        while attempt <= self.config.max_attempts {
            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        info!("Operation succeeded on attempt {}", attempt);
                    }
                    return Ok(result);
                }
                Err(error) => {
                    last_error = Some(error.clone());
                    
                    if !error.is_retryable() {
                        warn!("Error is not retryable: {}", error.detailed_message());
                        return Err(error);
                    }
                    
                    if attempt == self.config.max_attempts {
                        error!("Operation failed after {} attempts", self.config.max_attempts);
                        break;
                    }
                    
                    let delay = self.calculate_delay(attempt);
                    warn!("Operation failed on attempt {} (of {}), retrying in {}ms: {}", 
                          attempt, self.config.max_attempts, delay.as_millis(), error.detailed_message());
                    
                    sleep(delay).await;
                    attempt += 1;
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| panic!("No errors collected but retry failed")))
    }
    
    /// Calculate the delay for the next retry attempt
    fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_delay = self.config.base_delay_ms as f64;
        let multiplier = self.config.backoff_multiplier.powi((attempt - 1) as i32);
        let mut delay_ms = base_delay * multiplier;
        
        // Cap at max delay
        delay_ms = delay_ms.min(self.config.max_delay_ms as f64);
        
        // Add jitter if enabled
        if self.config.jitter {
            let jitter_range = delay_ms * 0.1; // 10% jitter
            let jitter = (fastrand::f64() - 0.5) * 2.0 * jitter_range;
            delay_ms += jitter;
            delay_ms = delay_ms.max(0.0);
        }
        
        Duration::from_millis(delay_ms as u64)
    }
}

/// Convenience function for retrying operations
pub async fn retry<F, T, E>(config: RetryConfig, operation: F) -> Result<T, E>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>> + Send + Sync,
    E: Clone + std::fmt::Debug,
{
    let retry_manager = RetryManager::new(config);
    retry_manager.execute(operation).await
}

/// Convenience function for retrying operations with AppError
pub async fn retry_with_app_error<F, T>(config: RetryConfig, operation: F) -> Result<T, AppError>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, AppError>> + Send>> + Send + Sync,
{
    let retry_manager = RetryManager::new(config);
    retry_manager.execute_with_retry_check(operation).await
}

/// Retry decorator for functions
pub fn with_retry<F, T, E>(config: RetryConfig) -> impl Fn(F) -> Box<dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>> + Send + Sync>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>> + Send + Sync + Clone + 'static,
    E: Clone + std::fmt::Debug + Send + 'static,
    T: Send + 'static,
{
    move |operation| {
        let retry_manager = RetryManager::new(config.clone());
        Box::new(move || {
            let retry_manager = retry_manager.clone();
            let operation = operation.clone();
            Box::pin(async move {
                retry_manager.execute(operation).await
            })
        })
    }
}

/// Circuit breaker pattern implementation
pub struct CircuitBreaker {
    failure_threshold: u32,
    timeout_duration: Duration,
    state: std::sync::Arc<std::sync::Mutex<CircuitState>>,
}

#[derive(Debug, Clone)]
enum CircuitState {
    Closed { failure_count: u32 },
    Open { opened_at: std::time::Instant },
    HalfOpen,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, timeout_duration: Duration) -> Self {
        Self {
            failure_threshold,
            timeout_duration,
            state: std::sync::Arc::new(std::sync::Mutex::new(CircuitState::Closed { failure_count: 0 })),
        }
    }
    
    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>> + Send + Sync,
        E: Clone + std::fmt::Debug + From<anyhow::Error>,
    {
        let state = self.state.clone();
        let failure_threshold = self.failure_threshold;
        let timeout_duration = self.timeout_duration;
        
        // Check circuit state
        {
            let mut circuit_state = state.lock().map_err(|_| anyhow::anyhow!("Failed to acquire circuit state lock"))?;
            match *circuit_state {
                CircuitState::Open { opened_at } => {
                    if opened_at.elapsed() < timeout_duration {
                        return Err(anyhow::anyhow!("Circuit breaker is open").into());
                    } else {
                        *circuit_state = CircuitState::HalfOpen;
                    }
                }
                CircuitState::HalfOpen => {
                    // Allow one request through
                }
                CircuitState::Closed { .. } => {
                    // Circuit is closed, proceed normally
                }
            }
        }
        
        // Execute operation
        match operation().await {
            Ok(result) => {
                // Success - reset circuit
                let mut circuit_state = state.lock().map_err(|_| anyhow::anyhow!("Failed to acquire circuit state lock"))?;
                *circuit_state = CircuitState::Closed { failure_count: 0 };
                Ok(result)
            }
            Err(error) => {
                // Failure - update circuit state
                let mut circuit_state = state.lock().map_err(|_| anyhow::anyhow!("Failed to acquire circuit state lock"))?;
                match *circuit_state {
                CircuitState::Closed { ref mut failure_count } => {
                    *failure_count += 1;
                    if *failure_count >= failure_threshold {
                        let failure_count = *failure_count;
                        *circuit_state = CircuitState::Open { opened_at: std::time::Instant::now() };
                        warn!("Circuit breaker opened after {} failures", failure_count);
                    }
                }
                    CircuitState::HalfOpen => {
                        *circuit_state = CircuitState::Open { opened_at: std::time::Instant::now() };
                        warn!("Circuit breaker opened after half-open failure");
                    }
                    CircuitState::Open { .. } => {
                        // Already open, no change needed
                    }
                }
                Err(error)
            }
        }
    }
}

/// Timeout wrapper for operations
pub async fn with_timeout<F, T>(duration: Duration, operation: F) -> Result<T, AppError>
where
    F: std::future::Future<Output = Result<T, AppError>>,
{
    tokio::time::timeout(duration, operation)
        .await
        .map_err(|_| AppError::internal_error("timeout", "Operation timed out"))?
}

/// Bulk operation with individual retry
pub async fn bulk_operation_with_retry<F, T, E>(
    items: Vec<T>,
    operation: F,
    retry_config: RetryConfig,
) -> Vec<Result<T, E>>
where
    F: Fn(T) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>> + Send + Sync + Clone + 'static,
    E: Clone + std::fmt::Debug + Send,
    T: Send + Sync + Clone + 'static,
{
    let mut results = Vec::new();
    
    for item in items {
        let operation = operation.clone();
        let retry_config = retry_config.clone();
        
        let result = retry(retry_config, move || {
            let operation = operation.clone();
            let item = item.clone();
            Box::pin(async move { operation(item).await })
        }).await;
        
        results.push(result);
    }
    
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    
    #[tokio::test]
    async fn test_retry_success() {
        let config = RetryConfig::default();
        let mut attempt = AtomicU32::new(0);
        
        let result = retry(config, || {
            let attempt = Arc::new(attempt);
            Box::pin(async move {
                let current = attempt.fetch_add(1, Ordering::SeqCst);
                if current < 2 {
                    Err("Not ready yet")
                } else {
                    Ok("Success")
                }
            })
        }).await;
        
        assert_eq!(result, Ok("Success"));
        assert_eq!(attempt.load(Ordering::SeqCst), 3);
    }
    
    #[tokio::test]
    async fn test_retry_max_attempts() {
        let config = RetryConfig::new(2, 10);
        
        let result = retry(config, || {
            Box::pin(async move {
                Err::<&str, &str>("Always fails")
            })
        }).await;
        
        assert_eq!(result, Err("Always fails"));
    }
    
    #[tokio::test]
    async fn test_circuit_breaker() {
        let breaker = CircuitBreaker::new(2, Duration::from_millis(100));
        let mut attempt = AtomicU32::new(0);
        
        // First two failures should open the circuit
        for _ in 0..2 {
            let result = breaker.execute(|| {
                let attempt = Arc::new(attempt);
                Box::pin(async move {
                    attempt.fetch_add(1, Ordering::SeqCst);
                    Err::<String, String>("Always fails".to_string())
                })
            }).await;
            assert!(result.is_err());
        }
        
        // Third attempt should fail immediately due to open circuit
        let result = breaker.execute(|| {
            Box::pin(async move {
                Ok("Should not reach here".to_string())
            })
        }).await;
        assert!(result.is_err());
        
        // Wait for timeout and try again
        tokio::time::sleep(Duration::from_millis(150)).await;
        let result = breaker.execute(|| {
            Box::pin(async move {
                Ok("Success after timeout".to_string())
            })
        }).await;
        assert_eq!(result, Ok("Success after timeout".to_string()));
    }
}
