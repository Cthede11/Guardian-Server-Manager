use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{warn, error, info, debug};
use serde::{Deserialize, Serialize};
use crate::core::error_handler::AppError;

/// Configuration for retry and backoff behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries (caps exponential backoff)
    pub max_delay: Duration,
    /// Multiplier for exponential backoff (e.g., 2.0 for doubling)
    pub backoff_multiplier: f64,
    /// Jitter factor to add randomness (0.0 to 1.0)
    pub jitter_factor: f64,
    /// Whether to use exponential backoff
    pub use_exponential_backoff: bool,
    /// Whether to reset backoff on success
    pub reset_on_success: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
            use_exponential_backoff: true,
            reset_on_success: true,
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration
    pub fn new(max_attempts: u32, initial_delay: Duration) -> Self {
        Self {
            max_attempts,
            initial_delay,
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
            use_exponential_backoff: true,
            reset_on_success: true,
        }
    }

    /// Create a configuration for crash recovery (more aggressive)
    pub fn crash_recovery() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter_factor: 0.2,
            use_exponential_backoff: true,
            reset_on_success: true,
        }
    }

    /// Create a configuration for network operations
    pub fn network_operation() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(2000),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
            use_exponential_backoff: true,
            reset_on_success: true,
        }
    }

    /// Create a configuration for database operations
    pub fn database_operation() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 1.5,
            jitter_factor: 0.05,
            use_exponential_backoff: true,
            reset_on_success: true,
        }
    }

    /// Create a configuration for external service calls
    pub fn external_service() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_millis(5000),
            max_delay: Duration::from_secs(120),
            backoff_multiplier: 2.0,
            jitter_factor: 0.15,
            use_exponential_backoff: true,
            reset_on_success: true,
        }
    }
}

/// Retry statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryStats {
    pub total_attempts: u32,
    pub successful_attempts: u32,
    pub failed_attempts: u32,
    pub total_duration: Duration,
    pub last_attempt_duration: Duration,
    pub average_attempt_duration: Duration,
    pub backoff_delays: Vec<Duration>,
}

impl Default for RetryStats {
    fn default() -> Self {
        Self {
            total_attempts: 0,
            successful_attempts: 0,
            failed_attempts: 0,
            total_duration: Duration::ZERO,
            last_attempt_duration: Duration::ZERO,
            average_attempt_duration: Duration::ZERO,
            backoff_delays: Vec::new(),
        }
    }
}

/// Retry manager for handling retry logic with exponential backoff
pub struct RetryManager {
    config: RetryConfig,
    stats: RetryStats,
    current_attempt: u32,
    last_success_time: Option<Instant>,
}

impl RetryManager {
    /// Create a new retry manager with the given configuration
    pub fn new(config: RetryConfig) -> Self {
        Self {
            config,
            stats: RetryStats::default(),
            current_attempt: 0,
            last_success_time: None,
        }
    }

    /// Execute a function with retry logic
    pub async fn execute<F, Fut, T, E>(&mut self, operation: F) -> Result<T, E>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Debug + Clone,
    {
        let start_time = Instant::now();
        let mut last_error: Option<E> = None;

        for attempt in 1..=self.config.max_attempts {
            self.current_attempt = attempt;
            let attempt_start = Instant::now();

            debug!("Retry attempt {}/{}", attempt, self.config.max_attempts);

            match operation().await {
                Ok(result) => {
                    let attempt_duration = attempt_start.elapsed();
                    self.stats.successful_attempts += 1;
                    self.stats.last_attempt_duration = attempt_duration;
                    self.stats.total_duration = start_time.elapsed();
                    self.stats.total_attempts = attempt;
                    self.last_success_time = Some(Instant::now());

                    if attempt > 1 {
                        info!("Operation succeeded on attempt {}", attempt);
                    }

                    return Ok(result);
                }
                Err(error) => {
                    let attempt_duration = attempt_start.elapsed();
                    self.stats.failed_attempts += 1;
                    self.stats.last_attempt_duration = attempt_duration;
                    last_error = Some(error.clone());

                    if attempt < self.config.max_attempts {
                        let delay = self.calculate_delay(attempt);
                        self.stats.backoff_delays.push(delay);
                        
                        warn!(
                            "Operation failed on attempt {}/{}, retrying in {:?}: {:?}",
                            attempt, self.config.max_attempts, delay, error
                        );

                        sleep(delay).await;
                    } else {
                        error!(
                            "Operation failed after {} attempts: {:?}",
                            self.config.max_attempts, error
                        );
                    }
                }
            }
        }

        self.stats.total_duration = start_time.elapsed();
        self.stats.average_attempt_duration = if self.stats.total_attempts > 0 {
            Duration::from_millis(
                self.stats.total_duration.as_millis() as u64 / self.stats.total_attempts as u64
            )
        } else {
            Duration::ZERO
        };

        Err(last_error.expect("Should have at least one error"))
    }

