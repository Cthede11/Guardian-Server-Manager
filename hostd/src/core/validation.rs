use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use regex::Regex;
use crate::core::error_handler::{AppError, Result};

/// Validation rule trait
pub trait ValidationRule<T> {
    fn validate(&self, value: &T) -> Result<()>;
    fn error_message(&self) -> String;
}

/// String validation rules
pub struct StringValidationRules {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<Regex>,
    pub required: bool,
    pub trim: bool,
}

impl StringValidationRules {
    pub fn new() -> Self {
        Self {
            min_length: None,
            max_length: None,
            pattern: None,
            required: false,
            trim: false,
        }
    }
    
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
    
    pub fn min_length(mut self, length: usize) -> Self {
        self.min_length = Some(length);
        self
    }
    
    pub fn max_length(mut self, length: usize) -> Self {
        self.max_length = Some(length);
        self
    }
    
    pub fn pattern(mut self, pattern: Regex) -> Self {
        self.pattern = Some(pattern);
        self
    }
    
    pub fn trim(mut self) -> Self {
        self.trim = true;
        self
    }
}

impl ValidationRule<String> for StringValidationRules {
    fn validate(&self, value: &String) -> Result<()> {
        let trimmed_value = if self.trim { value.trim() } else { value };
        
        if self.required && trimmed_value.is_empty() {
            return Err(AppError::validation_error(
                "string",
                value,
                "required",
                "Value is required"
            ));
        }
        
        if let Some(min_len) = self.min_length {
            if trimmed_value.len() < min_len {
                return Err(AppError::validation_error(
                    "string",
                    value,
                    "min_length",
                    format!("Value must be at least {} characters long", min_len)
                ));
            }
        }
        
        if let Some(max_len) = self.max_length {
            if trimmed_value.len() > max_len {
                return Err(AppError::validation_error(
                    "string",
                    value,
                    "max_length",
                    format!("Value must be at most {} characters long", max_len)
                ));
            }
        }
        
        if let Some(pattern) = &self.pattern {
            if !pattern.is_match(trimmed_value) {
                return Err(AppError::validation_error(
                    "string",
                    value,
                    "pattern",
                    "Value does not match required pattern"
                ));
            }
        }
        
        Ok(())
    }
    
    fn error_message(&self) -> String {
        "String validation failed".to_string()
    }
}

/// Numeric validation rules
pub struct NumericValidationRules {
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub integer_only: bool,
    pub positive_only: bool,
}

impl NumericValidationRules {
    pub fn new() -> Self {
        Self {
            min_value: None,
            max_value: None,
            integer_only: false,
            positive_only: false,
        }
    }
    
    pub fn min_value(mut self, value: f64) -> Self {
        self.min_value = Some(value);
        self
    }
    
    pub fn max_value(mut self, value: f64) -> Self {
        self.max_value = Some(value);
        self
    }
    
    pub fn integer_only(mut self) -> Self {
        self.integer_only = true;
        self
    }
    
    pub fn positive_only(mut self) -> Self {
        self.positive_only = true;
        self
    }
}

impl ValidationRule<f64> for NumericValidationRules {
    fn validate(&self, value: &f64) -> Result<()> {
        if self.integer_only && value.fract() != 0.0 {
            return Err(AppError::validation_error(
                "number",
                &value.to_string(),
                "integer_only",
                "Value must be an integer"
            ));
        }
        
        if self.positive_only && *value <= 0.0 {
            return Err(AppError::validation_error(
                "number",
                &value.to_string(),
                "positive_only",
                "Value must be positive"
            ));
        }
        
        if let Some(min_val) = self.min_value {
            if *value < min_val {
                return Err(AppError::validation_error(
                    "number",
                    &value.to_string(),
                    "min_value",
                    format!("Value must be at least {}", min_val)
                ));
            }
        }
        
        if let Some(max_val) = self.max_value {
            if *value > max_val {
                return Err(AppError::validation_error(
                    "number",
                    &value.to_string(),
                    "max_value",
                    format!("Value must be at most {}", max_val)
                ));
            }
        }
        
        Ok(())
    }
    
    fn error_message(&self) -> String {
        "Numeric validation failed".to_string()
    }
}

/// Email validation
pub struct EmailValidation;

impl ValidationRule<String> for EmailValidation {
    fn validate(&self, value: &String) -> Result<()> {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .map_err(|_| AppError::internal_error("validation", "Failed to compile email regex"))?;
        
        if !email_regex.is_match(value) {
            return Err(AppError::validation_error(
                "email",
                value,
                "format",
                "Invalid email format"
            ));
        }
        
        Ok(())
    }
    
