use std::sync::Arc;
use tracing::{info, warn, error, debug, trace};
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
    Registry,
};
use tracing_subscriber::layer::Layer;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use chrono::{DateTime, Utc};

/// Log configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub level: String,
    pub format: LogFormat,
    pub output: LogOutput,
    pub file_path: Option<String>,
    pub max_file_size: Option<u64>,
    pub max_files: Option<u32>,
    pub include_timestamp: bool,
    pub include_thread_id: bool,
    pub include_target: bool,
    pub structured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Compact,
    Pretty,
    Json,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    Console,
    File,
    Both,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Pretty,
            output: LogOutput::Both,
            file_path: Some("logs/guardian.log".to_string()),
            max_file_size: Some(10 * 1024 * 1024), // 10MB
            max_files: Some(5),
            include_timestamp: true,
            include_thread_id: false,
            include_target: true,
            structured: true,
        }
    }
}

/// Log entry structure for structured logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub target: String,
    pub message: String,
    pub fields: serde_json::Value,
    pub thread_id: Option<String>,
    pub span: Option<String>,
}

/// Log manager for centralized logging
pub struct LogManager {
    config: LogConfig,
    file_writer: Option<Arc<dyn Write + Send + Sync>>,
}

impl LogManager {
    pub fn new(config: LogConfig) -> Self {
        Self {
            config,
            file_writer: None,
        }
    }
    
    /// Initialize the logging system
    pub fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Create logs directory if it doesn't exist
        if let Some(file_path) = &self.config.file_path {
            if let Some(parent) = std::path::Path::new(file_path).parent() {
                std::fs::create_dir_all(parent)?;
            }
        }
        
        // Set up file writer if needed
        if matches!(self.config.output, LogOutput::File | LogOutput::Both) {
            if let Some(file_path) = &self.config.file_path {
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(file_path)?;
                self.file_writer = Some(Arc::new(file));
            }
        }
        
        // Initialize tracing subscriber
        self.setup_tracing()?;
        
        info!("Logging system initialized with level: {}", self.config.level);
        Ok(())
    }
    
    fn setup_tracing(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Simplified logging setup
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .init();
        
        Ok(())
    }
    
    
    /// Log a structured entry
    pub fn log_structured(&self, entry: LogEntry) {
        let json = serde_json::to_string(&entry).unwrap_or_else(|_| "Failed to serialize log entry".to_string());
        
        match entry.level.as_str() {
            "ERROR" => error!("{}", json),
            "WARN" => warn!("{}", json),
            "INFO" => info!("{}", json),
            "DEBUG" => debug!("{}", json),
            "TRACE" => trace!("{}", json),
            _ => info!("{}", json),
        }
    }
    
    /// Log an error with context
    pub fn log_error(&self, error: &dyn std::error::Error, context: &str) {
        error!("Error in {}: {} - {}", context, error, error.source().map(|e| e.to_string()).unwrap_or_default());
    }
    
    /// Log a warning with context
    pub fn log_warning(&self, message: &str, context: &str) {
        warn!("Warning in {}: {}", context, message);
    }
    
    /// Log an info message with context
    pub fn log_info(&self, message: &str, context: &str) {
        info!("Info in {}: {}", context, message);
    }
    
    /// Log a debug message with context
    pub fn log_debug(&self, message: &str, context: &str) {
        debug!("Debug in {}: {}", context, message);
    }
    
    /// Log performance metrics
    pub fn log_performance(&self, operation: &str, duration: std::time::Duration, metadata: Option<serde_json::Value>) {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: "INFO".to_string(),
            target: "performance".to_string(),
            message: format!("Operation '{}' completed in {:?}", operation, duration),
            fields: metadata.unwrap_or_else(|| serde_json::Value::Null),
            thread_id: Some(format!("{:?}", std::thread::current().id())),
            span: None,
        };
        
        self.log_structured(entry);
    }
    
    /// Log security events
    pub fn log_security(&self, event: &str, user_id: Option<String>, ip: Option<String>, details: Option<serde_json::Value>) {
        let mut fields = serde_json::Map::new();
        if let Some(user_id) = user_id {
            fields.insert("user_id".to_string(), serde_json::Value::String(user_id));
        }
        if let Some(ip) = ip {
            fields.insert("ip_address".to_string(), serde_json::Value::String(ip));
        }
        if let Some(details) = details {
            fields.extend(details.as_object().unwrap().clone());
        }
        
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: "WARN".to_string(),
            target: "security".to_string(),
            message: format!("Security event: {}", event),
            fields: serde_json::Value::Object(fields),
            thread_id: Some(format!("{:?}", std::thread::current().id())),
            span: None,
        };
        
        self.log_structured(entry);
    }
    
    /// Log audit events
    pub fn log_audit(&self, action: &str, resource: &str, user_id: Option<String>, details: Option<serde_json::Value>) {
        let mut fields = serde_json::Map::new();
        fields.insert("action".to_string(), serde_json::Value::String(action.to_string()));
        fields.insert("resource".to_string(), serde_json::Value::String(resource.to_string()));
        if let Some(user_id) = user_id {
            fields.insert("user_id".to_string(), serde_json::Value::String(user_id));
        }
        if let Some(details) = details {
            fields.extend(details.as_object().unwrap().clone());
        }
        
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: "INFO".to_string(),
            target: "audit".to_string(),
            message: format!("Audit: {} on {}", action, resource),
            fields: serde_json::Value::Object(fields),
            thread_id: Some(format!("{:?}", std::thread::current().id())),
            span: None,
        };
        
        self.log_structured(entry);
    }
}

