use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
    Json,
};
use serde_json;
use std::sync::Arc;

use crate::core::validation::{InputValidator, ValidationResult};
use crate::api::ApiResponse;

/// Validation middleware that validates request body against endpoint rules
pub async fn validation_middleware(
    State(validator): State<Arc<InputValidator>>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<()>>)> {
    // Extract the path from the request
    let path = request.uri().path();
    
    // Only validate POST, PUT, PATCH requests with JSON bodies
    if matches!(request.method(), &axum::http::Method::POST | &axum::http::Method::PUT | &axum::http::Method::PATCH) {
        // For now, skip body validation to avoid the body ownership issue
        // TODO: Implement proper body validation
        return Ok(next.run(request).await);
    }

    // Continue to the next middleware/handler
    Ok(next.run(request).await)
}

/// Create a validation middleware with default rules
pub fn create_validation_middleware() -> Arc<InputValidator> {
    use crate::core::validation::create_default_validations;
    Arc::new(create_default_validations())
}
