use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::Aead;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};

/// Encryption service for sensitive data
pub struct EncryptionService {
    key: [u8; 32],
}

impl EncryptionService {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    /// Generate a new encryption key
    pub fn generate_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        rand::thread_rng().fill(&mut key);
        key
    }

    /// Encrypt sensitive data
    pub fn encrypt(&self, plaintext: &str) -> Result<EncryptedData, EncryptionError> {
        let key = Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(key);
        
        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt the data
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|_| EncryptionError::EncryptionFailed)?;
        
        Ok(EncryptedData {
            ciphertext,
            nonce: nonce_bytes.to_vec(),
        })
    }

    /// Decrypt sensitive data
    pub fn decrypt(&self, encrypted_data: &EncryptedData) -> Result<String, EncryptionError> {
        let key = Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(&encrypted_data.nonce);
        
        let plaintext = cipher
            .decrypt(nonce, encrypted_data.ciphertext.as_slice())
            .map_err(|_| EncryptionError::DecryptionFailed)?;
        
        String::from_utf8(plaintext)
            .map_err(|_| EncryptionError::InvalidUtf8)
    }

    /// Hash password with salt
    pub fn hash_password(&self, password: &str) -> Result<String, EncryptionError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| EncryptionError::HashingFailed)?;
        
        Ok(password_hash.to_string())
    }

    /// Verify password against hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, EncryptionError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|_| EncryptionError::VerificationFailed)?;
        
        let argon2 = Argon2::default();
        let is_valid = argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok();
        
        Ok(is_valid)
    }

    /// Generate random salt
    fn generate_salt(&self) -> [u8; 16] {
        let mut salt = [0u8; 16];
        rand::thread_rng().fill(&mut salt);
        salt
    }
}

/// Encrypted data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

/// Encryption errors
#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("Encryption failed")]
    EncryptionFailed,
    #[error("Decryption failed")]
    DecryptionFailed,
    #[error("Invalid UTF-8")]
    InvalidUtf8,
    #[error("Hashing failed")]
    HashingFailed,
    #[error("Verification failed")]
    VerificationFailed,
}

/// Secure configuration manager
pub struct SecureConfig {
    encryption_service: EncryptionService,
    encrypted_values: HashMap<String, EncryptedData>,
}

impl SecureConfig {
    pub fn new(encryption_key: [u8; 32]) -> Self {
        Self {
            encryption_service: EncryptionService::new(encryption_key),
            encrypted_values: HashMap::new(),
        }
    }

    /// Store encrypted configuration value
    pub fn set_encrypted(&mut self, key: &str, value: &str) -> Result<(), EncryptionError> {
        let encrypted = self.encryption_service.encrypt(value)?;
        self.encrypted_values.insert(key.to_string(), encrypted);
        Ok(())
    }

    /// Retrieve and decrypt configuration value
    pub fn get_encrypted(&self, key: &str) -> Result<Option<String>, EncryptionError> {
        if let Some(encrypted) = self.encrypted_values.get(key) {
            let decrypted = self.encryption_service.decrypt(encrypted)?;
            Ok(Some(decrypted))
        } else {
            Ok(None)
        }
    }

    /// Check if encrypted value exists
    pub fn has_encrypted(&self, key: &str) -> bool {
        self.encrypted_values.contains_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let key = EncryptionService::generate_key();
        let encryption_service = EncryptionService::new(key);
        
        let plaintext = "sensitive data";
        let encrypted = encryption_service.encrypt(plaintext).unwrap();
        let decrypted = encryption_service.decrypt(&encrypted).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_password_hashing() {
        let key = EncryptionService::generate_key();
        let encryption_service = EncryptionService::new(key);
        
        let password = "test_password";
        let hash = encryption_service.hash_password(password).unwrap();
        let is_valid = encryption_service.verify_password(password, &hash).unwrap();
        
        assert!(is_valid);
        
        let is_invalid = encryption_service.verify_password("wrong_password", &hash).unwrap();
        assert!(!is_invalid);
    }

    #[test]
    fn test_secure_config() {
        let key = EncryptionService::generate_key();
        let mut config = SecureConfig::new(key);
        
        config.set_encrypted("api_key", "secret_api_key").unwrap();
        let retrieved = config.get_encrypted("api_key").unwrap().unwrap();
        
        assert_eq!(retrieved, "secret_api_key");
    }
}