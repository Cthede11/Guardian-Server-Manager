use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Encryption service for sensitive data
pub struct EncryptionService {
    argon2: Argon2<'static>,
    encryption_keys: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl EncryptionService {
    pub fn new() -> Self {
        Self {
            argon2: Argon2::default(),
            encryption_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Hash a password using Argon2
    pub fn hash_password(&self, password: &str) -> Result<String, EncryptionError> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| EncryptionError::HashingFailed)?;
        
        Ok(password_hash.to_string())
    }
    
    /// Verify a password against its hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, EncryptionError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|_| EncryptionError::InvalidHash)?;
        
        let result = self.argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok();
        
        Ok(result)
    }
    
    /// Encrypt sensitive data
    pub async fn encrypt_data(&self, data: &str, key_id: &str) -> Result<String, EncryptionError> {
        let keys = self.encryption_keys.read().await;
        let key = keys.get(key_id)
            .ok_or(EncryptionError::KeyNotFound)?;
        
        // Simple XOR encryption for demonstration
        // In production, use AES-256-GCM
        let encrypted: Vec<u8> = data
            .bytes()
            .zip(key.iter().cycle())
            .map(|(b, k)| b ^ k)
            .collect();
        
        use base64::{Engine as _, engine::general_purpose};
        Ok(general_purpose::STANDARD.encode(encrypted))
    }
    
    /// Decrypt sensitive data
    pub async fn decrypt_data(&self, encrypted_data: &str, key_id: &str) -> Result<String, EncryptionError> {
        let keys = self.encryption_keys.read().await;
        let key = keys.get(key_id)
            .ok_or(EncryptionError::KeyNotFound)?;
        
        use base64::{Engine as _, engine::general_purpose};
        let encrypted = general_purpose::STANDARD.decode(encrypted_data)
            .map_err(|_| EncryptionError::DecryptionFailed)?;
        
        // Simple XOR decryption for demonstration
        // In production, use AES-256-GCM
        let decrypted: Vec<u8> = encrypted
            .iter()
            .zip(key.iter().cycle())
            .map(|(b, k)| b ^ k)
            .collect();
        
        String::from_utf8(decrypted)
            .map_err(|_| EncryptionError::DecryptionFailed)
    }
    
    /// Generate a new encryption key
    pub async fn generate_key(&self, key_id: &str) -> Result<(), EncryptionError> {
        let mut keys = self.encryption_keys.write().await;
        let key = (0..32).map(|_| rand::random::<u8>()).collect();
        keys.insert(key_id.to_string(), key);
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("Password hashing failed")]
    HashingFailed,
    #[error("Invalid hash format")]
    InvalidHash,
    #[error("Key not found")]
    KeyNotFound,
    #[error("Encryption failed")]
    EncryptionFailed,
    #[error("Decryption failed")]
    DecryptionFailed,
}

/// Secure random string generator
pub struct SecureRandom;

impl SecureRandom {
    pub fn generate_token(length: usize) -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();
        
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
    
    pub fn generate_uuid() -> String {
        uuid::Uuid::new_v4().to_string()
    }
    
    pub fn generate_api_key() -> String {
        format!("gsk_{}", Self::generate_token(32))
    }
}

/// JWT token management
pub struct JWTManager {
    secret: String,
    issuer: String,
    audience: String,
}

impl JWTManager {
    pub fn new(secret: String, issuer: String, audience: String) -> Self {
        Self {
            secret,
            issuer,
            audience,
        }
    }
    
    pub fn generate_token(&self, user_id: &str, expires_in_hours: u64) -> Result<String, JWTError> {
        let now = chrono::Utc::now();
        let exp = now + chrono::Duration::hours(expires_in_hours as i64);
        
        let header = jsonwebtoken::Header::default();
        let claims = JWTPayload {
            sub: user_id.to_string(),
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
        };
        
        jsonwebtoken::encode(&header, &claims, &jsonwebtoken::EncodingKey::from_secret(self.secret.as_ref()))
            .map_err(|_| JWTError::TokenGenerationFailed)
    }
    
    pub fn verify_token(&self, token: &str) -> Result<JWTPayload, JWTError> {
        let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
        
        jsonwebtoken::decode::<JWTPayload>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(self.secret.as_ref()),
            &validation,
        )
        .map(|data| data.claims)
        .map_err(|_| JWTError::TokenVerificationFailed)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JWTPayload {
    pub sub: String,    // subject (user ID)
    pub iss: String,    // issuer
    pub aud: String,    // audience
    pub iat: i64,       // issued at
    pub exp: i64,       // expiration
}

#[derive(Debug, thiserror::Error)]
pub enum JWTError {
    #[error("Token generation failed")]
    TokenGenerationFailed,
    #[error("Token verification failed")]
    TokenVerificationFailed,
}
