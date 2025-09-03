use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{error, warn, info, debug};
use uuid::Uuid;

/// Comprehensive error types for the Guardian Platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GuardianError {
    /// Configuration errors
    ConfigError {
        field: String,
        message: String,
        value: Option<String>,
    },
    
    /// Authentication and authorization errors
    AuthError {
        kind: AuthErrorKind,
        message: String,
        user_id: Option<String>,
    },
    
    /// Resource management errors
    ResourceError {
        kind: ResourceErrorKind,
        resource_type: String,
        resource_id: String,
        message: String,
    },
    
    /// External service errors
    ServiceError {
        service: String,
        operation: String,
        message: String,
        retry_after: Option<Duration>,
    },
    
    /// Validation errors
    ValidationError {
        field: String,
        value: String,
        constraint: String,
        message: String,
    },
    
    /// Network and communication errors
    NetworkError {
        endpoint: String,
        message: String,
        status_code: Option<u16>,
    },
    
    /// Database and storage errors
    StorageError {
        operation: String,
        table: Option<String>,
        message: String,
        is_retryable: bool,
    },
    
    /// Plugin and extension errors
    PluginError {
        plugin_id: String,
        operation: String,
        message: String,
        is_fatal: bool,
    },
    
    /// AI/ML model errors
    ModelError {
        model_id: String,
        operation: String,
        message: String,
        confidence: Option<f64>,
    },
    
    /// Compliance and audit errors
    ComplianceError {
        framework: String,
        rule: String,
        message: String,
        severity: ComplianceSeverity,
    },
    
    /// Rate limiting errors
    RateLimitError {
        endpoint: String,
        limit: u32,
        window: Duration,
        retry_after: Duration,
    },
    
    /// Internal system errors
    InternalError {
        component: String,
        operation: String,
        message: String,
        context: HashMap<String, String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthErrorKind {
    InvalidCredentials,
    TokenExpired,
    InsufficientPermissions,
    AccountLocked,
    MfaRequired,
    SessionExpired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceErrorKind {
    NotFound,
    AlreadyExists,
    QuotaExceeded,
    AccessDenied,
    InUse,
    Corrupted,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for GuardianError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuardianError::ConfigError { field, message, .. } => {
                write!(f, "Configuration error in field '{}': {}", field, message)
            }
            GuardianError::AuthError { kind, message, .. } => {
                write!(f, "Authentication error ({:?}): {}", kind, message)
            }
            GuardianError::ResourceError { kind, resource_type, resource_id, message } => {
                write!(f, "Resource error ({:?}) for {} '{}': {}", kind, resource_type, resource_id, message)
            }
            GuardianError::ServiceError { service, operation, message, .. } => {
                write!(f, "Service error in {} during {}: {}", service, operation, message)
            }
            GuardianError::ValidationError { field, message, .. } => {
                write!(f, "Validation error for field '{}': {}", field, message)
            }
            GuardianError::NetworkError { endpoint, message, .. } => {
                write!(f, "Network error for endpoint '{}': {}", endpoint, message)
            }
            GuardianError::StorageError { operation, table, message, .. } => {
                if let Some(table) = table {
                    write!(f, "Storage error during {} on table '{}': {}", operation, table, message)
                } else {
                    write!(f, "Storage error during {}: {}", operation, message)
                }
            }
            GuardianError::PluginError { plugin_id, operation, message, .. } => {
                write!(f, "Plugin error in '{}' during {}: {}", plugin_id, operation, message)
            }
            GuardianError::ModelError { model_id, operation, message, .. } => {
                write!(f, "Model error in '{}' during {}: {}", model_id, operation, message)
            }
            GuardianError::ComplianceError { framework, rule, message, .. } => {
                write!(f, "Compliance error in {} for rule '{}': {}", framework, rule, message)
            }
            GuardianError::RateLimitError { endpoint, limit, window, .. } => {
                write!(f, "Rate limit exceeded for endpoint '{}': {} requests per {:?}", endpoint, limit, window)
            }
            GuardianError::InternalError { component, operation, message, .. } => {
                write!(f, "Internal error in {} during {}: {}", component, operation, message)
            }
        }
    }
}

