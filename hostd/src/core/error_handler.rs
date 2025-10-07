use std::fmt;
use std::error::Error as StdError;
use serde::{Deserialize, Serialize};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use tracing::{error, warn, info};

/// Comprehensive error types for the Guardian Server Manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppError {
    // Database errors
    DatabaseError {
        message: String,
        operation: String,
        table: Option<String>,
    },
    
    // Authentication errors
    AuthenticationError {
        message: String,
        reason: AuthErrorReason,
    },
    
    // Authorization errors
    AuthorizationError {
        message: String,
        required_permission: String,
        user_role: String,
    },
    
    // Server management errors
    ServerError {
        message: String,
        server_id: String,
        operation: String,
    },
    
    // File system errors
    FileSystemError {
        message: String,
        path: String,
        operation: String,
    },
    
    // Network errors
    NetworkError {
        message: String,
        endpoint: String,
        status_code: Option<u16>,
    },
    
    // Configuration errors
    ConfigurationError {
        message: String,
        config_key: String,
        expected_type: String,
    },
    
    // Validation errors
    ValidationError {
        message: String,
        field: String,
        value: String,
        constraint: String,
    },
    
    // Process management errors
    ProcessError {
        message: String,
        process_id: Option<u32>,
        operation: String,
    },
    
    // WebSocket errors
    WebSocketError {
        message: String,
        connection_id: Option<String>,
        event_type: Option<String>,
    },
    
    // Backup errors
    BackupError {
        message: String,
        backup_id: Option<String>,
        operation: String,
    },
    
    // Modpack errors
    ModpackError {
        message: String,
        modpack_id: Option<String>,
        operation: String,
    },
    
    // Internal errors
    InternalError {
        message: String,
        component: String,
        details: Option<String>,
    },
    
    // External service errors
    ExternalServiceError {
        message: String,
        service: String,
        status_code: Option<u16>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthErrorReason {
    InvalidCredentials,
    TokenExpired,
    TokenInvalid,
    UserNotFound,
    UserInactive,
    InsufficientPermissions,
    SessionExpired,
    RateLimitExceeded,
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::InternalError {
            message: err.to_string(),
            component: "unknown".to_string(),
            details: Some(format!("anyhow error: {}", err)),
        }
    }
}

impl From<uuid::Error> for AppError {
    fn from(err: uuid::Error) -> Self {
        AppError::ValidationError {
            message: err.to_string(),
            field: "uuid".to_string(),
            value: "invalid".to_string(),
            constraint: "valid UUID format".to_string(),
        }
    }
}

