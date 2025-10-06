use axum::{
    extract::{Request, State},
    http::{HeaderValue, Method},
    middleware::Next,
    response::Response,
};

/// CORS configuration
#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<Method>,
    pub allowed_headers: Vec<String>,
    pub exposed_headers: Vec<String>,
    pub max_age: u32,
    pub allow_credentials: bool,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec![
                "http://localhost:3000".to_string(),
                "http://localhost:5173".to_string(),
                "http://127.0.0.1:3000".to_string(),
                "http://127.0.0.1:5173".to_string(),
            ],
            allowed_methods: vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
                Method::OPTIONS,
            ],
            allowed_headers: vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "X-Requested-With".to_string(),
                "X-User-ID".to_string(),
            ],
            exposed_headers: vec![
                "X-RateLimit-Limit".to_string(),
                "X-RateLimit-Remaining".to_string(),
                "X-RateLimit-Reset".to_string(),
            ],
            max_age: 86400, // 24 hours
            allow_credentials: true,
        }
    }
}

/// CORS middleware
pub async fn cors_middleware(
    State(config): State<CorsConfig>,
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    
    // Add CORS headers
    add_cors_headers(&mut response, &config);
    
    response
}

/// Add CORS headers to response
pub fn add_cors_headers(response: &mut Response, config: &CorsConfig) {
    // Get origin header value before accessing headers_mut
    let origin_str: Option<String> = response.headers()
        .get("origin")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    
    // Now we can safely get the mutable headers reference
    let headers = response.headers_mut();
    
    // Access-Control-Allow-Origin
    if let Some(origin_str) = origin_str {
        if config.allowed_origins.contains(&origin_str) {
            if let Ok(header_value) = HeaderValue::from_str(&origin_str) {
                headers.insert("Access-Control-Allow-Origin", header_value);
            }
        }
    } else if config.allowed_origins.contains(&"*".to_string()) {
        headers.insert(
            "Access-Control-Allow-Origin",
            HeaderValue::from_static("*"),
        );
    }
    
    // Access-Control-Allow-Methods
    let methods: Vec<String> = config.allowed_methods
        .iter()
        .map(|m| m.to_string())
        .collect();
    headers.insert(
        "Access-Control-Allow-Methods",
        HeaderValue::from_str(&methods.join(", ")).unwrap(),
    );
    
    // Access-Control-Allow-Headers
    headers.insert(
        "Access-Control-Allow-Headers",
        HeaderValue::from_str(&config.allowed_headers.join(", ")).unwrap(),
    );
    
    // Access-Control-Expose-Headers
    if !config.exposed_headers.is_empty() {
        headers.insert(
            "Access-Control-Expose-Headers",
            HeaderValue::from_str(&config.exposed_headers.join(", ")).unwrap(),
        );
    }
    
    // Access-Control-Max-Age
    headers.insert(
        "Access-Control-Max-Age",
        HeaderValue::from_str(&config.max_age.to_string()).unwrap(),
    );
    
    // Access-Control-Allow-Credentials
    if config.allow_credentials {
        headers.insert(
            "Access-Control-Allow-Credentials",
            HeaderValue::from_static("true"),
        );
    }
}

/// Handle preflight OPTIONS requests
pub async fn handle_preflight(
    State(config): State<CorsConfig>,
    request: Request,
) -> Response {
    let mut response = Response::new(axum::body::Body::empty());
    response.headers_mut().insert(
        "Access-Control-Allow-Origin",
        HeaderValue::from_static("*"),
    );
    
    add_cors_headers(&mut response, &config);
    
    response
}

/// Production CORS configuration
pub fn production_cors_config() -> CorsConfig {
    CorsConfig {
        allowed_origins: vec![
            "https://guardian.example.com".to_string(),
            "https://app.guardian.example.com".to_string(),
        ],
        allowed_methods: vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ],
        allowed_headers: vec![
            "Content-Type".to_string(),
            "Authorization".to_string(),
            "X-Requested-With".to_string(),
            "X-User-ID".to_string(),
            "X-API-Key".to_string(),
        ],
        exposed_headers: vec![
            "X-RateLimit-Limit".to_string(),
            "X-RateLimit-Remaining".to_string(),
            "X-RateLimit-Reset".to_string(),
            "X-Request-ID".to_string(),
        ],
        max_age: 86400,
        allow_credentials: true,
    }
}

/// Development CORS configuration
pub fn development_cors_config() -> CorsConfig {
    CorsConfig {
        allowed_origins: vec![
            "http://localhost:3000".to_string(),
            "http://localhost:5173".to_string(),
            "http://127.0.0.1:3000".to_string(),
            "http://127.0.0.1:5173".to_string(),
            "http://localhost:8080".to_string(),
            "http://127.0.0.1:8080".to_string(),
        ],
        allowed_methods: vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ],
        allowed_headers: vec![
            "Content-Type".to_string(),
            "Authorization".to_string(),
            "X-Requested-With".to_string(),
            "X-User-ID".to_string(),
            "X-API-Key".to_string(),
            "X-Debug-Token".to_string(),
        ],
        exposed_headers: vec![
            "X-RateLimit-Limit".to_string(),
            "X-RateLimit-Remaining".to_string(),
            "X-RateLimit-Reset".to_string(),
            "X-Request-ID".to_string(),
            "X-Debug-Info".to_string(),
        ],
        max_age: 86400,
        allow_credentials: true,
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
    fn test_cors_config_default() {
        let config = CorsConfig::default();
        assert!(!config.allowed_origins.is_empty());
        assert!(!config.allowed_methods.is_empty());
        assert!(!config.allowed_headers.is_empty());
    }

    #[test]
    fn test_production_cors_config() {
        let config = production_cors_config();
        assert!(config.allowed_origins.iter().all(|origin| origin.starts_with("https://")));
        assert!(config.allow_credentials);
    }

    #[test]
    fn test_development_cors_config() {
        let config = development_cors_config();
        assert!(config.allowed_origins.iter().any(|origin| origin.contains("localhost")));
        assert!(config.allowed_headers.contains(&"X-Debug-Token".to_string()));
    }
}
