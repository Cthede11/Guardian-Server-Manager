use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::error_handler::{AppError, Result};

/// Validation rule for a field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    Required,
    MinLength(usize),
    MaxLength(usize),
    MinValue(i64),
    MaxValue(i64),
    MinValueFloat(f64),
    MaxValueFloat(f64),
    Pattern(String),
    Email,
    Url,
    Port,
    MemorySize,
    JavaVersion,
    MinecraftVersion,
    ServerName,
    FilePath,
    NoPathTraversal,
    NoCommandInjection,
    ValidEnum(Vec<String>),
}

/// Field validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValidation {
    pub field_name: String,
    pub rules: Vec<ValidationRule>,
    pub required: bool,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<FieldError>,
}

/// Field validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
    pub code: String,
    pub value: Option<serde_json::Value>,
}

impl FieldError {
    pub fn new(field: String, message: String, code: String, value: Option<serde_json::Value>) -> Self {
        Self {
            field,
            message,
            code,
            value,
        }
    }
}

/// Input validator
#[derive(Debug)]
pub struct InputValidator {
    validations: HashMap<String, Vec<FieldValidation>>,
}

impl InputValidator {
    pub fn new() -> Self {
        Self {
            validations: HashMap::new(),
        }
    }

    /// Add validation rules for an endpoint
    pub fn add_endpoint_validation(&mut self, endpoint: &str, validations: Vec<FieldValidation>) {
        self.validations.insert(endpoint.to_string(), validations);
    }

    /// Validate input for an endpoint
    pub fn validate(&self, endpoint: &str, data: &serde_json::Value) -> ValidationResult {
        let mut errors = Vec::new();

        if let Some(field_validations) = self.validations.get(endpoint) {
            for field_validation in field_validations {
                if let Some(field_value) = data.get(&field_validation.field_name) {
                    for rule in &field_validation.rules {
                        if let Err(error) = self.validate_rule(&field_validation.field_name, field_value, rule) {
                            errors.push(FieldError::new(
                                field_validation.field_name.clone(),
                                error.user_message(),
                                error.error_code(),
                                None,
                ));
            }
        }
                } else if field_validation.required {
                    errors.push(FieldError::new(
                        field_validation.field_name.clone(),
                        format!("Field '{}' is required", field_validation.field_name),
                        "REQUIRED".to_string(),
                        None,
                    ));
                }
            }
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
        }
    }