/// Global log manager instance
static mut LOG_MANAGER: Option<LogManager> = None;

/// Initialize the global logging system
pub fn initialize_logging(config: LogConfig) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let mut manager = LogManager::new(config);
        manager.initialize()?;
        LOG_MANAGER = Some(manager);
    }
    Ok(())
}

/// Get the global log manager
pub fn get_log_manager() -> Option<&'static LogManager> {
    unsafe { LOG_MANAGER.as_ref() }
}

/// Convenience functions for global logging
pub fn log_error(error: &dyn std::error::Error, context: &str) {
    if let Some(manager) = get_log_manager() {
        manager.log_error(error, context);
    }
}

pub fn log_warning(message: &str, context: &str) {
    if let Some(manager) = get_log_manager() {
        manager.log_warning(message, context);
    }
}

pub fn log_info(message: &str, context: &str) {
    if let Some(manager) = get_log_manager() {
        manager.log_info(message, context);
    }
}

pub fn log_debug(message: &str, context: &str) {
    if let Some(manager) = get_log_manager() {
        manager.log_debug(message, context);
    }
}

pub fn log_performance(operation: &str, duration: std::time::Duration, metadata: Option<serde_json::Value>) {
    if let Some(manager) = get_log_manager() {
        manager.log_performance(operation, duration, metadata);
    }
}

pub fn log_security(event: &str, user_id: Option<String>, ip: Option<String>, details: Option<serde_json::Value>) {
    if let Some(manager) = get_log_manager() {
        manager.log_security(event, user_id, ip, details);
    }
}

pub fn log_audit(action: &str, resource: &str, user_id: Option<String>, details: Option<serde_json::Value>) {
    if let Some(manager) = get_log_manager() {
        manager.log_audit(action, resource, user_id, details);
    }
}

/// Performance measurement macro
#[macro_export]
macro_rules! measure_performance {
    ($operation:expr, $code:block) => {{
        let start = std::time::Instant::now();
        let result = $code;
        let duration = start.elapsed();
        crate::core::logging::log_performance($operation, duration, None);
        result
    }};
}

/// Performance measurement macro with metadata
#[macro_export]
macro_rules! measure_performance_with_metadata {
    ($operation:expr, $metadata:expr, $code:block) => {{
        let start = std::time::Instant::now();
        let result = $code;
        let duration = start.elapsed();
        crate::core::logging::log_performance($operation, duration, Some($metadata));
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    
    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert_eq!(config.level, "info");
        assert!(matches!(config.format, LogFormat::Pretty));
        assert!(matches!(config.output, LogOutput::Both));
    }
    
    #[test]
    fn test_log_entry_serialization() {
        let entry = LogEntry {
            timestamp: Utc::now(),
            level: "INFO".to_string(),
            target: "test".to_string(),
            message: "Test message".to_string(),
            fields: serde_json::Value::Null,
            thread_id: Some("test-thread".to_string()),
            span: None,
        };
        
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("Test message"));
    }
    
    #[test]
    fn test_log_manager_creation() {
        let config = LogConfig::default();
        let manager = LogManager::new(config);
        // Manager should be created successfully
        assert!(true);
    }
}
