use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use anyhow::{Result, anyhow};
use crate::core::credential_manager::CredentialManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Admin,
    Moderator,
    User,
    ReadOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub permissions: Vec<Permission>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    // Server management
    CreateServer,
    DeleteServer,
    StartServer,
    StopServer,
    RestartServer,
    ViewServer,
    EditServer,
    
    // User management
    CreateUser,
    DeleteUser,
    EditUser,
    ViewUser,
    
    // Role management
    CreateRole,
    DeleteRole,
    EditRole,
    ViewRole,
    
    // Backup management
    CreateBackup,
    DeleteBackup,
    RestoreBackup,
    ViewBackup,
    
    // System management
    ViewLogs,
    ViewMetrics,
    SystemSettings,
    
    // Modpack management
    CreateModpack,
    DeleteModpack,
    EditModpack,
    ViewModpack,
    InstallMod,
    UninstallMod,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: Option<UserRole>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String, // user id
    pub username: String,
    pub role: UserRole,
    pub exp: usize,
    pub iat: usize,
}

pub struct AuthManager {
    users: Arc<RwLock<HashMap<Uuid, User>>>,
    roles: Arc<RwLock<HashMap<Uuid, Role>>>,
    user_sessions: Arc<RwLock<HashMap<String, Uuid>>>, // token -> user_id
    jwt_secret: String,
    jwt_expiry: Duration,
    credential_manager: Arc<CredentialManager>,
}

impl AuthManager {
    pub fn new(jwt_secret: String, credential_manager: Arc<CredentialManager>) -> Self {
        Self {
            users: Arc::new(RwLock::new(HashMap::new())),
            roles: Arc::new(RwLock::new(HashMap::new())),
            user_sessions: Arc::new(RwLock::new(HashMap::new())),
            jwt_secret,
            jwt_expiry: Duration::from_secs(24 * 60 * 60), // 24 hours
            credential_manager,
        }
    }
    
    pub async fn initialize(&self) -> Result<()> {
        // Create default roles
        self.create_default_roles().await?;
        
        // Create default admin user
        self.create_default_admin().await?;
        
        Ok(())
    }
    