    fn validate_rule(&self, field_name: &str, value: &serde_json::Value, rule: &ValidationRule) -> Result<()> {
        match rule {
            ValidationRule::Required => {
                if value.is_null() {
                    return Err(AppError::ValidationError {
                        message: format!("Field '{}' is required", field_name),
                        field: field_name.to_string(),
                        value: "null".to_string(),
                        constraint: "required".to_string(),
                    });
                }
            }
            ValidationRule::MinLength(min) => {
                if let Some(s) = value.as_str() {
                    if s.len() < *min {
                        return Err(AppError::ValidationError {
                            message: format!("Field '{}' must be at least {} characters long", field_name, min),
                            field: field_name.to_string(),
                            value: s.to_string(),
                            constraint: format!("min_length:{}", min),
                        });
                    }
                }
            }
            ValidationRule::MaxLength(max) => {
                if let Some(s) = value.as_str() {
                    if s.len() > *max {
                        return Err(AppError::ValidationError {
                            message: format!("Field '{}' must be at most {} characters long", field_name, max),
                            field: field_name.to_string(),
                            value: s.to_string(),
                            constraint: format!("max_length:{}", max),
                        });
                    }
                }
            }
            ValidationRule::MinValue(min) => {
                if let Some(n) = value.as_i64() {
                    if n < *min {
                        return Err(AppError::ValidationError {
                            message: format!("Field '{}' must be at least {}", field_name, min),
                            field: field_name.to_string(),
                            value: n.to_string(),
                            constraint: format!("min_value:{}", min),
                        });
                    }
                }
            }
            ValidationRule::MaxValue(max) => {
                if let Some(n) = value.as_i64() {
                    if n > *max {
                        return Err(AppError::ValidationError {
                            message: format!("Field '{}' must be at most {}", field_name, max),
                            field: field_name.to_string(),
                            value: n.to_string(),
                            constraint: format!("max_value:{}", max),
                        });
                    }
                }
            }
            ValidationRule::MinValueFloat(min) => {
                if let Some(n) = value.as_f64() {
                    if n < *min {
                        return Err(AppError::ValidationError {
                            message: format!("Field '{}' must be at least {}", field_name, min),
                            field: field_name.to_string(),
                            value: n.to_string(),
                            constraint: format!("min_value:{}", min),
                        });
                    }
                }
            }
            ValidationRule::MaxValueFloat(max) => {
                if let Some(n) = value.as_f64() {
                    if n > *max {
                        return Err(AppError::ValidationError {
                            message: format!("Field '{}' must be at most {}", field_name, max),
                            field: field_name.to_string(),
                            value: n.to_string(),
                            constraint: format!("max_value:{}", max),
                        });
                    }
                }
            }
            ValidationRule::Pattern(pattern) => {
                if let Some(s) = value.as_str() {
                    let regex = regex::Regex::new(pattern).unwrap();
                    if !regex.is_match(s) {
                        return Err(AppError::ValidationError {
                            message: format!("Field '{}' does not match required pattern", field_name),
                            field: field_name.to_string(),
                            value: s.to_string(),
                            constraint: format!("pattern:{}", pattern),
                        });
                    }
                }
            }
            ValidationRule::Email => {
                if let Some(s) = value.as_str() {
                    if !self.is_valid_email(s) {
                        return Err(AppError::ValidationError {
                            message: format!("Field '{}' must be a valid email address", field_name),
                            field: field_name.to_string(),
                            value: s.to_string(),
                            constraint: "email".to_string(),
                        });
                    }
                }
            }
            ValidationRule::Url => {
                if let Some(s) = value.as_str() {
                    if !self.is_valid_url(s) {
                        return Err(AppError::ValidationError {
                            message: format!("Field '{}' must be a valid URL", field_name),
                            field: field_name.to_string(),
                            value: s.to_string(),
                            constraint: "url".to_string(),
                        });
                    }
                }
            }
            ValidationRule::Port => {
                if let Some(n) = value.as_i64() {
                    if n < 1 || n > 65535 {
                        return Err(AppError::ValidationError {
                            message: format!("Field '{}' must be a valid port number (1-65535)", field_name),
                            field: field_name.to_string(),
                            value: n.to_string(),
                            constraint: "port".to_string(),
                        });
                    }
                }
            }
            ValidationRule::MemorySize => {
                if let Some(n) = value.as_i64() {
                    if n < 512 || n > 32768 {
                        return Err(AppError::ValidationError {
                            message: format!("Field '{}' must be between 512MB and 32GB", field_name),
                            field: field_name.to_string(),
                            value: n.to_string(),
                            constraint: "memory_size".to_string(),
                        });
                    }
                }
            }
            ValidationRule::JavaVersion => {
                if let Some(s) = value.as_str() {
                    if !self.is_valid_java_version(s) {
                        return Err(AppError::ValidationError {
                            message: format!("Field '{}' must be a valid Java version", field_name),
                            field: field_name.to_string(),
                            value: s.to_string(),
                            constraint: "java_version".to_string(),
                        });
                    }
                }
            }
            ValidationRule::MinecraftVersion => {
                if let Some(s) = value.as_str() {
                    if !self.is_valid_minecraft_version(s) {
                        return Err(AppError::ValidationError {
                            message: format!("Field '{}' must be a valid Minecraft version", field_name),
                            field: field_name.to_string(),
                            value: s.to_string(),
                            constraint: "minecraft_version".to_string(),
                        });
                    }
                }
            }
            ValidationRule::ServerName => {
                if let Some(s) = value.as_str() {
                    if !self.is_valid_server_name(s) {
                        return Err(AppError::ValidationError {
                            message: format!("Field '{}' must be a valid server name (alphanumeric, spaces, hyphens, underscores only)", field_name),
                            field: field_name.to_string(),
                            value: s.to_string(),
                            constraint: "server_name".to_string(),
                        });
                    }
                }
            }
            ValidationRule::FilePath => {
                if let Some(s) = value.as_str() {
                    if !self.is_valid_file_path(s) {
                        return Err(AppError::ValidationError {
                            message: format!("Field '{}' must be a valid file path", field_name),
                            field: field_name.to_string(),
                            value: s.to_string(),
                            constraint: "file_path".to_string(),
                        });
                    }
                }
            }
            ValidationRule::NoPathTraversal => {
                if let Some(s) = value.as_str() {
                    if self.contains_path_traversal(s) {
                        return Err(AppError::ValidationError {
                            message: format!("Field '{}' contains path traversal characters", field_name),
                            field: field_name.to_string(),
                            value: s.to_string(),
                            constraint: "no_path_traversal".to_string(),
                        });
                    }
                }
            }
            ValidationRule::NoCommandInjection => {
                if let Some(s) = value.as_str() {
                    if self.contains_command_injection(s) {
                        return Err(AppError::ValidationError {
                            message: format!("Field '{}' contains potentially dangerous command characters", field_name),
                            field: field_name.to_string(),
                            value: s.to_string(),
                            constraint: "no_command_injection".to_string(),
                        });
                    }
                }
            }
            ValidationRule::ValidEnum(values) => {
                if let Some(s) = value.as_str() {
                    if !values.contains(&s.to_string()) {
                        return Err(AppError::ValidationError {
                            message: format!("Field '{}' must be one of: {}", field_name, values.join(", ")),
                            field: field_name.to_string(),
                            value: s.to_string(),
                            constraint: "valid_enum".to_string(),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn is_valid_email(&self, email: &str) -> bool {
        let email_regex = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        email_regex.is_match(email)
    }

    fn is_valid_url(&self, url: &str) -> bool {
        let url_regex = regex::Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
        url_regex.is_match(url)
    }

    fn is_valid_java_version(&self, version: &str) -> bool {
        let version_regex = regex::Regex::new(r"^\d+(\.\d+)*$").unwrap();
        version_regex.is_match(version)
    }

    fn is_valid_minecraft_version(&self, version: &str) -> bool {
        let version_regex = regex::Regex::new(r"^\d+\.\d+(\.\d+)?$").unwrap();
        version_regex.is_match(version)
    }

    fn is_valid_server_name(&self, name: &str) -> bool {
        let name_regex = regex::Regex::new(r"^[a-zA-Z0-9\s\-_]+$").unwrap();
        name_regex.is_match(name) && name.len() >= 1 && name.len() <= 50
    }

    fn is_valid_file_path(&self, path: &str) -> bool {
        !path.contains("..") && !path.starts_with('/') && path.len() > 0 && path.len() < 1000
    }

    fn contains_path_traversal(&self, path: &str) -> bool {
        path.contains("..") || path.contains("~") || path.starts_with('/') || path.starts_with('\\')
    }

    fn contains_command_injection(&self, input: &str) -> bool {
        let dangerous_chars = [';', '&', '|', '`', '$', '(', ')', '<', '>', '\n', '\r'];
        dangerous_chars.iter().any(|&c| input.contains(c))
    }
}

impl Default for InputValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Create default validation rules for common endpoints
pub fn create_default_validations() -> InputValidator {
    let mut validator = InputValidator::new();

    // Server creation validation
    validator.add_endpoint_validation("create_server", vec![
        FieldValidation {
            field_name: "name".to_string(),
            required: true,
            rules: vec![
                ValidationRule::Required,
                ValidationRule::MinLength(1),
                ValidationRule::MaxLength(50),
                ValidationRule::ServerName,
            ],
        },
        FieldValidation {
            field_name: "minecraft_version".to_string(),
            required: true,
            rules: vec![
                ValidationRule::Required,
                ValidationRule::MinecraftVersion,
            ],
        },
        FieldValidation {
            field_name: "port".to_string(),
            required: true,
            rules: vec![
                ValidationRule::Required,
                ValidationRule::Port,
            ],
        },
        FieldValidation {
            field_name: "rcon_port".to_string(),
            required: true,
            rules: vec![
                ValidationRule::Required,
                ValidationRule::Port,
            ],
        },
        FieldValidation {
            field_name: "query_port".to_string(),
            required: true,
            rules: vec![
                ValidationRule::Required,
                ValidationRule::Port,
            ],
        },
        FieldValidation {
            field_name: "memory".to_string(),
            required: true,
            rules: vec![
                ValidationRule::Required,
                ValidationRule::MemorySize,
            ],
        },
        FieldValidation {
            field_name: "max_players".to_string(),
            required: true,
            rules: vec![
                ValidationRule::Required,
                ValidationRule::MinValue(1),
                ValidationRule::MaxValue(1000),
            ],
        },
        FieldValidation {
            field_name: "java_args".to_string(),
            required: false,
            rules: vec![
                ValidationRule::NoCommandInjection,
            ],
        },
        FieldValidation {
            field_name: "server_args".to_string(),
            required: false,
            rules: vec![
                ValidationRule::NoCommandInjection,
            ],
        },
    ]);

    // Server update validation
    validator.add_endpoint_validation("update_server", vec![
        FieldValidation {
            field_name: "name".to_string(),
            required: false,
            rules: vec![
                ValidationRule::MinLength(1),
                ValidationRule::MaxLength(50),
                ValidationRule::ServerName,
            ],
        },
        FieldValidation {
            field_name: "memory".to_string(),
            required: false,
            rules: vec![
                ValidationRule::MemorySize,
            ],
        },
        FieldValidation {
            field_name: "max_players".to_string(),
            required: false,
            rules: vec![
                ValidationRule::MinValue(1),
                ValidationRule::MaxValue(1000),
            ],
        },
    ]);

    // Command execution validation
    validator.add_endpoint_validation("execute_command", vec![
        FieldValidation {
            field_name: "command".to_string(),
            required: true,
            rules: vec![
                ValidationRule::Required,
                ValidationRule::MinLength(1),
                ValidationRule::MaxLength(1000),
                ValidationRule::NoCommandInjection,
            ],
        },
    ]);

    validator
}