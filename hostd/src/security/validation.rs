use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::{Validate, ValidationError, ValidationErrors};

/// Input validation for API endpoints
pub struct ValidationService;

impl ValidationService {
    /// Validate server name
    pub fn validate_server_name(name: &str) -> Result<(), ValidationError> {
        if name.is_empty() {
            return Err(ValidationError::new("empty"));
        }
        
        if name.len() > 50 {
            return Err(ValidationError::new("too_long"));
        }
        
        if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ' ') {
            return Err(ValidationError::new("invalid_characters"));
        }
        
        Ok(())
    }

    /// Validate port number
    pub fn validate_port(port: u16) -> Result<(), ValidationError> {
        if port < 1024 {
            return Err(ValidationError::new("privileged_port"));
        }
        
        // u16 can only hold values up to 65535, so no need to check upper bound
        
        Ok(())
    }

    /// Validate memory allocation
    pub fn validate_memory(memory: u32) -> Result<(), ValidationError> {
        if memory < 512 {
            return Err(ValidationError::new("insufficient_memory"));
        }
        
        if memory > 32768 {
            return Err(ValidationError::new("excessive_memory"));
        }
        
        Ok(())
    }

    /// Validate Minecraft version
    pub fn validate_minecraft_version(version: &str) -> Result<(), ValidationError> {
        if version.is_empty() {
            return Err(ValidationError::new("empty"));
        }
        
        // Basic version format validation (e.g., "1.20.1")
        if !version.chars().any(|c| c.is_numeric()) {
            return Err(ValidationError::new("invalid_format"));
        }
        
        Ok(())
    }

    /// Validate email address
    pub fn validate_email(email: &str) -> Result<(), ValidationError> {
        if email.is_empty() {
            return Err(ValidationError::new("empty"));
        }
        
        if !email.contains('@') {
            return Err(ValidationError::new("invalid_format"));
        }
        
        if email.len() > 254 {
            return Err(ValidationError::new("too_long"));
        }
        
        Ok(())
    }

    /// Validate username
    pub fn validate_username(username: &str) -> Result<(), ValidationError> {
        if username.is_empty() {
            return Err(ValidationError::new("empty"));
        }
        
        if username.len() < 3 {
            return Err(ValidationError::new("too_short"));
        }
        
        if username.len() > 30 {
            return Err(ValidationError::new("too_long"));
        }
        
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(ValidationError::new("invalid_characters"));
        }
        
        Ok(())
    }

    /// Validate password strength
    pub fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
        if password.len() < 8 {
            return Err(ValidationError::new("too_short"));
        }
        
        if password.len() > 128 {
            return Err(ValidationError::new("too_long"));
        }
        
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_numeric());
        let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));
        
        if !has_uppercase {
            return Err(ValidationError::new("no_uppercase"));
        }
        
        if !has_lowercase {
            return Err(ValidationError::new("no_lowercase"));
        }
        
        if !has_digit {
            return Err(ValidationError::new("no_digit"));
        }
        
        if !has_special {
            return Err(ValidationError::new("no_special"));
        }
        
        Ok(())
    }

    /// Sanitize input string
    pub fn sanitize_input(input: &str) -> String {
        input
            .chars()
            .filter(|c| !c.is_control())
            .collect::<String>()
            .trim()
            .to_string()
    }

    /// Validate file path
    pub fn validate_file_path(path: &str) -> Result<(), ValidationError> {
        if path.is_empty() {
            return Err(ValidationError::new("empty"));
        }
        
        if path.contains("..") {
            return Err(ValidationError::new("path_traversal"));
        }
        
        if path.starts_with('/') || path.contains('\\') {
            return Err(ValidationError::new("absolute_path"));
        }
        
        Ok(())
    }
}

/// Server creation validation
#[derive(Debug, Deserialize, Validate)]
pub struct ServerCreationRequest {
    #[validate(length(min = 1, max = 50))]
    pub name: String,
    
    #[validate(custom = "validate_minecraft_version")]
    pub minecraft_version: String,
    
    #[validate(custom = "validate_loader")]
    pub loader: String,
    
    #[validate(custom = "validate_port")]
    pub port: u16,
    
    #[validate(custom = "validate_memory")]
    pub memory: u32,
    
    #[validate(range(min = 1, max = 100))]
    pub max_players: u32,
}

/// User registration validation
#[derive(Debug, Deserialize, Validate)]
pub struct UserRegistrationRequest {
    #[validate(email)]
    pub email: String,
    
    #[validate(custom = "validate_username")]
    pub username: String,
    
    #[validate(custom = "validate_password_strength")]
    pub password: String,
}

/// Custom validation functions
fn validate_minecraft_version(version: &str) -> Result<(), ValidationError> {
    ValidationService::validate_minecraft_version(version)
}

fn validate_loader(loader: &str) -> Result<(), ValidationError> {
    let valid_loaders = ["vanilla", "forge", "fabric", "quilt"];
    if !valid_loaders.contains(&loader) {
        return Err(ValidationError::new("invalid_loader"));
    }
    Ok(())
}

fn validate_port(port: u16) -> Result<(), ValidationError> {
    ValidationService::validate_port(port)
}

fn validate_memory(memory: u32) -> Result<(), ValidationError> {
    ValidationService::validate_memory(memory)
}

fn validate_username(username: &str) -> Result<(), ValidationError> {
    ValidationService::validate_username(username)
}

fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    ValidationService::validate_password_strength(password)
}

/// Validation error response
#[derive(Debug, Serialize)]
pub struct ValidationErrorResponse {
    pub success: bool,
    pub error: String,
    pub field_errors: HashMap<String, Vec<String>>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ValidationErrorResponse {
    pub fn new(errors: ValidationErrors) -> Self {
        let mut field_errors = HashMap::new();
        
        for (field, errors) in errors.field_errors() {
        let error_messages: Vec<String> = errors
            .iter()
            .map(|e| e.message.as_ref().map(|s| s.to_string()).unwrap_or_else(|| "Invalid value".to_string()))
            .collect();
            field_errors.insert(field.to_string(), error_messages);
        }
        
        Self {
            success: false,
            error: "Validation failed".to_string(),
            field_errors,
            timestamp: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_name_validation() {
        assert!(ValidationService::validate_server_name("My Server").is_ok());
        assert!(ValidationService::validate_server_name("").is_err());
        assert!(ValidationService::validate_server_name(&"a".repeat(51)).is_err());
    }

    #[test]
    fn test_port_validation() {
        assert!(ValidationService::validate_port(25565).is_ok());
        assert!(ValidationService::validate_port(1023).is_err());
        assert!(ValidationService::validate_port(65536).is_err());
    }

    #[test]
    fn test_memory_validation() {
        assert!(ValidationService::validate_memory(2048).is_ok());
        assert!(ValidationService::validate_memory(256).is_err());
        assert!(ValidationService::validate_memory(65536).is_err());
    }

    #[test]
    fn test_password_strength() {
        assert!(ValidationService::validate_password_strength("Password123!").is_ok());
        assert!(ValidationService::validate_password_strength("weak").is_err());
        assert!(ValidationService::validate_password_strength("nouppercase123!").is_err());
    }

    #[test]
    fn test_input_sanitization() {
        let input = "  Hello\tWorld\n  ";
        let sanitized = ValidationService::sanitize_input(input);
        assert_eq!(sanitized, "Hello\tWorld");
    }
}