    /// Calculate the delay for the next retry attempt
    fn calculate_delay(&self, attempt: u32) -> Duration {
        if !self.config.use_exponential_backoff {
            return self.config.initial_delay;
        }

        // Calculate exponential backoff
        let base_delay = self.config.initial_delay.as_millis() as f64;
        let multiplier = self.config.backoff_multiplier.powi((attempt - 1) as i32);
        let delay_ms = base_delay * multiplier;

        // Cap at max delay
        let max_delay_ms = self.config.max_delay.as_millis() as f64;
        let capped_delay_ms = delay_ms.min(max_delay_ms);

        // Add jitter to prevent thundering herd
        let jitter_range = capped_delay_ms * self.config.jitter_factor;
        let jitter = (rand::random::<f64>() - 0.5) * 2.0 * jitter_range;
        let final_delay_ms = (capped_delay_ms + jitter).max(1.0);

        Duration::from_millis(final_delay_ms as u64)
    }

    /// Get current retry statistics
    pub fn get_stats(&self) -> &RetryStats {
        &self.stats
    }

    /// Reset the retry manager (useful for periodic operations)
    pub fn reset(&mut self) {
        self.stats = RetryStats::default();
        self.current_attempt = 0;
    }

    /// Check if we should retry based on the error type
    pub fn should_retry(&self, error: &AppError) -> bool {
        error.is_retryable()
    }

    /// Get the recommended delay for a specific error
    pub fn get_error_delay(&self, error: &AppError) -> Duration {
        Duration::from_millis(error.retry_delay_ms())
    }
}