    async fn create_default_roles(&self) -> Result<()> {
        let mut roles = self.roles.write().await;
        
        // Admin role - all permissions
        let admin_role = Role {
            id: Uuid::new_v4(),
            name: "Admin".to_string(),
            permissions: vec![
                Permission::CreateServer,
                Permission::DeleteServer,
                Permission::StartServer,
                Permission::StopServer,
                Permission::RestartServer,
                Permission::ViewServer,
                Permission::EditServer,
                Permission::CreateUser,
                Permission::DeleteUser,
                Permission::EditUser,
                Permission::ViewUser,
                Permission::CreateRole,
                Permission::DeleteRole,
                Permission::EditRole,
                Permission::ViewRole,
                Permission::CreateBackup,
                Permission::DeleteBackup,
                Permission::RestoreBackup,
                Permission::ViewBackup,
                Permission::ViewLogs,
                Permission::ViewMetrics,
                Permission::SystemSettings,
                Permission::CreateModpack,
                Permission::DeleteModpack,
                Permission::EditModpack,
                Permission::ViewModpack,
                Permission::InstallMod,
                Permission::UninstallMod,
            ],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        roles.insert(admin_role.id, admin_role);
        
        // Moderator role - server management and user viewing
        let moderator_role = Role {
            id: Uuid::new_v4(),
            name: "Moderator".to_string(),
            permissions: vec![
                Permission::StartServer,
                Permission::StopServer,
                Permission::RestartServer,
                Permission::ViewServer,
                Permission::EditServer,
                Permission::ViewUser,
                Permission::ViewBackup,
                Permission::ViewLogs,
                Permission::ViewMetrics,
                Permission::ViewModpack,
                Permission::InstallMod,
                Permission::UninstallMod,
            ],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        roles.insert(moderator_role.id, moderator_role);
        
        // User role - basic server operations
        let user_role = Role {
            id: Uuid::new_v4(),
            name: "User".to_string(),
            permissions: vec![
                Permission::ViewServer,
                Permission::ViewModpack,
            ],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        roles.insert(user_role.id, user_role);
        
        // Read-only role - view only
        let read_only_role = Role {
            id: Uuid::new_v4(),
            name: "ReadOnly".to_string(),
            permissions: vec![
                Permission::ViewServer,
                Permission::ViewModpack,
            ],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        roles.insert(read_only_role.id, read_only_role);
        
        Ok(())
    }
    
    async fn create_default_admin(&self) -> Result<()> {
        let admin_user = User {
            id: Uuid::new_v4(),
            username: "admin".to_string(),
            email: "admin@guardian.local".to_string(),
            password_hash: self.hash_password("admin123")?,
            role: UserRole::Admin,
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_login: None,
        };
        
        let mut users = self.users.write().await;
        users.insert(admin_user.id, admin_user);
        
        Ok(())
    }
    
    pub async fn register(&self, request: RegisterRequest) -> Result<LoginResponse> {
        // Check if username already exists
        let users = self.users.read().await;
        if users.values().any(|u| u.username == request.username) {
            return Err(anyhow!("Username already exists"));
        }
        
        // Check if email already exists
        if users.values().any(|u| u.email == request.email) {
            return Err(anyhow!("Email already exists"));
        }
        
        drop(users);
        
        // Hash password
        let password_hash = self.hash_password(&request.password)?;
        
        // Create user
        let user = User {
            id: Uuid::new_v4(),
            username: request.username,
            email: request.email,
            password_hash,
            role: request.role.unwrap_or(UserRole::User),
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_login: None,
        };
        
        // Store user
        let mut users = self.users.write().await;
        users.insert(user.id, user.clone());
        drop(users);
        
        // Generate JWT token
        let token = self.generate_token(&user)?;
        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(self.jwt_expiry.as_secs() as i64);
        
        // Store session
        let mut sessions = self.user_sessions.write().await;
        sessions.insert(token.clone(), user.id);
        drop(sessions);
        
        Ok(LoginResponse {
            token,
            user,
            expires_at,
        })
    }
    
    pub async fn login(&self, request: LoginRequest) -> Result<LoginResponse> {
        // Find user by username
        let users = self.users.read().await;
        let user = users.values()
            .find(|u| u.username == request.username && u.is_active)
            .ok_or_else(|| anyhow!("Invalid username or password"))?;
        
        // Verify password
        if !self.verify_password(&request.password, &user.password_hash)? {
            return Err(anyhow!("Invalid username or password"));
        }
        
        let user = user.clone();
        drop(users);
        
        // Update last login
        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(&user.id) {
            user.last_login = Some(chrono::Utc::now());
        }
        drop(users);
        
        // Generate JWT token
        let token = self.generate_token(&user)?;
        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(self.jwt_expiry.as_secs() as i64);
        
        // Store session
        let mut sessions = self.user_sessions.write().await;
        sessions.insert(token.clone(), user.id);
        drop(sessions);
        
        Ok(LoginResponse {
            token,
            user,
            expires_at,
        })
    }
    
    pub async fn logout(&self, token: &str) -> Result<()> {
        let mut sessions = self.user_sessions.write().await;
        sessions.remove(token);
        Ok(())
    }
    
    pub async fn validate_token(&self, token: &str) -> Result<User> {
        // Check if token is in active sessions
        let user_id = {
            let sessions = self.user_sessions.read().await;
            *sessions.get(token)
                .ok_or_else(|| anyhow!("Invalid token"))?
        };
        
        // Verify JWT token
        let _claims = self.decode_token(token)?;
        
        // Get user
        let users = self.users.read().await;
        let user = users.get(&user_id)
            .ok_or_else(|| anyhow!("User not found"))?;
        
        if !user.is_active {
            return Err(anyhow!("User account is disabled"));
        }
        
        Ok(user.clone())
    }
    
    pub async fn has_permission(&self, user_id: Uuid, permission: &Permission) -> bool {
        let users = self.users.read().await;
        if let Some(user) = users.get(&user_id) {
            match user.role {
                UserRole::Admin => true, // Admin has all permissions
                _ => {
                    // Check role permissions
                    let roles = self.roles.read().await;
                    if let Some(role) = roles.values().find(|r| r.name == format!("{:?}", user.role)) {
                        role.permissions.contains(permission)
                    } else {
                        false
                    }
                }
            }
        } else {
            false
        }
    }
    
    pub async fn get_user(&self, user_id: Uuid) -> Option<User> {
        self.users.read().await.get(&user_id).cloned()
    }
    
    pub async fn get_all_users(&self) -> Vec<User> {
        self.users.read().await.values().cloned().collect()
    }
    
    pub async fn update_user(&self, user_id: Uuid, updates: UserUpdate) -> Result<User> {
        let mut users = self.users.write().await;
        if let Some(user) = users.get_mut(&user_id) {
            if let Some(username) = updates.username {
                user.username = username;
            }
            if let Some(email) = updates.email {
                user.email = email;
            }
            if let Some(role) = updates.role {
                user.role = role;
            }
            if let Some(is_active) = updates.is_active {
                user.is_active = is_active;
            }
            user.updated_at = chrono::Utc::now();
            
            Ok(user.clone())
        } else {
            Err(anyhow!("User not found"))
        }
    }
    
    pub async fn delete_user(&self, user_id: Uuid) -> Result<()> {
        let mut users = self.users.write().await;
        users.remove(&user_id);
        Ok(())
    }
    
    pub async fn get_roles(&self) -> Vec<Role> {
        self.roles.read().await.values().cloned().collect()
    }
    
    fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Password hashing failed: {}", e))?;
        Ok(password_hash.to_string())
    }
    
    fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| anyhow!("Password hash parsing failed: {}", e))?;
        let argon2 = Argon2::default();
        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }
    
    fn generate_token(&self, user: &User) -> Result<String> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as usize;
        let exp = now + self.jwt_expiry.as_secs() as usize;
        
        let claims = JwtClaims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            role: user.role.clone(),
            exp,
            iat: now,
        };
        
        let header = Header::new(Algorithm::HS256);
        let token = encode(&header, &claims, &EncodingKey::from_secret(self.jwt_secret.as_ref()))?;
        
        Ok(token)
    }
    
    fn decode_token(&self, token: &str) -> Result<JwtClaims> {
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &validation,
        )?;
        
        Ok(token_data.claims)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserUpdate {
    pub username: Option<String>,
    pub email: Option<String>,
    pub role: Option<UserRole>,
    pub is_active: Option<bool>,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Admin => "admin",
            UserRole::Moderator => "moderator",
            UserRole::User => "user",
            UserRole::ReadOnly => "readonly",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "admin" => Some(UserRole::Admin),
            "moderator" => Some(UserRole::Moderator),
            "user" => Some(UserRole::User),
            "readonly" => Some(UserRole::ReadOnly),
            _ => None,
        }
    }
}
