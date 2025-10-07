use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{Response, IntoResponse},
    Json,
};
use serde_json::json;
use std::sync::Arc;
use crate::core::auth::{AuthManager, Permission};

#[derive(Clone)]
pub struct AuthContext {
    pub user_id: uuid::Uuid,
    pub username: String,
    pub role: crate::core::auth::UserRole,
}

pub async fn auth_middleware(
    State(auth_manager): State<Arc<AuthManager>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract token from Authorization header
    let token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "));
    
    let token = match token {
        Some(token) => token,
        None => {
            return Ok(Json(json!({
                "success": false,
                "error": "Missing authorization token",
                "timestamp": chrono::Utc::now()
            })).into_response());
        }
    };
    
    // Validate token and get user
    let user = match auth_manager.validate_token(token).await {
        Ok(user) => user,
        Err(_) => {
            return Ok(Json(json!({
                "success": false,
                "error": "Invalid or expired token",
                "timestamp": chrono::Utc::now()
            })).into_response());
        }
    };
    
    // Add user context to request extensions
    let auth_context = AuthContext {
        user_id: user.id,
        username: user.username,
        role: user.role,
    };
    
    request.extensions_mut().insert(auth_context);
    
    Ok(next.run(request).await)
}

pub async fn permission_middleware(
    State(auth_manager): State<Arc<AuthManager>>,
    State(required_permission): State<Permission>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract token from Authorization header
    let token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "));
    
    let token = match token {
        Some(token) => token,
        None => {
            return Ok(Json(json!({
                "success": false,
                "error": "Missing authorization token",
                "timestamp": chrono::Utc::now()
            })).into_response());
        }
    };
    
    // Validate token and get user
    let user = match auth_manager.validate_token(token).await {
        Ok(user) => user,
        Err(_) => {
            return Ok(Json(json!({
                "success": false,
                "error": "Invalid or expired token",
                "timestamp": chrono::Utc::now()
            })).into_response());
        }
    };
    
    // Check permission
    if !auth_manager.has_permission(user.id, &required_permission).await {
        return Ok(Json(json!({
            "success": false,
            "error": "Insufficient permissions",
            "timestamp": chrono::Utc::now()
        })).into_response());
    }
    
    // Add user context to request extensions
    let auth_context = AuthContext {
        user_id: user.id,
        username: user.username,
        role: user.role,
    };
    
    request.extensions_mut().insert(auth_context);
    
    Ok(next.run(request).await)
}

// This function is not used in the current implementation
// pub fn require_permission(permission: Permission) -> axum::middleware::FromFnLayer<impl Fn(State<Arc<AuthManager>>, State<Permission>, HeaderMap, Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send + 'static>> + Clone + Send + Sync + 'static, (State<Arc<AuthManager>>, State<Permission>)> {
//     axum::middleware::from_fn_with_state(
//         (Arc::new(AuthManager::new("default_secret".to_string())), permission),
//         permission_middleware,
//     )
// }

// Helper function to extract auth context from request
pub fn get_auth_context(request: &Request) -> Option<&AuthContext> {
    request.extensions().get::<AuthContext>()
}

// Helper function to check if user has permission
pub async fn check_permission(
    auth_manager: &AuthManager,
    user_id: uuid::Uuid,
    permission: &Permission,
) -> bool {
    auth_manager.has_permission(user_id, permission).await
}
