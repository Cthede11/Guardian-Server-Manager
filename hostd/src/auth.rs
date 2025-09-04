use crate::config::Config;
use anyhow::Result;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use chrono::{Duration, Utc};

/// JWT Claims for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // User ID
    pub exp: i64,          // Expiration time
    pub iat: i64,          // Issued at
    pub role: UserRole,    // User role
    pub tenant_id: Option<String>, // Tenant ID for multi-tenancy
    pub permissions: Vec<String>,  // Specific permissions
}

/// User roles with hierarchical permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    SuperAdmin,    // Full system access
    Admin,         // Tenant admin
    Operator,      // Server operations
    Viewer,        // Read-only access
    Plugin,        // Plugin-specific access
}

impl UserRole {
    pub fn has_permission(&self, permission: &str) -> bool {
        match self {
            UserRole::SuperAdmin => true,
            UserRole::Admin => matches!(permission, 
                "server:read" | "server:write" | "server:restart" | 
                "snapshot:read" | "snapshot:write" | "snapshot:delete" |
                "config:read" | "config:write" | "metrics:read" |
                "plugin:read" | "plugin:write" | "webhook:read" | "webhook:write"
            ),
            UserRole::Operator => matches!(permission,
                "server:read" | "server:restart" | 
                "snapshot:read" | "snapshot:write" |
                "config:read" | "metrics:read"
            ),
            UserRole::Viewer => matches!(permission,
                "server:read" | "snapshot:read" | "config:read" | "metrics:read"
            ),
            UserRole::Plugin => matches!(permission,
                "plugin:read" | "plugin:write" | "webhook:read" | "webhook:write"
            ),
        }
    }
}

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub tenant_id: Option<String>,
    pub permissions: Vec<String>,
    pub created_at: chrono::DateTime<Utc>,
    pub last_login: Option<chrono::DateTime<Utc>>,
    pub is_active: bool,
}

/// Authentication manager
pub struct AuthManager {
    config: Config,
    users: Arc<RwLock<Vec<User>>>,
    jwt_secret: String,
    rate_limiter: Arc<RwLock<RateLimiter>>,
}

/// Rate limiter for API endpoints
#[derive(Debug)]
pub struct RateLimiter {
    requests: std::collections::HashMap<String, Vec<chrono::DateTime<Utc>>>,
    limits: RateLimits,
}

#[derive(Debug, Clone)]
pub struct RateLimits {
    pub login_attempts: u32,
    pub api_requests: u32,
    pub window_minutes: i64,
}

impl AuthManager {
    pub fn new(config: Config) -> Self {
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "guardian-super-secret-key-change-in-production".to_string());
        
        Self {
            config,
            users: Arc::new(RwLock::new(Vec::new())),
            jwt_secret,
            rate_limiter: Arc::new(RwLock::new(RateLimiter::new())),
        }
    }

    /// Initialize default users and permissions
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing authentication system...");
        
        // Create default super admin user
        let super_admin = User {
            id: Uuid::new_v4().to_string(),
            username: "admin".to_string(),
            email: "admin@guardian.local".to_string(),
            role: UserRole::SuperAdmin,
            tenant_id: None,
            permissions: vec!["*".to_string()],
            created_at: Utc::now(),
            last_login: None,
            is_active: true,
        };

        let mut users = self.users.write().await;
        users.push(super_admin);
        
        info!("Authentication system initialized with default admin user");
        Ok(())
    }

    /// Authenticate user with username/password
    pub async fn authenticate(&self, username: &str, password: &str, ip: &str) -> Result<String> {
        // Check rate limiting
        if !self.rate_limiter.write().await.check_rate_limit(ip, "login") {
            return Err(anyhow::anyhow!("Too many login attempts"));
        }

        let (user_id, user_role, user_tenant_id, user_permissions) = {
            let users = self.users.read().await;
            let user = users.iter()
                .find(|u| u.username == username && u.is_active)
                .ok_or_else(|| anyhow::anyhow!("Invalid credentials"))?;
            
            // In production, use proper password hashing (bcrypt, argon2, etc.)
            // For now, using a simple check - CHANGE IN PRODUCTION
            if password != "admin123" {
                return Err(anyhow::anyhow!("Invalid credentials"));
            }
            
            (user.id.clone(), user.role.clone(), user.tenant_id.clone(), user.permissions.clone())
        };

        // Update last login
        let mut users = self.users.write().await;
        if let Some(user) = users.iter_mut().find(|u| u.id == user_id) {
            user.last_login = Some(Utc::now());
        }

        // Generate JWT token
        let claims = Claims {
            sub: user_id.clone(),
            exp: (Utc::now() + Duration::hours(24)).timestamp(),
            iat: Utc::now().timestamp(),
            role: user_role,
            tenant_id: user_tenant_id,
            permissions: user_permissions,
        };

        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;

        info!("User {} authenticated successfully", username);
        Ok(token)
    }

    /// Validate JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &validation,
        )?;

        Ok(token_data.claims)
    }

    /// Check if user has permission
    pub fn has_permission(claims: &Claims, permission: &str) -> bool {
        claims.role.has_permission(permission) || 
        claims.permissions.contains(&"*".to_string()) ||
        claims.permissions.contains(&permission.to_string())
    }

    /// Create new user
    pub async fn create_user(&self, user: User) -> Result<()> {
        let mut users = self.users.write().await;
        users.push(user);
        info!("New user created");
        Ok(())
    }

    /// Get user by ID
    pub async fn get_user(&self, user_id: &str) -> Option<User> {
        let users = self.users.read().await;
        users.iter().find(|u| u.id == user_id).cloned()
    }

    /// List all users
    pub async fn list_users(&self) -> Vec<User> {
        let users = self.users.read().await;
        users.clone()
    }
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            requests: std::collections::HashMap::new(),
            limits: RateLimits {
                login_attempts: 5,
                api_requests: 1000,
                window_minutes: 15,
            },
        }
    }

    pub fn check_rate_limit(&mut self, identifier: &str, endpoint: &str) -> bool {
        let now = Utc::now();
        let key = format!("{}:{}", identifier, endpoint);
        
        let limit = match endpoint {
            "login" => self.limits.login_attempts,
            _ => self.limits.api_requests,
        };

        let requests = self.requests.entry(key).or_insert_with(Vec::new);
        
        // Remove old requests outside the window
        requests.retain(|&time| now.signed_duration_since(time).num_minutes() < self.limits.window_minutes);
        
        if requests.len() >= limit as usize {
            return false;
        }
        
        requests.push(now);
        true
    }
}

/// Authentication middleware for Axum
pub async fn auth_middleware(
    State(auth_manager): State<Arc<AuthManager>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = headers.get("Authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let claims = match auth_manager.validate_token(token) {
        Ok(claims) => claims,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // Check if token is expired
    if claims.exp < Utc::now().timestamp() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Add claims to request extensions for use in handlers
    let mut request = request;
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Permission middleware
pub async fn require_permission(
    permission: &str,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims = request.extensions()
        .get::<Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !AuthManager::has_permission(claims, permission) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}

/// Multi-tenancy middleware
pub async fn require_tenant_access(
    tenant_id: &str,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims = request.extensions()
        .get::<Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Super admin can access all tenants
    if claims.role == UserRole::SuperAdmin {
        return Ok(next.run(request).await);
    }

    // Check if user has access to this tenant
    if claims.tenant_id.as_ref() != Some(&tenant_id.to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}
