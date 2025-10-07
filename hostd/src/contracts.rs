//! Canonical contracts and data structures used across the Guardian Server Manager
//! This file serves as the single source of truth for all shared types

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Standard API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
            timestamp: Utc::now(),
        }
    }
}

/// Canonical AppState structure
#[derive(Clone)]
pub struct AppState {
    // Core services
    pub database: Arc<crate::database::DatabaseManager>,
    pub websocket_manager: Arc<crate::websocket_manager::WebSocketManager>,
    pub minecraft_manager: crate::minecraft::MinecraftManager,
    pub mod_manager: crate::mod_manager::ModManager,
    
    // Resource management
    pub resource_monitor: Arc<crate::core::resource_monitor::ResourceMonitor>,
    pub crash_watchdog: Arc<crate::core::crash_watchdog::CrashWatchdog>,
    pub process_manager: Arc<crate::core::process_manager::ProcessManager>,
    
    // GPU and performance
    pub gpu_manager: Arc<tokio::sync::Mutex<crate::gpu_manager::GpuManager>>,
    pub performance_telemetry: Arc<crate::performance_telemetry::PerformanceTelemetry>,
    
    // Security and storage
    pub secret_storage: Arc<crate::security::secret_storage::SecretStorage>,
    
    // Rate limiting
    pub rate_limiter: Arc<crate::security::rate_limiting::RateLimiter>,
    
    // Test harness
    pub test_harness: Arc<crate::core::test_harness::TestHarness>,
    
    // SSE
    pub sse_sender: Option<tokio::sync::broadcast::Sender<serde_json::Value>>,
}