impl std::error::Error for GuardianError {}

/// Circuit breaker for external service calls
#[derive(Debug)]
pub struct CircuitBreaker {
    name: String,
    failure_threshold: u32,
    recovery_timeout: Duration,
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<RwLock<u32>>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,    // Normal operation
    Open,      // Circuit is open, failing fast
    HalfOpen,  // Testing if service has recovered
}

impl CircuitBreaker {
    pub fn new(name: String, failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            name,
            failure_threshold,
            recovery_timeout,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn call<F, T, E>(&self, operation: F) -> Result<T, GuardianError>
    where
        F: FnOnce() -> Result<T, E>,
        E: Into<GuardianError>,
    {
        let state = self.state.read().await;
        
        match *state {
            CircuitState::Open => {
                if self.should_attempt_reset().await {
                    self.transition_to_half_open().await;
                } else {
                    return Err(GuardianError::ServiceError {
                        service: self.name.clone(),
                        operation: "circuit_breaker".to_string(),
                        message: "Circuit breaker is open".to_string(),
                        retry_after: Some(self.recovery_timeout),
                    });
                }
            }
            CircuitState::HalfOpen => {
                // Allow one request to test if service has recovered
            }
            CircuitState::Closed => {
                // Normal operation
            }
        }

        drop(state);

        match operation() {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(error) => {
                let guardian_error: GuardianError = error.into();
                self.on_failure().await;
                Err(guardian_error)
            }
        }
    }

    async fn should_attempt_reset(&self) -> bool {
        let last_failure = self.last_failure_time.read().await;
        if let Some(last_failure) = *last_failure {
            last_failure.elapsed() >= self.recovery_timeout
        } else {
            true
        }
    }

    async fn transition_to_half_open(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::HalfOpen;
        info!("Circuit breaker '{}' transitioned to half-open", self.name);
    }

    async fn on_success(&self) {
        let mut state = self.state.write().await;
        let mut failure_count = self.failure_count.write().await;
        
        *state = CircuitState::Closed;
        *failure_count = 0;
        
        // Circuit breaker reset to closed state
    }

    async fn on_failure(&self) {
        let mut failure_count = self.failure_count.write().await;
        let mut last_failure_time = self.last_failure_time.write().await;
        
        *failure_count += 1;
        *last_failure_time = Some(Instant::now());
        
        if *failure_count >= self.failure_threshold {
            let mut state = self.state.write().await;
            *state = CircuitState::Open;
            warn!("Circuit breaker '{}' opened after {} failures", self.name, *failure_count);
        }
    }
}

/// Retry configuration for operations
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

/// Retry mechanism with exponential backoff
pub async fn retry_with_backoff<F, T, E>(
    operation: F,
    config: RetryConfig,
    error_handler: impl Fn(&E) -> bool, // Returns true if error is retryable
) -> Result<T, GuardianError>
where
    F: Fn() -> Result<T, E>,
    E: std::fmt::Display + std::fmt::Debug,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay;

    loop {
        attempt += 1;
        
        match operation() {
            Ok(result) => return Ok(result),
            Err(error) => {
                if attempt >= config.max_attempts || !error_handler(&error) {
                    return Err(GuardianError::InternalError {
                        component: "retry_mechanism".to_string(),
                        operation: "retry_with_backoff".to_string(),
                        message: format!("Operation failed after {} attempts: {}", attempt, error),
                        context: HashMap::new(),
                    });
                }

                warn!("Operation failed (attempt {}/{}): {}", attempt, config.max_attempts, error);
                
                // Add jitter to prevent thundering herd
                let actual_delay = if config.jitter {
                    let jitter_factor = 0.1 + (rand::random::<f64>() * 0.2); // 10-30% jitter
                    Duration::from_millis((delay.as_millis() as f64 * jitter_factor) as u64)
                } else {
                    delay
                };

                sleep(actual_delay).await;
                
                // Exponential backoff
                delay = Duration::from_millis(
                    (delay.as_millis() as f64 * config.backoff_multiplier) as u64
                ).min(config.max_delay);
            }
        }
    }
}

