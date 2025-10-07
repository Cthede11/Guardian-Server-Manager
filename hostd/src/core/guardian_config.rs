use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use anyhow::{Result, Context};

/// Centralized configuration for Guardian Server Manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianConfig {
    // API Keys
    pub curseforge_api_key: Option<String>,
    pub modrinth_api_key: Option<String>,
    
    // Server Configuration
    pub guardian_port: u16,
    pub guardian_host: String,
    
    // Database Configuration
    pub database_url: String,
    
    // Logging Configuration
    pub rust_log: String,
    pub log_level: String,
    
    // GPU Configuration
    pub gpu_enabled: bool,
    pub gpu_worker_path: PathBuf,
    
    // Java Agent Configuration
    pub java_agent_enabled: bool,
    pub java_agent_path: PathBuf,
    
    // Paths
    pub data_dir: PathBuf,
    pub servers_dir: PathBuf,
    pub backups_dir: PathBuf,
}

impl Default for GuardianConfig {
    fn default() -> Self {
        Self {
            curseforge_api_key: None,
            modrinth_api_key: None,
            guardian_port: 52100,
            guardian_host: "127.0.0.1".to_string(),
            database_url: "sqlite:guardian.db".to_string(),
            rust_log: "info".to_string(),
            log_level: "info".to_string(),
            gpu_enabled: false, // Off by default for safety
            gpu_worker_path: PathBuf::from("./gpu-worker.exe"),
            java_agent_enabled: false,
            java_agent_path: PathBuf::from("./guardian-agent.jar"),
            data_dir: PathBuf::from("data"),
            servers_dir: PathBuf::from("data/servers"),
            backups_dir: PathBuf::from("data/backups"),
        }
    }
}

impl GuardianConfig {
    /// Load configuration from environment variables and .env file
    pub fn load() -> Result<Self> {
        // Load .env file if it exists
        if dotenv::dotenv().is_ok() {
            tracing::info!("Loaded .env file");
        }
        
        let mut config = Self::default();
        
        // Load from environment variables
        if let Ok(api_key) = env::var("CURSEFORGE_API_KEY") {
            config.curseforge_api_key = Some(api_key);
        }
        
        if let Ok(api_key) = env::var("MODRINTH_API_KEY") {
            config.modrinth_api_key = Some(api_key);
        }
        
        if let Ok(port) = env::var("GUARDIAN_PORT") {
            config.guardian_port = port.parse()
                .context("Invalid GUARDIAN_PORT value")?;
        }
        
        if let Ok(host) = env::var("GUARDIAN_HOST") {
            config.guardian_host = host;
        }
        
        if let Ok(db_url) = env::var("DATABASE_URL") {
            config.database_url = db_url;
        }
        
        if let Ok(rust_log) = env::var("RUST_LOG") {
            config.rust_log = rust_log;
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            config.log_level = log_level;
        }
        
        if let Ok(gpu_enabled) = env::var("GPU_ENABLED") {
            config.gpu_enabled = gpu_enabled.parse()
                .unwrap_or(true);
        }
        
        if let Ok(gpu_path) = env::var("GPU_WORKER_PATH") {
            config.gpu_worker_path = PathBuf::from(gpu_path);
        }
        
        if let Ok(java_enabled) = env::var("JAVA_AGENT_ENABLED") {
            config.java_agent_enabled = java_enabled.parse()
                .unwrap_or(false);
        }
        
        if let Ok(java_path) = env::var("JAVA_AGENT_PATH") {
            config.java_agent_path = PathBuf::from(java_path);
        }
        
        // Ensure directories exist
        std::fs::create_dir_all(&config.data_dir)
            .context("Failed to create data directory")?;
        std::fs::create_dir_all(&config.servers_dir)
            .context("Failed to create servers directory")?;
        std::fs::create_dir_all(&config.backups_dir)
            .context("Failed to create backups directory")?;
        
        Ok(config)
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Check if required API keys are present when needed
        if self.curseforge_api_key.is_none() {
            tracing::warn!("CURSEFORGE_API_KEY not set - CurseForge integration will be disabled");
        }
        
        if self.modrinth_api_key.is_none() {
            tracing::warn!("MODRINTH_API_KEY not set - Modrinth integration will be disabled");
        }
        
        // Validate paths
        if self.gpu_enabled && !self.gpu_worker_path.exists() {
            tracing::warn!("GPU worker not found at {:?} - GPU features will be disabled", self.gpu_worker_path);
        }
        
        if self.java_agent_enabled && !self.java_agent_path.exists() {
            tracing::warn!("Java agent not found at {:?} - Java agent features will be disabled", self.java_agent_path);
        }
        
        Ok(())
    }
    
    /// Get the server address
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.guardian_host, self.guardian_port)
    }
    
    /// Check if API integration is available
    pub fn has_curseforge(&self) -> bool {
        self.curseforge_api_key.is_some()
    }
    
    pub fn has_modrinth(&self) -> bool {
        self.modrinth_api_key.is_some()
    }
    
    /// Check if GPU features are available
    pub fn gpu_available(&self) -> bool {
        self.gpu_enabled && self.gpu_worker_path.exists()
    }
    
    /// Check if Java agent is available
    pub fn java_agent_available(&self) -> bool {
        self.java_agent_enabled && self.java_agent_path.exists()
    }
}
