use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};
use std::collections::HashMap;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref VERSION_REGEX: Regex = Regex::new(r"^\d+\.\d+(\.\d+)?$").unwrap();
}

/// Server configuration validation
#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct ServerConfigValidation {
    #[validate(length(min = 1, max = 50))]
    pub name: String,
    
    #[validate(regex = "VERSION_REGEX")]
    pub minecraft_version: String,
    
    #[validate(custom = "validate_loader")]
    pub loader: String,
    
    #[validate(range(min = 1, max = 65535))]
    pub port: u16,
    
    #[validate(range(min = 1, max = 1000))]
    pub max_players: u32,
    
    #[validate(range(min = 512, max = 32768))]
    pub memory: u32,
    
    #[validate(custom = "validate_world_name")]
    pub world_name: String,
    
    #[validate(custom = "validate_difficulty")]
    pub difficulty: String,
    
    #[validate(custom = "validate_gamemode")]
    pub gamemode: String,
}

/// Custom validation functions
fn validate_loader(loader: &str) -> Result<(), ValidationError> {
    match loader {
        "vanilla" | "fabric" | "forge" | "quilt" | "neoforge" => Ok(()),
        _ => Err(ValidationError::new("invalid_loader")),
    }
}

fn validate_world_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() || name.len() > 50 {
        return Err(ValidationError::new("invalid_world_name"));
    }
    
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(ValidationError::new("invalid_world_name"));
    }
    
    Ok(())
}

fn validate_difficulty(difficulty: &str) -> Result<(), ValidationError> {
    match difficulty {
        "peaceful" | "easy" | "normal" | "hard" => Ok(()),
        _ => Err(ValidationError::new("invalid_difficulty")),
    }
}

fn validate_gamemode(gamemode: &str) -> Result<(), ValidationError> {
    match gamemode {
        "survival" | "creative" | "adventure" | "spectator" => Ok(()),
        _ => Err(ValidationError::new("invalid_gamemode")),
    }
}

/// Input validation utilities
pub struct InputValidator;

impl InputValidator {
    pub fn validate_server_name(name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("Server name cannot be empty".to_string());
        }
        
        if name.len() > 50 {
            return Err("Server name must be 50 characters or less".to_string());
        }
        
        if !name.chars().all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_') {
            return Err("Server name contains invalid characters".to_string());
        }
        
        Ok(())
    }
    
    pub fn validate_minecraft_version(version: &str) -> Result<(), String> {
        let version_regex = regex::Regex::new(r"^\d+\.\d+(\.\d+)?$").unwrap();
        
        if !version_regex.is_match(version) {
            return Err("Invalid Minecraft version format".to_string());
        }
        
        Ok(())
    }
    
    pub fn validate_port(port: u16) -> Result<(), String> {
        if port < 1024 || port > 65535 {
            return Err("Port must be between 1024 and 65535".to_string());
        }
        
        Ok(())
    }
    
    pub fn validate_memory(memory: u32) -> Result<(), String> {
        if memory < 512 || memory > 32768 {
            return Err("Memory must be between 512MB and 32GB".to_string());
        }
        
        Ok(())
    }
    
    pub fn validate_max_players(max_players: u32) -> Result<(), String> {
        if max_players < 1 || max_players > 1000 {
            return Err("Max players must be between 1 and 1000".to_string());
        }
        
        Ok(())
    }
}

/// SQL injection prevention
pub struct SQLInjectionPrevention;

impl SQLInjectionPrevention {
    pub fn is_safe_query(query: &str) -> bool {
        let dangerous_patterns = [
            "DROP", "DELETE", "INSERT", "UPDATE", "ALTER", "CREATE", "TRUNCATE",
            "EXEC", "EXECUTE", "UNION", "SELECT", "SCRIPT", "IF", "WHILE",
            "DECLARE", "CAST", "CONVERT", "WAITFOR", "DELAY", "BULK",
            "OPENROWSET", "OPENDATASOURCE", "SP_", "XP_", "fn_",
        ];
        
        let query_upper = query.to_uppercase();
        
        for pattern in &dangerous_patterns {
            if query_upper.contains(pattern) {
                return false;
            }
        }
        
        true
    }
    
    pub fn sanitize_query(query: &str) -> String {
        query
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || ".,!?@#$%^&*()_+-=[]{}|;':\",./<>?~`".contains(*c))
            .collect()
    }
}

/// XSS prevention
pub struct XSSPrevention;

impl XSSPrevention {
    pub fn sanitize_html(input: &str) -> String {
        input
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
            .replace('&', "&amp;")
    }
    
    pub fn sanitize_js(input: &str) -> String {
        input
            .replace('\\', "\\\\")
            .replace('\'', "\\'")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }
}

/// Path traversal prevention
pub struct PathTraversalPrevention;

impl PathTraversalPrevention {
    pub fn is_safe_path(path: &str) -> bool {
        !path.contains("..") && !path.contains("~") && !path.starts_with('/')
    }
    
    pub fn sanitize_path(path: &str) -> String {
        path
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '/' || *c == '.' || *c == '_' || *c == '-')
            .collect()
    }
}

/// Command injection prevention
pub struct CommandInjectionPrevention;

impl CommandInjectionPrevention {
    pub fn is_safe_command(command: &str) -> bool {
        let dangerous_chars = [';', '&', '|', '`', '$', '(', ')', '<', '>', '\n', '\r'];
        
        !command.chars().any(|c| dangerous_chars.contains(&c))
    }
    
    pub fn sanitize_command(command: &str) -> String {
        command
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || ".,!?@#$%^&*()_+-=[]{}|;':\",./<>?~`".contains(*c))
            .collect()
    }
}