/// Error context for structured logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub error_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub component: String,
    pub operation: String,
    pub user_id: Option<String>,
    pub tenant_id: Option<String>,
    pub request_id: Option<String>,
    pub stack_trace: Option<String>,
    pub additional_context: HashMap<String, String>,
}

impl ErrorContext {
    pub fn new(component: String, operation: String) -> Self {
        Self {
            error_id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            component,
            operation,
            user_id: None,
            tenant_id: None,
            request_id: None,
            stack_trace: None,
            additional_context: HashMap::new(),
        }
    }

    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_tenant(mut self, tenant_id: String) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    pub fn with_context(mut self, key: String, value: String) -> Self {
        self.additional_context.insert(key, value);
        self
    }
}

/// Error recovery strategies
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry the operation
    Retry(RetryConfig),
    /// Use a fallback value
    Fallback(serde_json::Value),
    /// Use a different service/endpoint
    Failover(String),
    /// Graceful degradation
    Degrade,
    /// Fail immediately
    Fail,
}

/// Error handler with recovery strategies
pub struct ErrorHandler {
    strategies: HashMap<String, RecoveryStrategy>,
    circuit_breakers: HashMap<String, Arc<CircuitBreaker>>,
}

impl ErrorHandler {
    pub fn new() -> Self {
        Self {
            strategies: HashMap::new(),
            circuit_breakers: HashMap::new(),
        }
    }

    pub fn add_strategy(&mut self, error_type: String, strategy: RecoveryStrategy) {
        self.strategies.insert(error_type, strategy);
    }

    pub fn add_circuit_breaker(&mut self, name: String, breaker: Arc<CircuitBreaker>) {
        self.circuit_breakers.insert(name, breaker);
    }

    pub async fn handle_error<T>(
        &self,
        error: GuardianError,
        context: ErrorContext,
        recovery_operation: impl FnOnce() -> Result<T, GuardianError>,
    ) -> Result<T, GuardianError> {
        // Log the error with context
        self.log_error(&error, &context).await;

        // Determine recovery strategy
        let strategy = self.determine_strategy(&error);
        
        match strategy {
            RecoveryStrategy::Retry(config) => {
                retry_with_backoff(
                    || recovery_operation(),
                    config,
                    |_| true, // Assume all errors are retryable for now
                ).await
            }
            RecoveryStrategy::Fallback(value) => {
                // This would need to be adapted based on the return type
                Err(GuardianError::InternalError {
                    component: "error_handler".to_string(),
                    operation: "fallback_recovery".to_string(),
                    message: "Fallback recovery not implemented for this type".to_string(),
                    context: HashMap::new(),
                })
            }
            RecoveryStrategy::Failover(service) => {
                warn!("Attempting failover to service: {}", service);
                recovery_operation()
            }
            RecoveryStrategy::Degrade => {
                warn!("Degrading service functionality due to error");
                recovery_operation()
            }
            RecoveryStrategy::Fail => {
                Err(error)
            }
        }
    }

    fn determine_strategy(&self, error: &GuardianError) -> &RecoveryStrategy {
        let error_type = match error {
            GuardianError::ServiceError { .. } => "service_error",
            GuardianError::NetworkError { .. } => "network_error",
            GuardianError::StorageError { .. } => "storage_error",
            GuardianError::RateLimitError { .. } => "rate_limit_error",
            _ => "default",
        };

        self.strategies.get(error_type)
            .unwrap_or(&RecoveryStrategy::Fail)
    }

