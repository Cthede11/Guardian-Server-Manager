use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};

/// Secret storage service for API keys and sensitive data
pub struct SecretStorage {
    secrets: Arc<RwLock<HashMap<String, SecretEntry>>>,
    encryption_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SecretEntry {
    value: String,
    encrypted: bool,
    created_at: chrono::DateTime<chrono::Utc>,
    last_accessed: Option<chrono::DateTime<chrono::Utc>>,
}

impl SecretStorage {
    pub fn new() -> Self {
        Self {
            secrets: Arc::new(RwLock::new(HashMap::new())),
            encryption_key: None,
        }
    }

    pub fn with_encryption(encryption_key: String) -> Self {
        Self {
            secrets: Arc::new(RwLock::new(HashMap::new())),
            encryption_key: Some(encryption_key),
        }
    }

    /// Store a secret value
    pub async fn store_secret(&self, key: &str, value: &str) -> Result<()> {
        let mut secrets = self.secrets.write().await;
        
        let processed_value = if let Some(enc_key) = &self.encryption_key {
            self.encrypt_value(value, enc_key)?
        } else {
            value.to_string()
        };
        
        let entry = SecretEntry {
            value: processed_value,
            encrypted: self.encryption_key.is_some(),
            created_at: chrono::Utc::now(),
            last_accessed: None,
        };
        
        secrets.insert(key.to_string(), entry);
        Ok(())
    }

    /// Retrieve a secret value
    pub async fn get_secret(&self, key: &str) -> Result<Option<String>> {
        let mut secrets = self.secrets.write().await;
        
        if let Some(entry) = secrets.get_mut(key) {
            entry.last_accessed = Some(chrono::Utc::now());
            
            let value = if entry.encrypted {
                if let Some(enc_key) = &self.encryption_key {
                    self.decrypt_value(&entry.value, enc_key)?
                } else {
                    return Err(anyhow::anyhow!("Secret is encrypted but no encryption key provided"));
                }
            } else {
                entry.value.clone()
            };
            
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Check if a secret exists
    pub async fn has_secret(&self, key: &str) -> bool {
        let secrets = self.secrets.read().await;
        secrets.contains_key(key)
    }

    /// Remove a secret
    pub async fn remove_secret(&self, key: &str) -> Result<bool> {
        let mut secrets = self.secrets.write().await;
        Ok(secrets.remove(key).is_some())
    }

    /// List all secret keys (without values)
    pub async fn list_secrets(&self) -> Vec<String> {
        let secrets = self.secrets.read().await;
        secrets.keys().cloned().collect()
    }

    /// Encrypt a value using a simple XOR cipher (for demonstration)
    /// In production, use a proper encryption library like AES
    fn encrypt_value(&self, value: &str, key: &str) -> Result<String> {
        let key_bytes = key.as_bytes();
        let value_bytes = value.as_bytes();
        let mut encrypted = Vec::new();
        
        for (i, &byte) in value_bytes.iter().enumerate() {
            encrypted.push(byte ^ key_bytes[i % key_bytes.len()]);
        }
        
        Ok(base64::encode(encrypted))
    }

    /// Decrypt a value using a simple XOR cipher
    fn decrypt_value(&self, encrypted: &str, key: &str) -> Result<String> {
        let key_bytes = key.as_bytes();
        let encrypted_bytes = base64::decode(encrypted)
            .context("Failed to decode base64 encrypted value")?;
        let mut decrypted = Vec::new();
        
        for (i, &byte) in encrypted_bytes.iter().enumerate() {
            decrypted.push(byte ^ key_bytes[i % key_bytes.len()]);
        }
        
        String::from_utf8(decrypted)
            .context("Failed to convert decrypted bytes to string")
    }

    /// Store API keys securely
    pub async fn store_api_key(&self, provider: &str, api_key: &str) -> Result<()> {
        let key = format!("api_key_{}", provider);
        self.store_secret(&key, api_key).await
    }

    /// Retrieve API key
    pub async fn get_api_key(&self, provider: &str) -> Result<Option<String>> {
        let key = format!("api_key_{}", provider);
        self.get_secret(&key).await
    }

    /// Store configuration secrets
    pub async fn store_config_secret(&self, config_key: &str, value: &str) -> Result<()> {
        let key = format!("config_{}", config_key);
        self.store_secret(&key, value).await
    }

    /// Retrieve configuration secret
    pub async fn get_config_secret(&self, config_key: &str) -> Result<Option<String>> {
        let key = format!("config_{}", config_key);
        self.get_secret(&key).await
    }
}

/// API key manager for external services
pub struct ApiKeyManager {
    secret_storage: SecretStorage,
}

impl ApiKeyManager {
    pub fn new() -> Self {
        Self {
            secret_storage: SecretStorage::new(),
        }
    }

    pub fn with_encryption(encryption_key: String) -> Self {
        Self {
            secret_storage: SecretStorage::with_encryption(encryption_key),
        }
    }

    /// Set CurseForge API key
    pub async fn set_curseforge_key(&self, api_key: &str) -> Result<()> {
        self.secret_storage.store_api_key("curseforge", api_key).await
    }

    /// Get CurseForge API key
    pub async fn get_curseforge_key(&self) -> Result<Option<String>> {
        self.secret_storage.get_api_key("curseforge").await
    }

    /// Set Modrinth API key
    pub async fn set_modrinth_key(&self, api_key: &str) -> Result<()> {
        self.secret_storage.store_api_key("modrinth", api_key).await
    }

    /// Get Modrinth API key
    pub async fn get_modrinth_key(&self) -> Result<Option<String>> {
        self.secret_storage.get_api_key("modrinth").await
    }

    /// Check if API key is available for provider
    pub async fn has_api_key(&self, provider: &str) -> bool {
        self.secret_storage.has_secret(&format!("api_key_{}", provider)).await
    }

    /// Validate API key format (basic validation)
    pub fn validate_api_key_format(&self, api_key: &str) -> Result<()> {
        if api_key.is_empty() {
            return Err(anyhow::anyhow!("API key cannot be empty"));
        }
        
        if api_key.len() < 10 {
            return Err(anyhow::anyhow!("API key too short"));
        }
        
        if api_key.len() > 200 {
            return Err(anyhow::anyhow!("API key too long"));
        }
        
        // Basic format validation
        if !api_key.chars().all(|c| c.is_alphanumeric() || "_-.".contains(c)) {
            return Err(anyhow::anyhow!("API key contains invalid characters"));
        }
        
        Ok(())
    }

    /// Test API key by making a simple request
    pub async fn test_api_key(&self, provider: &str, api_key: &str) -> Result<bool> {
        match provider {
            "curseforge" => self.test_curseforge_key(api_key).await,
            "modrinth" => self.test_modrinth_key(api_key).await,
            _ => Err(anyhow::anyhow!("Unknown provider: {}", provider)),
        }
    }

    async fn test_curseforge_key(&self, api_key: &str) -> Result<bool> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.curseforge.com/v1/games")
            .header("x-api-key", api_key)
            .send()
            .await?;
        
        Ok(response.status().is_success())
    }

    async fn test_modrinth_key(&self, api_key: &str) -> Result<bool> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.modrinth.com/v2/user")
            .header("Authorization", api_key)
            .send()
            .await?;
        
        Ok(response.status().is_success())
    }
}

/// Secure logging for sensitive data
pub struct SecureLogger;

impl SecureLogger {
    /// Log a message with sensitive data redacted
    pub fn log_with_redaction(message: &str, sensitive_data: &[&str]) {
        let mut redacted_message = message.to_string();
        
        for &sensitive in sensitive_data {
            let redacted = "*".repeat(sensitive.len().min(8));
            redacted_message = redacted_message.replace(sensitive, &redacted);
        }
        
        tracing::info!("{}", redacted_message);
    }

