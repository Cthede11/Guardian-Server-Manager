use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use crate::core::error_handler::{AppError, Result};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Manages secure generation and storage of credentials
#[derive(Debug)]
pub struct CredentialManager {
    /// In-memory storage of credentials (in production, use encrypted storage)
    credentials: Arc<RwLock<HashMap<String, CredentialInfo>>>,
    /// Master key for encryption (in production, load from secure key management)
    master_key: String,
}

#[derive(Debug, Clone)]
pub struct CredentialInfo {
    pub value: String,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub server_id: Option<Uuid>,
    pub credential_type: CredentialType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CredentialType {
    RconPassword,
    JwtSecret,
    DatabasePassword,
    ApiKey,
}

impl CredentialManager {
    pub fn new() -> Self {
        Self {
            credentials: Arc::new(RwLock::new(HashMap::new())),
            master_key: Self::generate_master_key(),
        }
    }

    /// Generate a secure master key
    fn generate_master_key() -> String {
        let mut rng = thread_rng();
        (0..32)
            .map(|_| rng.sample(Alphanumeric) as char)
            .collect()
    }

    /// Generate a secure random password
    pub fn generate_password(&self, length: usize) -> String {
        let mut rng = thread_rng();
        (0..length)
            .map(|_| rng.sample(Alphanumeric) as char)
            .collect()
    }

    /// Generate a secure random token
    pub fn generate_token(&self, length: usize) -> String {
        let mut rng = thread_rng();
        (0..length)
            .map(|_| rng.sample(Alphanumeric) as char)
            .collect()
    }

    /// Store a credential securely
    pub async fn store_credential(
        &self,
        key: String,
        value: String,
        credential_type: CredentialType,
        server_id: Option<Uuid>,
        expires_in_hours: Option<u64>,
    ) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let expires_at = expires_in_hours.map(|hours| now + (hours * 3600));

        let credential_info = CredentialInfo {
            value: self.encrypt_value(&value)?,
            created_at: now,
            expires_at,
            server_id,
            credential_type,
        };

        let mut credentials = self.credentials.write().await;
        credentials.insert(key, credential_info);

        Ok(())
    }

    /// Retrieve a credential
    pub async fn get_credential(&self, key: &str) -> Result<Option<String>> {
        let credentials = self.credentials.read().await;
        
        if let Some(credential_info) = credentials.get(key) {
            // Check if expired
            if let Some(expires_at) = credential_info.expires_at {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                
                if now > expires_at {
                    return Ok(None);
                }
            }

            Ok(Some(self.decrypt_value(&credential_info.value)?))
        } else {
            Ok(None)
        }
    }

    /// Generate and store a new RCON password for a server
    pub async fn generate_rcon_password(&self, server_id: Uuid) -> Result<String> {
        let password = self.generate_password(16);
        let key = format!("rcon_password_{}", server_id);
        
        self.store_credential(
            key,
            password.clone(),
            CredentialType::RconPassword,
            Some(server_id),
            None, // RCON passwords don't expire
        ).await?;

        Ok(password)
    }

    /// Get RCON password for a server
    pub async fn get_rcon_password(&self, server_id: Uuid) -> Result<Option<String>> {
        let key = format!("rcon_password_{}", server_id);
        self.get_credential(&key).await
    }

    /// Generate and store a JWT secret
    pub async fn generate_jwt_secret(&self) -> Result<String> {
        let secret = self.generate_token(64);
        let key = "jwt_secret".to_string();
        
        self.store_credential(
            key,
            secret.clone(),
            CredentialType::JwtSecret,
            None,
            None, // JWT secrets don't expire
        ).await?;

        Ok(secret)
    }

    /// Get JWT secret
    pub async fn get_jwt_secret(&self) -> Result<Option<String>> {
        self.get_credential("jwt_secret").await
    }

    /// Generate and store a database password
    pub async fn generate_database_password(&self) -> Result<String> {
        let password = self.generate_password(32);
        let key = "database_password".to_string();
        
        self.store_credential(
            key,
            password.clone(),
            CredentialType::DatabasePassword,
            None,
            None, // Database passwords don't expire
        ).await?;

        Ok(password)
    }

    /// Get database password
    pub async fn get_database_password(&self) -> Result<Option<String>> {
        self.get_credential("database_password").await
    }

    /// Generate and store an API key
    pub async fn generate_api_key(&self, server_id: Option<Uuid>) -> Result<String> {
        let api_key = self.generate_token(32);
        let key = if let Some(server_id) = server_id {
            format!("api_key_{}", server_id)
        } else {
            "api_key_global".to_string()
        };
        
        self.store_credential(
            key,
            api_key.clone(),
            CredentialType::ApiKey,
            server_id,
            Some(24 * 30), // Expires in 30 days
        ).await?;

        Ok(api_key)
    }

    /// Get API key
    pub async fn get_api_key(&self, server_id: Option<Uuid>) -> Result<Option<String>> {
        let key = if let Some(server_id) = server_id {
            format!("api_key_{}", server_id)
        } else {
            "api_key_global".to_string()
        };
        
        self.get_credential(&key).await
    }

    /// Remove a credential
    pub async fn remove_credential(&self, key: &str) -> Result<()> {
        let mut credentials = self.credentials.write().await;
        credentials.remove(key);
        Ok(())
    }

    /// Remove all credentials for a server
    pub async fn remove_server_credentials(&self, server_id: Uuid) -> Result<()> {
        let mut credentials = self.credentials.write().await;
        
        // Remove RCON password
        credentials.remove(&format!("rcon_password_{}", server_id));
        
        // Remove API key
        credentials.remove(&format!("api_key_{}", server_id));

        Ok(())
    }

    /// List all credentials (for debugging - in production, this should be restricted)
    pub async fn list_credentials(&self) -> Result<Vec<String>> {
        let credentials = self.credentials.read().await;
        Ok(credentials.keys().cloned().collect())
    }

    /// Simple encryption (in production, use proper encryption like AES)
    fn encrypt_value(&self, value: &str) -> Result<String> {
        // This is a simple XOR encryption - in production, use proper encryption
        let mut encrypted = String::new();
        let key_bytes = self.master_key.as_bytes();
        
        for (i, byte) in value.bytes().enumerate() {
            let key_byte = key_bytes[i % key_bytes.len()];
            encrypted.push((byte ^ key_byte) as char);
        }
        
        Ok(base64::encode(encrypted))
    }

    /// Simple decryption (in production, use proper decryption)
    fn decrypt_value(&self, encrypted: &str) -> Result<String> {
        let decoded = base64::decode(encrypted)
            .map_err(|e| AppError::InternalError {
                message: format!("Failed to decode credential: {}", e),
                component: "credential_manager".to_string(),
                details: Some("decrypt".to_string()),
            })?;
        
        let mut decrypted = String::new();
        let key_bytes = self.master_key.as_bytes();
        
        for (i, &byte) in decoded.iter().enumerate() {
            let key_byte = key_bytes[i % key_bytes.len()];
            decrypted.push((byte ^ key_byte) as char);
        }
        
        Ok(decrypted)
    }

    /// Clean up expired credentials
    pub async fn cleanup_expired(&self) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut credentials = self.credentials.write().await;
        credentials.retain(|_, credential_info| {
            if let Some(expires_at) = credential_info.expires_at {
                now <= expires_at
            } else {
                true // Never expires
            }
        });

        Ok(())
    }
}

impl Default for CredentialManager {
    fn default() -> Self {
        Self::new()
    }
}