    async fn log_error(&self, error: &GuardianError, context: &ErrorContext) {
        let error_data = serde_json::json!({
            "error_id": context.error_id,
            "timestamp": context.timestamp,
            "component": context.component,
            "operation": context.operation,
            "error": error,
            "user_id": context.user_id,
            "tenant_id": context.tenant_id,
            "request_id": context.request_id,
            "additional_context": context.additional_context,
        });

        error!("Error occurred: {}", serde_json::to_string(&error_data).unwrap_or_default());
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        let mut handler = Self::new();
        
        // Add default strategies
        handler.add_strategy("service_error".to_string(), RecoveryStrategy::Retry(RetryConfig::default()));
        handler.add_strategy("network_error".to_string(), RecoveryStrategy::Retry(RetryConfig::default()));
        handler.add_strategy("rate_limit_error".to_string(), RecoveryStrategy::Retry(RetryConfig {
            max_attempts: 5,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter: true,
        }));
        
        handler
    }
}

/// Utility functions for common error scenarios
pub mod utils {
    use super::*;

    pub fn config_error(field: &str, message: &str, value: Option<&str>) -> GuardianError {
        GuardianError::ConfigError {
            field: field.to_string(),
            message: message.to_string(),
            value: value.map(|v| v.to_string()),
        }
    }

    pub fn auth_error(kind: AuthErrorKind, message: &str, user_id: Option<&str>) -> GuardianError {
        GuardianError::AuthError {
            kind,
            message: message.to_string(),
            user_id: user_id.map(|id| id.to_string()),
        }
    }

    pub fn resource_error(
        kind: ResourceErrorKind,
        resource_type: &str,
        resource_id: &str,
        message: &str,
    ) -> GuardianError {
        GuardianError::ResourceError {
            kind,
            resource_type: resource_type.to_string(),
            resource_id: resource_id.to_string(),
            message: message.to_string(),
        }
    }

    pub fn service_error(service: &str, operation: &str, message: &str) -> GuardianError {
        GuardianError::ServiceError {
            service: service.to_string(),
            operation: operation.to_string(),
            message: message.to_string(),
            retry_after: None,
        }
    }

    pub fn validation_error(field: &str, value: &str, constraint: &str, message: &str) -> GuardianError {
        GuardianError::ValidationError {
            field: field.to_string(),
            value: value.to_string(),
            constraint: constraint.to_string(),
            message: message.to_string(),
        }
    }

    pub fn network_error(endpoint: &str, message: &str, status_code: Option<u16>) -> GuardianError {
        GuardianError::NetworkError {
            endpoint: endpoint.to_string(),
            message: message.to_string(),
            status_code,
        }
    }

    pub fn storage_error(operation: &str, table: Option<&str>, message: &str, is_retryable: bool) -> GuardianError {
        GuardianError::StorageError {
            operation: operation.to_string(),
            table: table.map(|t| t.to_string()),
            message: message.to_string(),
            is_retryable,
        }
    }

    pub fn plugin_error(plugin_id: &str, operation: &str, message: &str, is_fatal: bool) -> GuardianError {
        GuardianError::PluginError {
            plugin_id: plugin_id.to_string(),
            operation: operation.to_string(),
            message: message.to_string(),
            is_fatal,
        }
    }

    pub fn model_error(model_id: &str, operation: &str, message: &str, confidence: Option<f64>) -> GuardianError {
        GuardianError::ModelError {
            model_id: model_id.to_string(),
            operation: operation.to_string(),
            message: message.to_string(),
            confidence,
        }
    }

    pub fn compliance_error(framework: &str, rule: &str, message: &str, severity: ComplianceSeverity) -> GuardianError {
        GuardianError::ComplianceError {
            framework: framework.to_string(),
            rule: rule.to_string(),
            message: message.to_string(),
            severity,
        }
    }

    pub fn rate_limit_error(endpoint: &str, limit: u32, window: Duration, retry_after: Duration) -> GuardianError {
        GuardianError::RateLimitError {
            endpoint: endpoint.to_string(),
            limit,
            window,
            retry_after,
        }
    }

    pub fn internal_error(component: &str, operation: &str, message: &str) -> GuardianError {
        GuardianError::InternalError {
            component: component.to_string(),
            operation: operation.to_string(),
            message: message.to_string(),
            context: HashMap::new(),
        }
    }
}