    fn error_message(&self) -> String {
        "Email validation failed".to_string()
    }
}

/// URL validation
pub struct UrlValidation;

impl ValidationRule<String> for UrlValidation {
    fn validate(&self, value: &String) -> Result<()> {
        let url_regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$")
            .map_err(|_| AppError::internal_error("validation", "Failed to compile URL regex"))?;
        
        if !url_regex.is_match(value) {
            return Err(AppError::validation_error(
                "url",
                value,
                "format",
                "Invalid URL format"
            ));
        }
        
        Ok(())
    }
    
    fn error_message(&self) -> String {
        "URL validation failed".to_string()
    }
}

/// Minecraft version validation
pub struct MinecraftVersionValidation;

impl ValidationRule<String> for MinecraftVersionValidation {
    fn validate(&self, value: &String) -> Result<()> {
        let version_regex = Regex::new(r"^\d+\.\d+(\.\d+)?$")
            .map_err(|_| AppError::internal_error("validation", "Failed to compile version regex"))?;
        
        if !version_regex.is_match(value) {
            return Err(AppError::validation_error(
                "minecraft_version",
                value,
                "format",
                "Invalid Minecraft version format (expected: x.y.z)"
            ));
        }
        
        // Parse version components
        let parts: Vec<&str> = value.split('.').collect();
        if parts.len() < 2 || parts.len() > 3 {
            return Err(AppError::validation_error(
                "minecraft_version",
                value,
                "format",
                "Minecraft version must have 2 or 3 parts (x.y or x.y.z)"
            ));
        }
        
        // Validate major version
        if let Ok(major) = parts[0].parse::<u32>() {
            if major < 1 {
                return Err(AppError::validation_error(
                    "minecraft_version",
                    value,
                    "range",
                    "Major version must be at least 1"
                ));
            }
        } else {
            return Err(AppError::validation_error(
                "minecraft_version",
                value,
                "format",
                "Major version must be a number"
            ));
        }
        
        Ok(())
    }
    
    fn error_message(&self) -> String {
        "Minecraft version validation failed".to_string()
    }
}

/// Port validation
pub struct PortValidation;

impl ValidationRule<u16> for PortValidation {
    fn validate(&self, value: &u16) -> Result<()> {
        if *value < 1024 {
            return Err(AppError::validation_error(
                "port",
                &value.to_string(),
                "range",
                "Port must be at least 1024 (privileged ports not allowed)"
            ));
        }
        
        if *value > 65535 {
            return Err(AppError::validation_error(
                "port",
                &value.to_string(),
                "range",
                "Port must be at most 65535"
            ));
        }
        
        Ok(())
    }
    
    fn error_message(&self) -> String {
        "Port validation failed".to_string()
    }
}

/// UUID validation
pub struct UuidValidation;

impl ValidationRule<String> for UuidValidation {
    fn validate(&self, value: &String) -> Result<()> {
        let uuid_regex = Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")
            .map_err(|_| AppError::internal_error("validation", "Failed to compile UUID regex"))?;
        
        if !uuid_regex.is_match(value) {
            return Err(AppError::validation_error(
                "uuid",
                value,
                "format",
                "Invalid UUID format"
            ));
        }
        
        Ok(())
    }
    
    fn error_message(&self) -> String {
        "UUID validation failed".to_string()
    }
}

/// Validator struct for complex validation
pub struct Validator<T> {
    rules: Vec<Box<dyn ValidationRule<T> + Send + Sync>>,
}

impl<T> Validator<T> {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }
    
    pub fn add_rule(mut self, rule: Box<dyn ValidationRule<T> + Send + Sync>) -> Self {
        self.rules.push(rule);
        self
    }
    
    pub fn validate(&self, value: &T) -> Result<()> {
        for rule in &self.rules {
            rule.validate(value)?;
        }
        Ok(())
    }
}

impl<T> Default for Validator<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for Validator<T> {
    fn clone(&self) -> Self {
        Self {
            rules: Vec::new(), // Cannot clone trait objects, so create empty validator
        }
    }
}

/// Common validation schemas
pub struct ValidationSchemas;

impl ValidationSchemas {
    /// Server name validation
    pub fn server_name() -> Validator<String> {
        Validator::new()
            .add_rule(Box::new(StringValidationRules::new()
                .required()
                .min_length(3)
                .max_length(50)
                .pattern(Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap())
            ))
    }
    
    /// Username validation
    pub fn username() -> Validator<String> {
        Validator::new()
            .add_rule(Box::new(StringValidationRules::new()
                .required()
                .min_length(3)
                .max_length(30)
                .pattern(Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap())
            ))
    }
    