    /// Log API key operations without exposing the key
    pub fn log_api_key_operation(operation: &str, provider: &str, success: bool) {
        let status = if success { "success" } else { "failed" };
        tracing::info!("API key {} for {} {}", operation, provider, status);
    }

    /// Log configuration changes without exposing sensitive values
    pub fn log_config_change(key: &str, success: bool) {
        let status = if success { "success" } else { "failed" };
        tracing::info!("Configuration change for {} {}", key, status);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_secret_storage() {
        let storage = SecretStorage::new();
        
        // Test storing and retrieving a secret
        storage.store_secret("test_key", "test_value").await.unwrap();
        let retrieved = storage.get_secret("test_key").await.unwrap();
        assert_eq!(retrieved, Some("test_value".to_string()));
        
        // Test non-existent key
        let not_found = storage.get_secret("non_existent").await.unwrap();
        assert_eq!(not_found, None);
    }

    #[tokio::test]
    async fn test_api_key_manager() {
        let manager = ApiKeyManager::new();
        
        // Test API key validation
        assert!(manager.validate_api_key_format("valid_key_123").is_ok());
        assert!(manager.validate_api_key_format("").is_err());
        assert!(manager.validate_api_key_format("short").is_err());
    }

    #[test]
    fn test_secure_logger() {
        SecureLogger::log_with_redaction(
            "API key is: secret123",
            &["secret123"]
        );
        
        SecureLogger::log_api_key_operation("set", "curseforge", true);
        SecureLogger::log_config_change("database_url", true);
    }
}
