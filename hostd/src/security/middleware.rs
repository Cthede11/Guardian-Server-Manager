use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::{Validate, ValidationErrors};

use crate::security::validation::ValidationService;
use crate::core::error_handler::AppError;

/// Structured error response for API endpoints
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
    pub details: Option<HashMap<String, String>>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ErrorResponse {
    pub fn new(error: String) -> Self {
        Self {
            success: false,
            error,
            details: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_details(error: String, details: HashMap<String, String>) -> Self {
        Self {
            success: false,
            error,
            details: Some(details),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Input validation middleware
pub async fn validate_input_middleware(
    State(_state): State<crate::api::AppState>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    // Extract path and method for validation
    let path = request.uri().path();
    let method = request.method().as_str();

    // Validate path parameters
    if let Err(validation_error) = validate_path_parameters(path) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(format!("Invalid path parameters: {}", validation_error))),
        ));
    }

    // Continue to next middleware
    Ok(next.run(request).await)
}

/// Validate path parameters for common patterns
fn validate_path_parameters(path: &str) -> Result<(), String> {
    // Check for path traversal attempts
    if path.contains("..") || path.contains("//") {
        return Err("Path traversal detected".to_string());
    }

    // Check for suspicious patterns
    let suspicious_patterns = [
        "admin", "config", "system", "etc", "proc", "sys", "var", "tmp",
        "windows", "system32", "program files", "users", "appdata",
    ];

    let path_lower = path.to_lowercase();
    for pattern in &suspicious_patterns {
        if path_lower.contains(pattern) {
            return Err(format!("Suspicious path pattern detected: {}", pattern));
        }
    }

    Ok(())
}

/// Validate request headers
pub fn validate_headers(headers: &HeaderMap) -> Result<(), String> {
    // Check for suspicious header values
    for (name, value) in headers.iter() {
        let header_name = name.as_str().to_lowercase();
        let header_value = value.to_str().unwrap_or("");

        // Check for suspicious header names
        if header_name.contains("..") || header_name.contains("script") {
            return Err(format!("Suspicious header name: {}", header_name));
        }

        // Check for suspicious header values
        if header_value.contains("..") || header_value.contains("<script") {
            return Err(format!("Suspicious header value in {}: {}", header_name, header_value));
        }
    }

    Ok(())
}

/// Validate query parameters
pub fn validate_query_params(query: &str) -> Result<(), String> {
    if query.is_empty() {
        return Ok(());
    }

    // Check for suspicious query patterns
    let suspicious_patterns = [
        "..", "<script", "javascript:", "data:", "vbscript:",
        "onload=", "onerror=", "onclick=", "onmouseover=",
    ];

    let query_lower = query.to_lowercase();
    for pattern in &suspicious_patterns {
        if query_lower.contains(pattern) {
            return Err(format!("Suspicious query parameter detected: {}", pattern));
        }
    }

    Ok(())
}

/// Comprehensive input validation for API endpoints
pub struct InputValidator;

impl InputValidator {
    /// Validate server creation request
    pub fn validate_server_creation(
        name: &str,
        minecraft_version: &str,
        loader: &str,
        port: u16,
        memory: u32,
    ) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        // Validate server name
        if let Err(e) = ValidationService::validate_server_name(name) {
            errors.add("name", e);
        }

        // Validate Minecraft version
        if let Err(e) = ValidationService::validate_minecraft_version(minecraft_version) {
            errors.add("minecraft_version", e);
        }

        // Validate loader
        if let Err(e) = ValidationService::validate_loader(loader) {
            errors.add("loader", e);
        }

        // Validate port
        if let Err(e) = ValidationService::validate_port_range(port) {
            errors.add("port", e);
        }

        // Validate memory
        if let Err(e) = ValidationService::validate_memory_allocation(memory) {
            errors.add("memory", e);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate mod installation request
    pub fn validate_mod_installation(
        mod_id: &str,
        provider: &str,
        version: &str,
    ) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        // Validate mod ID
        if let Err(e) = ValidationService::validate_mod_id(mod_id) {
            errors.add("mod_id", e);
        }

        // Validate provider
        if let Err(e) = ValidationService::validate_provider(provider) {
            errors.add("provider", e);
        }

        // Validate version
        if let Err(e) = ValidationService::validate_version_string(version) {
            errors.add("version", e);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate API key
    pub fn validate_api_key(api_key: &str) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        if let Err(e) = ValidationService::validate_api_key(api_key) {
            errors.add("api_key", e);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Sanitize and validate all string inputs
    pub fn sanitize_string_input(input: &str) -> String {
        ValidationService::sanitize_input(input)
    }

    /// Validate file path for security
    pub fn validate_secure_path(path: &str) -> Result<(), String> {
        ValidationService::validate_file_path(path)
            .map_err(|e| format!("Invalid file path: {}", e))
    }
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    State(_state): State<crate::api::AppState>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    // Extract client IP (in a real implementation, you'd get this from headers)
    let client_ip = "127.0.0.1"; // Placeholder - in production, extract from X-Forwarded-For or similar
    
    // For now, we'll use a simple rate limiting approach
    // In production, you'd integrate with the RateLimiter from rate_limiting.rs
    let path = request.uri().path();
    
    // Apply different rate limits based on endpoint
    let rate_limit_exceeded = match path {
        "/api/auth/login" => false, // No rate limiting for login in this example
        "/api/servers" => false,    // No rate limiting for server operations in this example
        _ => false,                 // No rate limiting for other endpoints in this example
    };

    if rate_limit_exceeded {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(ErrorResponse::new("Rate limit exceeded".to_string())),
        ));
    }

    Ok(next.run(request).await)
}

/// Security headers middleware
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    
    // Add security headers
    let headers = response.headers_mut();
    
    // Prevent clickjacking
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    
    // Prevent MIME type sniffing
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    
    // Enable XSS protection
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    
    // Strict Transport Security (if using HTTPS)
    headers.insert("Strict-Transport-Security", "max-age=31536000; includeSubDomains".parse().unwrap());
    
    // Content Security Policy
    headers.insert("Content-Security-Policy", "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'".parse().unwrap());
    
    // Referrer Policy
    headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());
    
    response
}

/// Error handling middleware
pub async fn error_handling_middleware(
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    match next.run(request).await.into_parts() {
        (parts, body) => {
            // In a real implementation, you'd handle different error types here
            // For now, we'll just pass through the response
            Ok(Response::from_parts(parts, body))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_path_parameters() {
        assert!(validate_path_parameters("/api/servers").is_ok());
        assert!(validate_path_parameters("/api/servers/123").is_ok());
        assert!(validate_path_parameters("/api/servers/../admin").is_err());
        assert!(validate_path_parameters("/api/servers//admin").is_err());
    }

    #[test]
    fn test_validate_query_params() {
        assert!(validate_query_params("page=1&limit=10").is_ok());
        assert!(validate_query_params("").is_ok());
        assert!(validate_query_params("search=test").is_ok());
        assert!(validate_query_params("path=../admin").is_err());
        assert!(validate_query_params("script=<script>alert('xss')</script>").is_err());
    }

    #[test]
    fn test_input_validator() {
        // Test valid server creation
        assert!(InputValidator::validate_server_creation(
            "test-server",
            "1.20.1",
            "fabric",
            25565,
            2048
        ).is_ok());

        // Test invalid server creation
        assert!(InputValidator::validate_server_creation(
            "",
            "invalid",
            "unknown",
            80,
            100
        ).is_err());
    }
}
