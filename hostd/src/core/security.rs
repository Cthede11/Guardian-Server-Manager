use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::core::{
    config::SecurityConfig,
    error_handler::{AppError, Result},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID
    pub username: String,
    pub role: String,
    pub exp: usize, // Expiration time
    pub iat: usize, // Issued at
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Admin,
    Moderator,
    User,
}

impl UserRole {
    pub fn has_permission(&self, permission: &Permission) -> bool {
        match self {
            UserRole::Admin => true,
            UserRole::Moderator => {
                matches!(permission, 
                    Permission::ViewServers | 
                    Permission::StartServer | 
                    Permission::StopServer | 
                    Permission::RestartServer |
                    Permission::ViewLogs |
                    Permission::SendCommands
                )
            },
            UserRole::User => {
                matches!(permission, 
                    Permission::ViewServers | 
                    Permission::ViewLogs
                )
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Permission {
    ViewServers,
    CreateServer,
    DeleteServer,
    StartServer,
    StopServer,
    RestartServer,
    ViewLogs,
    SendCommands,
    ManageUsers,
    ManageSettings,
    ViewMetrics,
}

pub struct SecurityManager {
    config: SecurityConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl SecurityManager {
    pub fn new(config: &SecurityConfig) -> Result<Self> {
        let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_ref());
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_ref());
        
        Ok(Self {
            config: config.clone(),
            encoding_key,
            decoding_key,
        })
    }
    
    pub fn generate_token(&self, user: &User) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| AppError::InternalError {
                message: "Failed to get current time".to_string(),
                component: "time".to_string(),
                details: Some("System time unavailable".to_string()),
            })?
            .as_secs() as usize;
        
        let exp = now + self.config.token_expiry as usize;
        
        let claims = Claims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            role: user.role.to_string(),
            exp,
            iat: now,
        };
        
        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::AuthenticationError {
                message: format!("Failed to generate token: {}", e),
                reason: crate::core::error_handler::AuthErrorReason::InvalidCredentials,
            })
    }
    
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let validation = Validation::new(Algorithm::HS256);
        
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| AppError::AuthenticationError {
                message: format!("Invalid token: {}", e),
                reason: crate::core::error_handler::AuthErrorReason::TokenInvalid,
            })?;
        
        Ok(token_data.claims)
    }
    
    pub fn hash_password(&self, password: &str) -> Result<String> {
        use argon2::{Argon2, PasswordHasher};
        use argon2::password_hash::SaltString;
        
        let salt = SaltString::generate(&mut rand::thread_rng());
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::AuthenticationError {
                message: format!("Failed to hash password: {}", e),
                reason: crate::core::error_handler::AuthErrorReason::InvalidCredentials,
            })?;
        
        Ok(password_hash.to_string())
    }
    
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        use argon2::{Argon2, PasswordVerifier};
        use argon2::password_hash::PasswordHash;
        
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::AuthenticationError {
                message: format!("Invalid password hash: {}", e),
                reason: crate::core::error_handler::AuthErrorReason::InvalidCredentials,
            })?;
        
        let argon2 = Argon2::default();
        let is_valid = argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok();
        
        Ok(is_valid)
    }
    
    pub fn check_permission(&self, user: &User, permission: &Permission) -> Result<()> {
        if !user.is_active {
            return Err(AppError::AuthorizationError {
                message: "User account is inactive".to_string(),
                required_permission: "active_account".to_string(),
                user_role: "inactive".to_string(),
            });
        }
        
        if !user.role.has_permission(permission) {
            return Err(AppError::AuthorizationError {
                message: format!("User {} does not have permission: {:?}", user.username, permission),
                required_permission: format!("{:?}", permission),
                user_role: user.role.to_string(),
            });
        }
        
        Ok(())
    }
    
    pub fn check_rate_limit(&self, user_id: &str, action: &str) -> Result<()> {
        // Simple in-memory rate limiting
        // In production, this should use Redis or similar
        // For now, we'll just return Ok()
        Ok(())
    }
    
    pub fn create_default_admin_user(&self) -> Result<User> {
        Ok(User {
            id: Uuid::new_v4(),
            username: "admin".to_string(),
            email: "admin@guardian.local".to_string(),
            role: UserRole::Admin,
            is_active: true,
            created_at: chrono::Utc::now(),
            last_login: None,
        })
    }
}

impl ToString for UserRole {
    fn to_string(&self) -> String {
        match self {
            UserRole::Admin => "admin".to_string(),
            UserRole::Moderator => "moderator".to_string(),
            UserRole::User => "user".to_string(),
        }
    }
}

impl std::str::FromStr for UserRole {
    type Err = AppError;
    
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "admin" => Ok(UserRole::Admin),
            "moderator" => Ok(UserRole::Moderator),
            "user" => Ok(UserRole::User),
            _ => Err(AppError::ValidationError {
                message: format!("Invalid user role: {}", s),
                field: "role".to_string(),
                value: s.to_string(),
                constraint: "must be one of: admin, moderator, user".to_string(),
            }),
        }
    }
}
