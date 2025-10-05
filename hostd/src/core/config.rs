use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub security: SecurityConfig,
    pub monitoring: MonitoringConfig,
    pub minecraft: MinecraftConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
    pub max_connections: usize,
    pub request_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout: u64,
    pub idle_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub token_expiry: u64,
    pub rate_limit_requests: u32,
    pub rate_limit_window: u64,
    pub enable_cors: bool,
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enable_metrics: bool,
    pub metrics_port: u16,
    pub log_level: String,
    pub log_file: PathBuf,
    pub enable_health_checks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftConfig {
    pub server_jar_directory: PathBuf,
    pub world_directory: PathBuf,
    pub mods_directory: PathBuf,
    pub config_directory: PathBuf,
    pub logs_directory: PathBuf,
    pub backups_directory: PathBuf,
    pub java_executable: PathBuf,
    pub default_memory: u32,
    pub default_max_players: u32,
    pub default_port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 52100,
                cors_origins: vec!["http://localhost:3000".to_string(), "http://127.0.0.1:3000".to_string()],
                max_connections: 1000,
                request_timeout: 30,
            },
            database: DatabaseConfig {
                url: "sqlite:guardian.db".to_string(),
                max_connections: 10,
                connection_timeout: 30,
                idle_timeout: 600,
            },
            security: SecurityConfig {
                jwt_secret: "your-secret-key-change-in-production".to_string(),
                token_expiry: 3600,
                rate_limit_requests: 100,
                rate_limit_window: 60,
                enable_cors: true,
                allowed_origins: vec!["*".to_string()],
            },
            monitoring: MonitoringConfig {
                enable_metrics: true,
                metrics_port: 9090,
                log_level: "info".to_string(),
                log_file: PathBuf::from("guardian.log"),
                enable_health_checks: true,
            },
            minecraft: MinecraftConfig {
                server_jar_directory: PathBuf::from("./servers"),
                world_directory: PathBuf::from("./worlds"),
                mods_directory: PathBuf::from("./mods"),
                config_directory: PathBuf::from("./configs"),
                logs_directory: PathBuf::from("./logs"),
                backups_directory: PathBuf::from("./backups"),
                java_executable: PathBuf::from("java"),
                default_memory: 2048,
                default_max_players: 20,
                default_port: 25565,
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        // Try to load from config file first
        if let Ok(config_file) = std::fs::read_to_string("config.toml") {
            return Ok(toml::from_str(&config_file)?);
        }
        
        // Fallback to environment variables
        let mut config = Self::default();
        
        if let Ok(port) = std::env::var("GUARDIAN_PORT") {
            config.server.port = port.parse()?;
        }
        
        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            config.database.url = db_url;
        }
        
        if let Ok(jwt_secret) = std::env::var("JWT_SECRET") {
            config.security.jwt_secret = jwt_secret;
        }
        
        Ok(config)
    }
    
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_str = toml::to_string_pretty(self)?;
        std::fs::write("config.toml", config_str)?;
        Ok(())
    }
    
    pub fn server_addr(&self) -> SocketAddr {
        format!("{}:{}", self.server.host, self.server.port)
            .parse()
            .expect("Invalid server address")
    }
}