    /// Password validation
    pub fn password() -> Validator<String> {
        Validator::new()
            .add_rule(Box::new(StringValidationRules::new()
                .required()
                .min_length(8)
                .max_length(128)
            ))
    }
    
    /// Email validation
    pub fn email() -> Validator<String> {
        Validator::new()
            .add_rule(Box::new(StringValidationRules::new()
                .required()
                .trim()
            ))
            .add_rule(Box::new(EmailValidation))
    }
    
    /// Minecraft version validation
    pub fn minecraft_version() -> Validator<String> {
        Validator::new()
            .add_rule(Box::new(StringValidationRules::new()
                .required()
            ))
            .add_rule(Box::new(MinecraftVersionValidation))
    }
    
    /// Port validation
    pub fn port() -> Validator<u16> {
        Validator::new()
            .add_rule(Box::new(PortValidation))
    }
    
    /// Memory validation (in MB)
    pub fn memory() -> Validator<f64> {
        Validator::new()
            .add_rule(Box::new(NumericValidationRules::new()
                .min_value(512.0)
                .max_value(32768.0)
                .integer_only()
                .positive_only()
            ))
    }
    
    /// Max players validation
    pub fn max_players() -> Validator<f64> {
        Validator::new()
            .add_rule(Box::new(NumericValidationRules::new()
                .min_value(1.0)
                .max_value(1000.0)
                .integer_only()
                .positive_only()
            ))
    }
}

/// Validation result with detailed error information
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub value: String,
    pub constraint: String,
    pub message: String,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }
    
    pub fn add_error(&mut self, field: &str, value: &str, constraint: &str, message: &str) {
        self.is_valid = false;
        self.errors.push(ValidationError {
            field: field.to_string(),
            value: value.to_string(),
            constraint: constraint.to_string(),
            message: message.to_string(),
        });
    }
    
    pub fn merge(&mut self, other: ValidationResult) {
        if !other.is_valid {
            self.is_valid = false;
            self.errors.extend(other.errors);
        }
    }
}

/// Macro for easy validation
#[macro_export]
macro_rules! validate {
    ($validator:expr, $value:expr) => {
        $validator.validate($value)
    };
}

/// Macro for validating multiple fields
#[macro_export]
macro_rules! validate_fields {
    ($($field:expr => $validator:expr),*) => {
        {
            let mut result = ValidationResult::new();
            $(
                if let Err(error) = $validator.validate($field) {
                    if let AppError::ValidationError { field, value, constraint, message } = error {
                        result.add_error(&field, &value, &constraint, &message);
                    }
                }
            )*
            result
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_string_validation() {
        let rules = StringValidationRules::new()
            .required()
            .min_length(3)
            .max_length(10);
        
        assert!(rules.validate(&"hello".to_string()).is_ok());
        assert!(rules.validate(&"".to_string()).is_err());
        assert!(rules.validate(&"hi".to_string()).is_err());
        assert!(rules.validate(&"verylongstring".to_string()).is_err());
    }
    
    #[test]
    fn test_email_validation() {
        let email_validator = EmailValidation;
        
        assert!(email_validator.validate(&"test@example.com".to_string()).is_ok());
        assert!(email_validator.validate(&"invalid-email".to_string()).is_err());
        assert!(email_validator.validate(&"@example.com".to_string()).is_err());
    }
    
    #[test]
    fn test_minecraft_version_validation() {
        let version_validator = MinecraftVersionValidation;
        
        assert!(version_validator.validate(&"1.20.1".to_string()).is_ok());
        assert!(version_validator.validate(&"1.21".to_string()).is_ok());
        assert!(version_validator.validate(&"invalid".to_string()).is_err());
        assert!(version_validator.validate(&"0.20.1".to_string()).is_err());
    }
    
    #[test]
    fn test_port_validation() {
        let port_validator = PortValidation;
        
        assert!(port_validator.validate(&25565u16).is_ok());
        assert!(port_validator.validate(&1023u16).is_err());
        assert!(port_validator.validate(&65536u16).is_err());
    }
    
    #[test]
    fn test_server_name_validation() {
        let validator = ValidationSchemas::server_name();
        
        assert!(validator.validate(&"my-server".to_string()).is_ok());
        assert!(validator.validate(&"server123".to_string()).is_ok());
        assert!(validator.validate(&"my server".to_string()).is_err()); // Contains space
        assert!(validator.validate(&"ab".to_string()).is_err()); // Too short
        assert!(validator.validate(&"a".repeat(51)).is_err()); // Too long
    }
}