/// Server configuration (runtime-facing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub id: String,
    pub name: String,
    pub minecraft_version: String,
    pub loader: String,
    pub loader_version: String,
    pub server_directory: String,
    pub jar_path: Option<String>,
    pub max_players: Option<u32>,
    pub memory_mb: Option<u32>,
    pub auto_start: Option<bool>,
    pub auto_restart: Option<bool>,
    pub world_name: Option<String>,
    pub difficulty: Option<String>,
    pub gamemode: Option<String>,
    pub pvp: Option<bool>,
    pub allow_flight: Option<bool>,
    pub allow_nether: Option<bool>,
    pub spawn_protection: Option<u32>,
    pub view_distance: Option<u32>,
    pub simulation_distance: Option<u32>,
    pub hardcore: Option<bool>,
    pub online_mode: Option<bool>,
    pub white_list: Option<bool>,
    pub enable_command_block: Option<bool>,
    pub motd: Option<String>,
    pub player_idle_timeout: Option<u32>,
    pub max_world_size: Option<u32>,
    pub network_compression_threshold: Option<u32>,
    pub max_tick_time: Option<u32>,
    pub use_native_transport: Option<bool>,
    pub enable_jmx_monitoring: Option<bool>,
    pub enable_status: Option<bool>,
    pub sync_chunk_writes: Option<bool>,
    pub enable_query: Option<bool>,
    pub query_port: Option<u16>,
    pub rcon_enabled: Option<bool>,
    pub rcon_port: Option<u16>,
    pub rcon_password: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Server record (database-facing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerRecord {
    pub id: String,
    pub name: String,
    pub minecraft_version: String,
    pub loader: String,
    pub loader_version: String,
    pub server_directory: String,
    pub jar_path: Option<String>,
    pub max_players: Option<u32>,
    pub memory_mb: Option<u32>,
    pub auto_start: Option<bool>,
    pub auto_restart: Option<bool>,
    pub world_name: Option<String>,
    pub difficulty: Option<String>,
    pub gamemode: Option<String>,
    pub pvp: Option<bool>,
    pub allow_flight: Option<bool>,
    pub allow_nether: Option<bool>,
    pub spawn_protection: Option<u32>,
    pub view_distance: Option<u32>,
    pub simulation_distance: Option<u32>,
    pub hardcore: Option<bool>,
    pub online_mode: Option<bool>,
    pub white_list: Option<bool>,
    pub enable_command_block: Option<bool>,
    pub motd: Option<String>,
    pub player_idle_timeout: Option<u32>,
    pub max_world_size: Option<u32>,
    pub network_compression_threshold: Option<u32>,
    pub max_tick_time: Option<u32>,
    pub use_native_transport: Option<bool>,
    pub enable_jmx_monitoring: Option<bool>,
    pub enable_status: Option<bool>,
    pub sync_chunk_writes: Option<bool>,
    pub enable_query: Option<bool>,
    pub query_port: Option<u16>,
    pub rcon_enabled: Option<bool>,
    pub rcon_port: Option<u16>,
    pub rcon_password: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Mod provider enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModProvider {
    Modrinth,
    CurseForge,
    Custom,
}

/// Mod identifier structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModIdentifier {
    pub provider: ModProvider,
    pub project_id: Option<String>,
    pub slug: Option<String>,
    pub file_id: Option<String>,
    pub version_id: Option<String>,
    pub version: String,
    pub sha1: Option<String>,
    pub loader: String,
    pub mc_version: String,
}

/// Mod metadata (canonical)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModMetadata {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub authors: Vec<String>,
    pub version: String,
    pub minecraft_version: String,
    pub loader: String,
    pub project_url: Option<String>,
    pub source_url: Option<String>,
    pub issues_url: Option<String>,
    pub wiki_url: Option<String>,
    pub discord_url: Option<String>,
    pub donation_url: Option<String>,
    pub license: Option<String>,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Mod version (canonical)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModVersion {
    pub id: String,
    pub mod_metadata_id: String,
    pub version: String,
    pub minecraft_version: String,
    pub loader: String,
    pub filename: String,
    pub file_size: u64,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub sha512: Option<String>,
    pub download_url: String,
    pub release_type: String, // 'release', 'beta', 'alpha'
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Mod dependency (canonical)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependency {
    pub id: String,
    pub mod_metadata_id: String,
    pub dependency_mod_id: String,
    pub version_range: String,
    pub required: bool,
    pub side: String, // 'client', 'server', 'both'
    pub created_at: DateTime<Utc>,
}

/// Installed mod with metadata (canonical)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledModWithMetadata {
    pub id: String,
    pub server_id: String,
    pub mod_metadata: ModMetadata,
    pub mod_version: ModVersion,
    pub file_path: String,
    pub enabled: bool,
    pub installed_at: DateTime<Utc>,
}

/// Health status payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub details: Option<String>,
    pub services: Option<serde_json::Value>,
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebSocketMessage {
    JobStarted { job_id: String, server_id: String, job_type: String },
    JobProgress { job_id: String, server_id: String, progress: f32, message: String },
    JobCompleted { job_id: String, server_id: String, result: serde_json::Value },
    JobFailed { job_id: String, server_id: String, error: String },
    ServerStatus { server_id: String, status: String, details: Option<serde_json::Value> },
    ServerMetrics { server_id: String, metrics: serde_json::Value },
}

/// Progress event payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub job_id: String,
    pub server_id: String,
    pub progress: f32,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

/// Modpack state structure
#[derive(Clone)]
pub struct ModpackState {
    pub database: Arc<crate::database::DatabaseManager>,
    pub version_resolver: crate::version_resolver::VersionResolver,
}

impl ModpackState {
    pub fn new(database: Arc<crate::database::DatabaseManager>, version_resolver: crate::version_resolver::VersionResolver) -> Self {
        Self {
            database,
            version_resolver,
        }
    }
}

/// Conversion implementations for backward compatibility

impl From<ServerRecord> for ServerConfig {
    fn from(record: ServerRecord) -> Self {
        Self {
            id: record.id,
            name: record.name,
            minecraft_version: record.minecraft_version,
            loader: record.loader,
            loader_version: record.loader_version,
            server_directory: record.server_directory,
            jar_path: record.jar_path,
            max_players: record.max_players,
            memory_mb: record.memory_mb,
            auto_start: record.auto_start,
            auto_restart: record.auto_restart,
            world_name: record.world_name,
            difficulty: record.difficulty,
            gamemode: record.gamemode,
            pvp: record.pvp,
            allow_flight: record.allow_flight,
            allow_nether: record.allow_nether,
            spawn_protection: record.spawn_protection,
            view_distance: record.view_distance,
            simulation_distance: record.simulation_distance,
            hardcore: record.hardcore,
            online_mode: record.online_mode,
            white_list: record.white_list,
            enable_command_block: record.enable_command_block,
            motd: record.motd,
            player_idle_timeout: record.player_idle_timeout,
            max_world_size: record.max_world_size,
            network_compression_threshold: record.network_compression_threshold,
            max_tick_time: record.max_tick_time,
            use_native_transport: record.use_native_transport,
            enable_jmx_monitoring: record.enable_jmx_monitoring,
            enable_status: record.enable_status,
            sync_chunk_writes: record.sync_chunk_writes,
            enable_query: record.enable_query,
            query_port: record.query_port,
            rcon_enabled: record.rcon_enabled,
            rcon_port: record.rcon_port,
            rcon_password: record.rcon_password,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

impl From<ServerConfig> for ServerRecord {
    fn from(config: ServerConfig) -> Self {
        Self {
            id: config.id,
            name: config.name,
            minecraft_version: config.minecraft_version,
            loader: config.loader,
            loader_version: config.loader_version,
            server_directory: config.server_directory,
            jar_path: config.jar_path,
            max_players: config.max_players,
            memory_mb: config.memory_mb,
            auto_start: config.auto_start,
            auto_restart: config.auto_restart,
            world_name: config.world_name,
            difficulty: config.difficulty,
            gamemode: config.gamemode,
            pvp: config.pvp,
            allow_flight: config.allow_flight,
            allow_nether: config.allow_nether,
            spawn_protection: config.spawn_protection,
            view_distance: config.view_distance,
            simulation_distance: config.simulation_distance,
            hardcore: config.hardcore,
            online_mode: config.online_mode,
            white_list: config.white_list,
            enable_command_block: config.enable_command_block,
            motd: config.motd,
            player_idle_timeout: config.player_idle_timeout,
            max_world_size: config.max_world_size,
            network_compression_threshold: config.network_compression_threshold,
            max_tick_time: config.max_tick_time,
            use_native_transport: config.use_native_transport,
            enable_jmx_monitoring: config.enable_jmx_monitoring,
            enable_status: config.enable_status,
            sync_chunk_writes: config.sync_chunk_writes,
            enable_query: config.enable_query,
            query_port: config.query_port,
            rcon_enabled: config.rcon_enabled,
            rcon_port: config.rcon_port,
            rcon_password: config.rcon_password,
            created_at: config.created_at,
            updated_at: config.updated_at,
        }
    }
}
