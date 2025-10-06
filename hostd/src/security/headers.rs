use axum::{
    extract::{Request, State},
    http::HeaderValue,
    middleware::Next,
    response::Response,
};
use std::time::{SystemTime, UNIX_EPOCH};

/// Security headers configuration
#[derive(Debug, Clone)]
pub struct SecurityHeadersConfig {
    pub enable_hsts: bool,
    pub enable_csp: bool,
    pub enable_xss_protection: bool,
    pub enable_frame_options: bool,
    pub enable_content_type_nosniff: bool,
    pub enable_referrer_policy: bool,
    pub enable_permissions_policy: bool,
}

impl Default for SecurityHeadersConfig {
    fn default() -> Self {
        Self {
            enable_hsts: true,
            enable_csp: true,
            enable_xss_protection: true,
            enable_frame_options: true,
            enable_content_type_nosniff: true,
            enable_referrer_policy: true,
            enable_permissions_policy: true,
        }
    }
}

/// Security headers middleware
pub async fn security_headers_middleware(
    State(config): State<SecurityHeadersConfig>,
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    
    // Add security headers
    add_security_headers(&mut response, &config);
    
    response
}

/// Add security headers to response
pub fn add_security_headers(response: &mut Response, config: &SecurityHeadersConfig) {
    let headers = response.headers_mut();
    
    // HTTP Strict Transport Security (HSTS)
    if config.enable_hsts {
        headers.insert(
            "Strict-Transport-Security",
            HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
        );
    }
    
    // Content Security Policy (CSP)
    if config.enable_csp {
        let csp = "default-src 'self'; \
                   script-src 'self' 'unsafe-inline' 'unsafe-eval'; \
                   style-src 'self' 'unsafe-inline'; \
                   img-src 'self' data: https:; \
                   font-src 'self' data:; \
                   connect-src 'self' ws: wss:; \
                   frame-ancestors 'none'; \
                   base-uri 'self'; \
                   form-action 'self'";
        headers.insert(
            "Content-Security-Policy",
            HeaderValue::from_str(csp).unwrap(),
        );
    }
    
    // X-XSS-Protection
    if config.enable_xss_protection {
        headers.insert(
            "X-XSS-Protection",
            HeaderValue::from_static("1; mode=block"),
        );
    }
    
    // X-Frame-Options
    if config.enable_frame_options {
        headers.insert(
            "X-Frame-Options",
            HeaderValue::from_static("DENY"),
        );
    }
    
    // X-Content-Type-Options
    if config.enable_content_type_nosniff {
        headers.insert(
            "X-Content-Type-Options",
            HeaderValue::from_static("nosniff"),
        );
    }
    
    // Referrer-Policy
    if config.enable_referrer_policy {
        headers.insert(
            "Referrer-Policy",
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        );
    }
    
    // Permissions-Policy
    if config.enable_permissions_policy {
        let permissions = "camera=(), microphone=(), geolocation=(), \
                          payment=(), usb=(), magnetometer=(), \
                          gyroscope=(), accelerometer=(), ambient-light-sensor=()";
        headers.insert(
            "Permissions-Policy",
            HeaderValue::from_str(permissions).unwrap(),
        );
    }
    
    // Additional security headers
    headers.insert(
        "X-Permitted-Cross-Domain-Policies",
        HeaderValue::from_static("none"),
    );
    
    headers.insert(
        "Cross-Origin-Embedder-Policy",
        HeaderValue::from_static("require-corp"),
    );
    
    headers.insert(
        "Cross-Origin-Opener-Policy",
        HeaderValue::from_static("same-origin"),
    );
    
    headers.insert(
        "Cross-Origin-Resource-Policy",
        HeaderValue::from_static("same-origin"),
    );
}

/// Add request ID header
pub fn add_request_id_header(response: &mut Response) {
    let request_id = generate_request_id();
    response.headers_mut().insert(
        "X-Request-ID",
        HeaderValue::from_str(&request_id).unwrap(),
    );
}

/// Generate unique request ID
fn generate_request_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    
    let random = rand::random::<u32>();
    format!("req_{}_{}", timestamp, random)
}

/// Add cache control headers
pub fn add_cache_control_headers(response: &mut Response, cache_duration: u32) {
    let cache_control = format!("public, max-age={}", cache_duration);
    response.headers_mut().insert(
        "Cache-Control",
        HeaderValue::from_str(&cache_control).unwrap(),
    );
}

/// Add no-cache headers
pub fn add_no_cache_headers(response: &mut Response) {
    response.headers_mut().insert(
        "Cache-Control",
        HeaderValue::from_static("no-cache, no-store, must-revalidate"),
    );
    
    response.headers_mut().insert(
        "Pragma",
        HeaderValue::from_static("no-cache"),
    );
    
    response.headers_mut().insert(
        "Expires",
        HeaderValue::from_static("0"),
    );
}

/// Add CORS headers for specific origins
pub fn add_cors_headers_for_origin(response: &mut Response, origin: &str) {
    let allowed_origins = vec![
        "http://localhost:3000",
        "http://localhost:5173",
        "http://127.0.0.1:3000",
        "http://127.0.0.1:5173",
    ];
    
    if allowed_origins.contains(&origin) {
        response.headers_mut().insert(
            "Access-Control-Allow-Origin",
            HeaderValue::from_str(origin).unwrap(),
        );
        
        response.headers_mut().insert(
            "Access-Control-Allow-Credentials",
            HeaderValue::from_static("true"),
        );
    }
}

/// Add API version header
pub fn add_api_version_header(response: &mut Response, version: &str) {
    response.headers_mut().insert(
        "X-API-Version",
        HeaderValue::from_str(version).unwrap(),
    );
}

/// Add server information header
pub fn add_server_header(response: &mut Response, server_name: &str) {
    response.headers_mut().insert(
        "Server",
        HeaderValue::from_str(server_name).unwrap(),
    );
}

/// Production security headers configuration
pub fn production_security_config() -> SecurityHeadersConfig {
    SecurityHeadersConfig {
        enable_hsts: true,
        enable_csp: true,
        enable_xss_protection: true,
        enable_frame_options: true,
        enable_content_type_nosniff: true,
        enable_referrer_policy: true,
        enable_permissions_policy: true,
    }
}

/// Development security headers configuration
pub fn development_security_config() -> SecurityHeadersConfig {
    SecurityHeadersConfig {
        enable_hsts: false, // Disable HSTS in development
        enable_csp: true,
        enable_xss_protection: true,
        enable_frame_options: true,
        enable_content_type_nosniff: true,
        enable_referrer_policy: true,
        enable_permissions_policy: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        response::Response,
    };

    #[test]
    fn test_security_headers_config_default() {
        let config = SecurityHeadersConfig::default();
        assert!(config.enable_hsts);
        assert!(config.enable_csp);
        assert!(config.enable_xss_protection);
    }

    #[test]
    fn test_production_security_config() {
        let config = production_security_config();
        assert!(config.enable_hsts);
        assert!(config.enable_csp);
    }

    #[test]
    fn test_development_security_config() {
        let config = development_security_config();
        assert!(!config.enable_hsts); // HSTS disabled in development
        assert!(config.enable_csp);
    }

    #[test]
    fn test_request_id_generation() {
        let id1 = generate_request_id();
        let id2 = generate_request_id();
        
        assert!(!id1.is_empty());
        assert!(!id2.is_empty());
        assert_ne!(id1, id2);
        assert!(id1.starts_with("req_"));
        assert!(id2.starts_with("req_"));
    }
}