/// Convenience function for executing operations with retry
pub async fn with_retry<F, Fut, T, E>(
    operation: F,
    config: RetryConfig,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug + Clone,
{
    let mut retry_manager = RetryManager::new(config);
    retry_manager.execute(operation).await
}

/// Convenience function for crash recovery retry
pub async fn with_crash_recovery_retry<F, Fut, T, E>(
    operation: F,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug + Clone,
{
    with_retry(operation, RetryConfig::crash_recovery()).await
}

/// Convenience function for network operation retry
pub async fn with_network_retry<F, Fut, T, E>(
    operation: F,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug + Clone,
{
    with_retry(operation, RetryConfig::network_operation()).await
}

/// Convenience function for database operation retry
pub async fn with_database_retry<F, Fut, T, E>(
    operation: F,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug + Clone,
{
    with_retry(operation, RetryConfig::database_operation()).await
}

/// Convenience function for external service retry
pub async fn with_external_service_retry<F, Fut, T, E>(
    operation: F,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug + Clone,
{
    with_retry(operation, RetryConfig::external_service()).await
}

/// Circuit breaker for preventing cascading failures
pub struct CircuitBreaker {
    failure_threshold: u32,
    recovery_timeout: Duration,
    state: CircuitState,
    failure_count: u32,
    last_failure_time: Option<Instant>,
    success_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
enum CircuitState {
    Closed,    // Normal operation
    Open,      // Circuit is open, failing fast
    HalfOpen,  // Testing if service has recovered
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            failure_threshold,
            recovery_timeout,
            state: CircuitState::Closed,
            failure_count: 0,
            last_failure_time: None,
            success_count: 0,
        }
    }

    /// Execute an operation through the circuit breaker
    pub async fn execute<F, Fut, T>(&mut self, operation: F) -> Result<T, anyhow::Error>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, anyhow::Error>>,
    {
        match self.state {
            CircuitState::Open => {
                if self.should_attempt_reset() {
                    self.state = CircuitState::HalfOpen;
                    self.success_count = 0;
                    debug!("Circuit breaker transitioning to half-open state");
                } else {
                    return Err(anyhow::anyhow!("Circuit breaker is open"));
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited attempts in half-open state
                if self.success_count >= 3 {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    debug!("Circuit breaker transitioning to closed state");
                }
            }
            CircuitState::Closed => {
                // Normal operation
            }
        }

        match operation().await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(error) => {
                self.on_failure();
                Err(error)
            }
        }
    }

    fn should_attempt_reset(&self) -> bool {
        if let Some(last_failure) = self.last_failure_time {
            last_failure.elapsed() >= self.recovery_timeout
        } else {
            true
        }
    }

    fn on_success(&mut self) {
        self.success_count += 1;
        if self.state == CircuitState::HalfOpen {
            if self.success_count >= 3 {
                self.state = CircuitState::Closed;
                self.failure_count = 0;
                info!("Circuit breaker closed after successful recovery");
            }
        }
    }

    fn on_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
            warn!("Circuit breaker opened due to failure threshold reached");
        }
    }

    /// Get the current state of the circuit breaker
    pub fn state(&self) -> &CircuitState {
        &self.state
    }

    /// Get failure statistics
    pub fn failure_count(&self) -> u32 {
        self.failure_count
    }

    /// Reset the circuit breaker
    pub fn reset(&mut self) {
        self.state = CircuitState::Closed;
        self.failure_count = 0;
        self.success_count = 0;
        self.last_failure_time = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_success_on_first_attempt() {
        let config = RetryConfig::default();
        let mut retry_manager = RetryManager::new(config);

        let result = retry_manager.execute(|| async { Ok::<i32, String>(42) }).await;
        assert_eq!(result, Ok(42));
        assert_eq!(retry_manager.get_stats().total_attempts, 1);
        assert_eq!(retry_manager.get_stats().successful_attempts, 1);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let config = RetryConfig::new(3, Duration::from_millis(10));
        let mut retry_manager = RetryManager::new(config);
        let attempt_count = Arc::new(AtomicU32::new(0));

        let attempt_count_clone = attempt_count.clone();
        let result = retry_manager.execute(|| {
            let count = attempt_count_clone.clone();
            async move {
                let current = count.fetch_add(1, Ordering::SeqCst);
                if current < 2 {
                    Err::<i32, String>("Not ready yet".to_string())
                } else {
                    Ok(42)
                }
            }
        }).await;

        assert_eq!(result, Ok(42));
        assert_eq!(retry_manager.get_stats().total_attempts, 3);
        assert_eq!(retry_manager.get_stats().successful_attempts, 1);
    }

    #[tokio::test]
    async fn test_retry_exhausts_attempts() {
        let config = RetryConfig::new(2, Duration::from_millis(10));
        let mut retry_manager = RetryManager::new(config);

        let result = retry_manager.execute(|| async { Err::<i32, String>("Always fails".to_string()) }).await;
        assert!(result.is_err());
        assert_eq!(retry_manager.get_stats().total_attempts, 2);
        assert_eq!(retry_manager.get_stats().failed_attempts, 2);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_threshold() {
        let mut circuit_breaker = CircuitBreaker::new(2, Duration::from_secs(1));

        // First failure
        let result = circuit_breaker.execute(|| async { Err::<i32, String>("Fail".to_string()) }).await;
        assert!(result.is_err());
        assert_eq!(circuit_breaker.state(), &CircuitState::Closed);

        // Second failure should open the circuit
        let result = circuit_breaker.execute(|| async { Err::<i32, String>("Fail".to_string()) }).await;
        assert!(result.is_err());
        assert_eq!(circuit_breaker.state(), &CircuitState::Open);
    }
}