impl AppError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::DatabaseError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::AuthenticationError { reason, .. } => match reason {
                AuthErrorReason::InvalidCredentials => StatusCode::UNAUTHORIZED,
                AuthErrorReason::TokenExpired => StatusCode::UNAUTHORIZED,
                AuthErrorReason::TokenInvalid => StatusCode::UNAUTHORIZED,
                AuthErrorReason::UserNotFound => StatusCode::UNAUTHORIZED,
                AuthErrorReason::UserInactive => StatusCode::FORBIDDEN,
                AuthErrorReason::InsufficientPermissions => StatusCode::FORBIDDEN,
                AuthErrorReason::SessionExpired => StatusCode::UNAUTHORIZED,
                AuthErrorReason::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            },
            AppError::AuthorizationError { .. } => StatusCode::FORBIDDEN,
            AppError::ServerError { .. } => StatusCode::BAD_REQUEST,
            AppError::FileSystemError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NetworkError { .. } => StatusCode::BAD_GATEWAY,
            AppError::ConfigurationError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            AppError::ProcessError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::WebSocketError { .. } => StatusCode::BAD_REQUEST,
            AppError::BackupError { .. } => StatusCode::BAD_REQUEST,
            AppError::ModpackError { .. } => StatusCode::BAD_REQUEST,
            AppError::InternalError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ExternalServiceError { .. } => StatusCode::BAD_GATEWAY,
        }
    }
    
    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            AppError::DatabaseError { message, .. } => {
                format!("Database operation failed: {}", message)
            }
            AppError::AuthenticationError { message, .. } => message.clone(),
            AppError::AuthorizationError { message, .. } => message.clone(),
            AppError::ServerError { message, .. } => {
                format!("Server operation failed: {}", message)
            }
            AppError::FileSystemError { message, .. } => {
                format!("File system error: {}", message)
            }
            AppError::NetworkError { message, .. } => {
                format!("Network error: {}", message)
            }
            AppError::ConfigurationError { message, .. } => {
                format!("Configuration error: {}", message)
            }
            AppError::ValidationError { message, .. } => {
                format!("Validation error: {}", message)
            }
            AppError::ProcessError { message, .. } => {
                format!("Process error: {}", message)
            }
            AppError::WebSocketError { message, .. } => {
                format!("WebSocket error: {}", message)
            }
            AppError::BackupError { message, .. } => {
                format!("Backup error: {}", message)
            }
            AppError::ModpackError { message, .. } => {
                format!("Modpack error: {}", message)
            }
            AppError::InternalError { message, .. } => {
                format!("Internal error: {}", message)
            }
            AppError::ExternalServiceError { message, .. } => {
                format!("External service error: {}", message)
            }
        }
    }
    
    /// Get a detailed error message for logging
    pub fn detailed_message(&self) -> String {
        match self {
            AppError::DatabaseError { message, operation, table } => {
                format!("Database error in operation '{}' on table '{:?}': {}", operation, table, message)
            }
            AppError::AuthenticationError { message, reason } => {
                format!("Authentication error ({:?}): {}", reason, message)
            }
            AppError::AuthorizationError { message, required_permission, user_role } => {
                format!("Authorization error: {} (required: {}, user role: {})", message, required_permission, user_role)
            }
            AppError::ServerError { message, server_id, operation } => {
                format!("Server error for server '{}' in operation '{}': {}", server_id, operation, message)
            }
            AppError::FileSystemError { message, path, operation } => {
                format!("File system error in operation '{}' on path '{}': {}", operation, path, message)
            }
            AppError::NetworkError { message, endpoint, status_code } => {
                format!("Network error for endpoint '{}' (status: {:?}): {}", endpoint, status_code, message)
            }
            AppError::ConfigurationError { message, config_key, expected_type } => {
                format!("Configuration error for key '{}' (expected type: {}): {}", config_key, expected_type, message)
            }
            AppError::ValidationError { message, field, value, constraint } => {
                format!("Validation error for field '{}' with value '{}' (constraint: {}): {}", field, value, constraint, message)
            }
            AppError::ProcessError { message, process_id, operation } => {
                format!("Process error for PID {:?} in operation '{}': {}", process_id, operation, message)
            }
            AppError::WebSocketError { message, connection_id, event_type } => {
                format!("WebSocket error for connection {:?} (event: {:?}): {}", connection_id, event_type, message)
            }
            AppError::BackupError { message, backup_id, operation } => {
                format!("Backup error for backup {:?} in operation '{}': {}", backup_id, operation, message)
            }
            AppError::ModpackError { message, modpack_id, operation } => {
                format!("Modpack error for modpack {:?} in operation '{}': {}", modpack_id, operation, message)
            }
            AppError::InternalError { message, component, details } => {
                format!("Internal error in component '{}' (details: {:?}): {}", component, details, message)
            }
            AppError::ExternalServiceError { message, service, status_code } => {
                format!("External service error for '{}' (status: {:?}): {}", service, status_code, message)
            }
        }
    }
    
    /// Get the error category for monitoring and alerting
    pub fn category(&self) -> &'static str {
        match self {
            AppError::DatabaseError { .. } => "database",
            AppError::AuthenticationError { .. } => "authentication",
            AppError::AuthorizationError { .. } => "authorization",
            AppError::ServerError { .. } => "server",
            AppError::FileSystemError { .. } => "filesystem",
            AppError::NetworkError { .. } => "network",
            AppError::ConfigurationError { .. } => "configuration",
            AppError::ValidationError { .. } => "validation",
            AppError::ProcessError { .. } => "process",
            AppError::WebSocketError { .. } => "websocket",
            AppError::BackupError { .. } => "backup",
            AppError::ModpackError { .. } => "modpack",
            AppError::InternalError { .. } => "internal",
            AppError::ExternalServiceError { .. } => "external",
        }
    }
    
    /// Check if this error should be retried
    pub fn is_retryable(&self) -> bool {
        match self {
            AppError::DatabaseError { .. } => true,
            AppError::NetworkError { .. } => true,
            AppError::ExternalServiceError { .. } => true,
            AppError::ProcessError { .. } => true,
            AppError::InternalError { .. } => true,
            _ => false,
        }
    }

    /// Get a safe error code for client responses (no sensitive info)
    pub fn error_code(&self) -> String {
        match self {
            AppError::DatabaseError { .. } => "DATABASE_ERROR".to_string(),
            AppError::AuthenticationError { reason, .. } => match reason {
                AuthErrorReason::InvalidCredentials => "INVALID_CREDENTIALS".to_string(),
                AuthErrorReason::TokenExpired => "TOKEN_EXPIRED".to_string(),
                AuthErrorReason::TokenInvalid => "TOKEN_INVALID".to_string(),
                AuthErrorReason::UserNotFound => "USER_NOT_FOUND".to_string(),
                AuthErrorReason::UserInactive => "USER_INACTIVE".to_string(),
                AuthErrorReason::InsufficientPermissions => "INSUFFICIENT_PERMISSIONS".to_string(),
                AuthErrorReason::SessionExpired => "SESSION_EXPIRED".to_string(),
                AuthErrorReason::RateLimitExceeded => "RATE_LIMIT_EXCEEDED".to_string(),
            },
            AppError::AuthorizationError { .. } => "AUTHORIZATION_ERROR".to_string(),
            AppError::ServerError { .. } => "SERVER_ERROR".to_string(),
            AppError::FileSystemError { .. } => "FILESYSTEM_ERROR".to_string(),
            AppError::NetworkError { .. } => "NETWORK_ERROR".to_string(),
            AppError::ConfigurationError { .. } => "CONFIGURATION_ERROR".to_string(),
            AppError::ValidationError { .. } => "VALIDATION_ERROR".to_string(),
            AppError::ProcessError { .. } => "PROCESS_ERROR".to_string(),
            AppError::WebSocketError { .. } => "WEBSOCKET_ERROR".to_string(),
            AppError::BackupError { .. } => "BACKUP_ERROR".to_string(),
            AppError::ModpackError { .. } => "MODPACK_ERROR".to_string(),
            AppError::InternalError { .. } => "INTERNAL_ERROR".to_string(),
            AppError::ExternalServiceError { .. } => "EXTERNAL_SERVICE_ERROR".to_string(),
        }
    }

    /// Get safe details for client responses (no sensitive info)
    pub fn safe_details(&self) -> Option<String> {
        match self {
            AppError::ValidationError { field, constraint, .. } => {
                Some(format!("Field '{}' failed validation: {}", field, constraint))
            }
            AppError::ServerError { server_id, operation, .. } => {
                Some(format!("Server '{}' operation '{}' failed", server_id, operation))
            }
            AppError::FileSystemError { path, operation, .. } => {
                Some(format!("File operation '{}' failed on path", operation))
            }
            AppError::NetworkError { endpoint, .. } => {
                Some(format!("Network request to '{}' failed", endpoint))
            }
            _ => None, // Don't expose internal details for other error types
        }
    }
    
    /// Get the retry delay in milliseconds
    pub fn retry_delay_ms(&self) -> u64 {
        match self {
            AppError::DatabaseError { .. } => 1000,
            AppError::NetworkError { .. } => 2000,
            AppError::ExternalServiceError { .. } => 5000,
            AppError::ProcessError { .. } => 1000,
            AppError::InternalError { .. } => 2000,
            _ => 0,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

impl StdError for AppError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        
        // Create sanitized error response for client
        let error_response = ErrorResponse {
            success: false,
            error: self.user_message(),
            error_code: self.error_code(),
            category: self.category().to_string(),
            timestamp: chrono::Utc::now(),
            details: self.safe_details(),
        };
        
        // Log the full error details server-side only
        match status {
            StatusCode::INTERNAL_SERVER_ERROR => error!("Internal server error: {}", self.detailed_message()),
            StatusCode::BAD_GATEWAY => error!("Bad gateway: {}", self.detailed_message()),
            StatusCode::UNAUTHORIZED => warn!("Authentication error: {}", self.detailed_message()),
            StatusCode::FORBIDDEN => warn!("Authorization error: {}", self.detailed_message()),
            StatusCode::TOO_MANY_REQUESTS => warn!("Rate limit exceeded: {}", self.detailed_message()),
            StatusCode::BAD_REQUEST => info!("Bad request: {}", self.detailed_message()),
            _ => info!("API error: {}", self.detailed_message()),
        }
        
        (status, Json(error_response)).into_response()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    pub error_code: String,
    pub category: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub details: Option<String>,
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, AppError>;

/// Error handler trait for components
pub trait ErrorHandler {
    fn handle_error(&self, error: AppError) -> AppError;
    fn log_error(&self, error: &AppError);
    fn should_retry(&self, error: &AppError) -> bool;
}

/// Default error handler implementation
pub struct DefaultErrorHandler;

impl ErrorHandler for DefaultErrorHandler {
    fn handle_error(&self, error: AppError) -> AppError {
        self.log_error(&error);
        error
    }
    
    fn log_error(&self, error: &AppError) {
        match error.status_code() {
            StatusCode::INTERNAL_SERVER_ERROR => error!("{}", error.detailed_message()),
            StatusCode::BAD_GATEWAY => error!("{}", error.detailed_message()),
            StatusCode::UNAUTHORIZED => warn!("{}", error.detailed_message()),
            StatusCode::FORBIDDEN => warn!("{}", error.detailed_message()),
            StatusCode::TOO_MANY_REQUESTS => warn!("{}", error.detailed_message()),
            _ => info!("{}", error.detailed_message()),
        }
    }
    
    fn should_retry(&self, error: &AppError) -> bool {
        error.is_retryable()
    }
}

/// Helper functions for creating common errors
impl AppError {
    pub fn database_error(operation: &str, message: impl Into<String>) -> Self {
        AppError::DatabaseError {
            message: message.into(),
            operation: operation.to_string(),
            table: None,
        }
    }
    
    pub fn database_error_with_table(operation: &str, table: &str, message: impl Into<String>) -> Self {
        AppError::DatabaseError {
            message: message.into(),
            operation: operation.to_string(),
            table: Some(table.to_string()),
        }
    }
    
    pub fn authentication_error(reason: AuthErrorReason, message: impl Into<String>) -> Self {
        AppError::AuthenticationError {
            message: message.into(),
            reason,
        }
    }
    
    pub fn authorization_error(required_permission: &str, user_role: &str, message: impl Into<String>) -> Self {
        AppError::AuthorizationError {
            message: message.into(),
            required_permission: required_permission.to_string(),
            user_role: user_role.to_string(),
        }
    }
    
    pub fn server_error(server_id: &str, operation: &str, message: impl Into<String>) -> Self {
        AppError::ServerError {
            message: message.into(),
            server_id: server_id.to_string(),
            operation: operation.to_string(),
        }
    }
    
    pub fn validation_error(field: &str, value: &str, constraint: &str, message: impl Into<String>) -> Self {
        AppError::ValidationError {
            message: message.into(),
            field: field.to_string(),
            value: value.to_string(),
            constraint: constraint.to_string(),
        }
    }
    
    pub fn internal_error(component: &str, message: impl Into<String>) -> Self {
        AppError::InternalError {
            message: message.into(),
            component: component.to_string(),
            details: None,
        }
    }
    
    pub fn internal_error_with_details(component: &str, message: impl Into<String>, details: impl Into<String>) -> Self {
        AppError::InternalError {
            message: message.into(),
            component: component.to_string(),
            details: Some(details.into()),
        }
    }
}

/// Macro for easy error creation
#[macro_export]
macro_rules! app_error {
    ($variant:ident, $($field:ident: $value:expr),*) => {
        AppError::$variant {
            $($field: $value,)*
        }
    };
}

/// Macro for database errors
#[macro_export]
macro_rules! db_error {
    ($operation:expr, $msg:expr) => {
        AppError::database_error($operation, $msg)
    };
    ($operation:expr, $table:expr, $msg:expr) => {
        AppError::database_error_with_table($operation, $table, $msg)
    };
}

/// Macro for validation errors
#[macro_export]
macro_rules! validation_error {
    ($field:expr, $value:expr, $constraint:expr, $msg:expr) => {
        AppError::validation_error($field, $value, $constraint, $msg)
    };
}

/// Macro for server errors
#[macro_export]
macro_rules! server_error {
    ($server_id:expr, $operation:expr, $msg:expr) => {
        AppError::server_error($server_id, $operation, $msg)
    };
}
