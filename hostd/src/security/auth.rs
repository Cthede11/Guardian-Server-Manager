use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // Subject (user ID)
    pub exp: usize,         // Expiration time
    pub iat: usize,         // Issued at
    pub role: String,       // User role
    pub permissions: Vec<String>, // User permissions
}

/// User authentication data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
}

/// Authentication service
pub struct AuthService {
    jwt_secret: String,
    jwt_expiry: u64, // in seconds
}

impl AuthService {
    pub fn new(jwt_secret: String) -> Self {
        Self {
            jwt_secret,
            jwt_expiry: 3600, // 1 hour default
        }
    }

    /// Generate JWT token for user
    pub fn generate_token(&self, user: &User) -> Result<String, jsonwebtoken::errors::Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        let claims = Claims {
            sub: user.id.clone(),
            exp: now + self.jwt_expiry as usize,
            iat: now,
            role: user.role.clone(),
            permissions: user.permissions.clone(),
        };

        let header = Header::new(Algorithm::HS256);
        encode(&header, &claims, &EncodingKey::from_secret(self.jwt_secret.as_ref()))
    }

    /// Validate JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &validation,
        )?;

        Ok(token_data.claims)
    }

    /// Authenticate user with username/password
    pub async fn authenticate_user(
        &self,
        username: &str,
        password: &str,
    ) -> Result<User, AuthError> {
        // In production, this would query the database
        // For now, we'll use a simple hardcoded admin user
        if username == "admin" && password == "admin123" {
            Ok(User {
                id: Uuid::new_v4().to_string(),
                username: username.to_string(),
                email: "admin@guardian.local".to_string(),
                role: "admin".to_string(),
                permissions: vec![
                    "servers:read".to_string(),
                    "servers:write".to_string(),
                    "servers:delete".to_string(),
                    "users:read".to_string(),
                    "users:write".to_string(),
                    "settings:read".to_string(),
                    "settings:write".to_string(),
                ],
                is_active: true,
                created_at: chrono::Utc::now(),
                last_login: Some(chrono::Utc::now()),
            })
        } else {
            Err(AuthError::InvalidCredentials)
        }
    }
}

/// Authentication errors
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid token")]
    InvalidToken,
    #[error("User not found")]
    UserNotFound,
    #[error("User inactive")]
    UserInactive,
    #[error("Insufficient permissions")]
    InsufficientPermissions,
}

/// Authentication middleware
pub async fn auth_middleware(
    State(auth_service): State<AuthService>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract token from Authorization header
    let token = headers
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "));

    let token = match token {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // Validate token
    let claims = match auth_service.validate_token(token) {
        Ok(claims) => claims,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // Add user info to request extensions
    // This would be used by downstream handlers
    // request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Permission-based authorization
pub fn require_permission(required_permission: &str) -> impl Fn(&Claims) -> bool {
    let permission = required_permission.to_string();
    move |claims: &Claims| claims.permissions.contains(&permission)
}

/// Role-based authorization
pub fn require_role(required_role: &str) -> impl Fn(&Claims) -> bool {
    let role = required_role.to_string();
    move |claims: &Claims| claims.role == role
}

/// Login request structure
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response structure
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
    pub expires_in: u64,
}

/// Login endpoint handler
pub async fn login_handler(
    State(auth_service): State<AuthService>,
    Json(login_request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let user = auth_service
        .authenticate_user(&login_request.username, &login_request.password)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let token = auth_service
        .generate_token(&user)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(LoginResponse {
        token,
        user,
        expires_in: auth_service.jwt_expiry,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_authentication() {
        let auth_service = AuthService::new("test_secret".to_string());
        
        // Test valid credentials
        let user = auth_service
            .authenticate_user("admin", "admin123")
            .await
            .unwrap();
        
        assert_eq!(user.username, "admin");
        assert_eq!(user.role, "admin");
        
        // Test invalid credentials
        let result = auth_service
            .authenticate_user("admin", "wrong_password")
            .await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_token_generation() {
        let auth_service = AuthService::new("test_secret".to_string());
        let user = User {
            id: "test_id".to_string(),
            username: "test_user".to_string(),
            email: "test@example.com".to_string(),
            role: "user".to_string(),
            permissions: vec!["read".to_string()],
            is_active: true,
            created_at: chrono::Utc::now(),
            last_login: None,
        };

        let token = auth_service.generate_token(&user).unwrap();
        let claims = auth_service.validate_token(&token).unwrap();
        
        assert_eq!(claims.sub, user.id);
        assert_eq!(claims.role, user.role);
    }
}