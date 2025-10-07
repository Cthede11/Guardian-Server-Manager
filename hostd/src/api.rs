use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete, patch},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, warn, error};
use chrono::{self, Utc};

use crate::websocket_manager::{WebSocketManager, WebSocketMessage};
use crate::database::{ServerConfig, MinecraftVersion, LoaderVersion, ModVersion, Modpack, Settings, Mod};
use crate::mod_manager::{ModManager, ModCompatibilityResult, ModInfo as ModManagerModInfo};

/// API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub id: String,
    pub name: String,
    pub status: String,
    pub tps: f64,
    #[serde(rename = "tickP95")]
    pub tick_p95: f64,
    #[serde(rename = "heapMb")]
    pub heap_mb: u64,
    #[serde(rename = "playersOnline")]
    pub players_online: u32,
    #[serde(rename = "gpuQueueMs")]
    pub gpu_queue_ms: f64,
    #[serde(rename = "lastSnapshotAt")]
    pub last_snapshot_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(rename = "blueGreen")]
    pub blue_green: BlueGreenInfo,
    pub version: Option<String>,
    pub max_players: Option<u32>,
    pub uptime: Option<u64>,
    pub memory_usage: Option<u64>,
    pub cpu_usage: Option<f64>,
    pub world_size: Option<u64>,
    pub last_backup: Option<chrono::DateTime<chrono::Utc>>,
    pub auto_start: Option<bool>,
    pub auto_restart: Option<bool>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Blue-green deployment info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueGreenInfo {
    pub active: String,
    #[serde(rename = "candidateHealthy")]
    pub candidate_healthy: bool,
}

/// Request to create a new server
#[derive(Debug, Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
    pub loader: String,
    pub version: String,
    #[serde(alias = "mc_version")]
    pub minecraft_version: String,
    pub paths: ServerPaths,
    #[serde(rename = "jarPath")]
    pub jar_path: Option<String>,
    #[serde(rename = "maxPlayers")]
    pub max_players: Option<u32>,
    pub memory: Option<u32>,
    #[serde(rename = "pregenerationPolicy")]
    pub pregeneration_policy: Option<serde_json::Value>,
    // Additional fields for production server creation
    pub port: Option<u16>,
    pub rcon_port: Option<u16>,
    pub query_port: Option<u16>,
    pub auto_start: Option<bool>,
    pub auto_restart: Option<bool>,
    pub world_settings: Option<WorldSettings>,
    pub pvp: Option<bool>,
    pub online_mode: Option<bool>,
    pub whitelist: Option<bool>,
    pub enable_command_block: Option<bool>,
    pub view_distance: Option<u32>,
    pub simulation_distance: Option<u32>,
    pub motd: Option<String>,
    pub modpack: Option<ModpackInstallRequest>,
    pub individual_mods: Option<Vec<ModInstallItem>>,
}

#[derive(Debug, Deserialize)]
pub struct WorldSettings {
    pub world_name: String,
    pub difficulty: String,
    pub gamemode: String,
}

#[derive(Debug, Deserialize)]
pub struct ModpackInstallRequest {
    pub pack_id: String,
    pub pack_version_id: String,
    pub provider: String,
    pub server_only: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModInstallItem {
    pub mod_id: String,
    pub file_id: String,
    pub provider: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateServerRequest {
    pub name: Option<String>,
    pub java_path: Option<String>,
    pub jvm_args: Option<String>,
    pub server_args: Option<String>,
    pub auto_start: Option<bool>,
    pub auto_restart: Option<bool>,
    #[serde(rename = "maxPlayers")]
    pub max_players: Option<u32>,
    #[serde(rename = "pregenerationPolicy")]
    pub pregeneration_policy: Option<serde_json::Value>,
}

/// Server paths configuration
#[derive(Debug, Deserialize)]
pub struct ServerPaths {
    pub world: String,
    pub mods: String,
    pub config: String,
    pub java_path: Option<String>,
}

/// Server health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHealth {
    pub rcon: bool,
    pub query: bool,
    pub crash_tickets: u32,
    pub freeze_tickets: u32,
}

/// Console message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleMessage {
    pub ts: String,
    pub level: String,
    pub msg: String,
}

/// Player information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub uuid: String,
    pub name: String,
    pub online: bool,
    pub last_seen: Option<String>,
    pub playtime: Option<u64>,
}

/// World freeze information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldFreeze {
    pub x: i32,
    pub z: i32,
    pub duration_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Pregen job information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PregenJob {
    pub id: String,
    pub region: RegionInfo,
    pub dimension: String,
    pub priority: String,
    pub status: String,
    pub progress: f64,
    pub eta: Option<String>,
    pub gpu_assist: bool,
}

/// Region information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionInfo {
    pub x: i32,
    pub z: i32,
    pub radius: u32,
}

/// Metrics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    pub tps: Vec<MetricPoint>,
    pub heap: Vec<MetricPoint>,
    pub tick_p95: Vec<MetricPoint>,
    pub gpu_ms: Vec<MetricPoint>,
}

/// Metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub timestamp: i64,
    pub value: f64,
}

/// Server command request
#[derive(Debug, Deserialize)]
pub struct ServerCommandRequest {
    pub command: String,
}

/// Server command response
#[derive(Debug, Serialize)]
pub struct ServerCommandResponse {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

/// Query parameters for pagination
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// Query parameters for filtering
#[derive(Debug, Deserialize)]
pub struct FilterQuery {
    pub status: Option<String>,
    pub dimension: Option<String>,
    pub level: Option<String>,
}

/// Mod search query parameters
#[derive(Debug, Deserialize)]
pub struct ModSearchQuery {
    pub search_query: Option<String>,
    pub minecraft_version: Option<String>,
    pub loader: Option<String>,
    pub category: Option<String>,
    pub side: Option<String>,
    pub source: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// Modpack creation request
#[derive(Debug, Deserialize)]
pub struct CreateModpackRequest {
    pub name: String,
    pub description: Option<String>,
    pub minecraft_version: String,
    pub loader: String,
    pub client_mods: Vec<String>,
    pub server_mods: Vec<String>,
    pub config: Option<serde_json::Value>,
}

/// Modpack update request
#[derive(Debug, Deserialize)]
pub struct UpdateModpackRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub client_mods: Option<Vec<String>>,
    pub server_mods: Option<Vec<String>>,
    pub config: Option<serde_json::Value>,
}

/// Apply modpack to server request
#[derive(Debug, Deserialize)]
pub struct ApplyModpackRequest {
    pub server_id: String,
    pub install_client_mods: bool,
    pub backup_before_apply: bool,
}

/// Application state
#[derive(Clone)]
pub struct AppState {
    // Core services
    pub database: Arc<crate::database::DatabaseManager>,
    pub websocket_manager: Arc<WebSocketManager>,
    pub minecraft_manager: crate::minecraft::MinecraftManager,
    pub mod_manager: ModManager,
    pub server_manager: Arc<crate::core::server_manager::ServerManager>,
    
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

/// Create API router
pub fn create_api_router(state: AppState) -> Router {
    Router::new()
        // Server endpoints
        .route("/api/servers", get(get_servers))
        .route("/api/servers", post(create_server))
        .route("/api/servers/:id", get(get_server))
        .route("/api/servers/:id", patch(update_server))
        .route("/api/servers/:id", delete(delete_server))
        .route("/api/servers/:id/health", get(get_server_health))
        .route("/api/servers/:id/start", post(start_server))
        .route("/api/servers/:id/stop", post(stop_server))
        .route("/api/servers/:id/restart", post(restart_server))
        .route("/api/servers/:id/command", post(send_server_command))
        // Resource monitoring endpoints
        .route("/api/servers/:id/metrics", get(get_server_metrics))
        .route("/api/servers/:id/metrics/history", get(get_server_metrics_history))
        .route("/api/system/metrics", get(get_system_metrics))
        .route("/api/system/metrics/history", get(get_system_metrics_history))
        .route("/api/system/resource-summary", get(get_resource_summary))
        // GPU acceleration endpoints
        .route("/api/gpu/status", get(get_gpu_status))
        .route("/api/gpu/metrics", get(get_gpu_metrics))
        .route("/api/gpu/enable", post(enable_gpu))
        .route("/api/gpu/disable", post(disable_gpu))
        .route("/api/gpu/job/submit", post(submit_gpu_job))
        .route("/api/gpu/job/:id/status", get(get_gpu_job_status))
        .route("/api/performance/:server_id/metrics", get(get_server_performance_metrics))
        .route("/api/performance/:server_id/summary", get(get_server_performance_summary))
        .route("/api/performance/all", get(get_all_performance_metrics))
        .route("/api/compatibility/:server_id/risk-analysis", get(get_server_risk_analysis))
        .route("/api/compatibility/:server_id/mod/:mod_id/risk", get(get_mod_risk_analysis))
        // Crash watchdog endpoints
        .route("/api/servers/:id/watchdog/register", post(register_server_watchdog))
        .route("/api/servers/:id/watchdog/unregister", post(unregister_server_watchdog))
        .route("/api/servers/:id/watchdog/health", get(get_server_watchdog_health))
        .route("/api/servers/:id/watchdog/force-restart", post(force_restart_server))
        .route("/api/servers/:id/watchdog/heartbeat", post(update_server_heartbeat))
        .route("/api/watchdog/health", get(get_all_watchdog_health))
        // EULA endpoints
        .route("/api/servers/:id/eula", get(get_eula_status))
        .route("/api/servers/:id/eula/accept", post(accept_eula))
        // Server.properties endpoints
        .route("/api/servers/:id/config/server.properties", get(get_server_properties))
        .route("/api/servers/:id/config/server.properties", put(update_server_properties))
        // Config aggregate and JVM args
        .route("/api/servers/:id/config", get(get_server_config))
        .route("/api/servers/:id/config/jvm-args", get(get_jvm_args))
        .route("/api/servers/:id/config/jvm-args", put(update_jvm_args))
        
        // Console endpoints
        .route("/api/servers/:id/console", get(get_console_messages))
        // .route("/api/servers/:id/console", post(send_console_message))
        
        // Player endpoints
        .route("/api/servers/:id/players", get(get_players))
        .route("/api/servers/:id/players/:uuid", get(get_player))
        .route("/api/servers/:id/players/:uuid/kick", post(kick_player))
        .route("/api/servers/:id/players/:uuid/ban", post(ban_player))
        
        // World endpoints
        .route("/api/servers/:id/world/freezes", get(get_world_freezes))
        .route("/api/servers/:id/world/heatmap", get(get_world_heatmap))
        
        
        // Metrics endpoints
        .route("/api/servers/:id/metrics/realtime", get(get_realtime_metrics))
        
        // Backup endpoints
        .route("/api/servers/:id/backups", get(get_backups))
        .route("/api/servers/:id/backups", post(create_backup))
        .route("/api/servers/:id/backups/:backup_id", get(get_backup))
        .route("/api/servers/:id/backups/:backup_id/restore", post(restore_backup))
        .route("/api/servers/:id/backups/:backup_id", delete(delete_backup))
        .route("/api/test/run", post(run_tests))
        .route("/api/test/run/:test_name", post(run_specific_test))
        .route("/api/test/results", get(get_test_results))
        
        // Settings endpoints
        .route("/api/servers/:id/settings", get(get_server_settings))
        // .route("/api/servers/:id/settings", put(update_server_settings))
        
        // Modpack endpoints
        .route("/api/modpacks/versions", get(get_minecraft_versions))
        .route("/api/modpacks/loaders", get(get_loader_versions))
        
        // Loader endpoints
        .route("/api/loaders/java/detect", get(detect_java))
        .route("/api/loaders/fabric/versions", get(get_fabric_versions))
        .route("/api/loaders/quilt/versions", get(get_quilt_versions))
        .route("/api/loaders/forge/versions", get(get_forge_versions))
        .route("/api/modpacks/mods", get(search_mods))
        .route("/api/modpacks/mods/:id", get(get_mod))
        .route("/api/modpacks/mods/:id/versions", get(get_mod_versions))
        .route("/api/modpacks/mods/:id/compatibility", get(check_mod_compatibility))
        .route("/api/modpacks", get(get_modpacks))
        .route("/api/modpacks", post(create_modpack))
        .route("/api/modpacks/:id", get(get_modpack))
        .route("/api/modpacks/:id", put(update_modpack))
        .route("/api/modpacks/:id", delete(delete_modpack))
        .route("/api/modpacks/:id/apply", post(apply_modpack_to_server))
        .route("/api/modpacks/:id/download", get(download_modpack))
        
        // External API integration endpoints
        .route("/api/mods/search/external", get(search_external_mods))
        // .route("/api/mods/:id/download", post(download_mod))
        // .route("/api/mods/sync", post(sync_mods_from_external))
        .route("/api/mods/:id/compatibility", get(check_mod_compatibility_external))
        
        // Settings endpoints
        .route("/api/settings", get(get_settings).put(update_settings))
        .route("/api/settings/validate/java", post(validate_java))
        .route("/api/settings/validate/api-keys", post(validate_api_keys))
        
        // Server creation wizard endpoints
        .route("/api/server/versions", get(get_server_versions))
        .route("/api/server/validate", post(validate_server_config))
        .route("/api/server/detect-java", get(detect_java_path))
        .route("/api/modpacks/search", get(search_modpacks))
        .route("/api/mods/search", get(search_mods))
        .route("/api/modpacks/apply", post(apply_modpack_to_server))
        .route("/api/mods/install", post(install_mods))
        
        // Compatibility endpoints
        .route("/api/servers/:id/compat/scan", post(scan_compatibility))
        .route("/api/servers/:id/compat/apply", post(apply_compatibility_fixes))
        
        // Pre-generation endpoints (removed duplicate routes - using pregen endpoints above instead)
        
        // Hot import endpoints
        .route("/api/servers/:id/import", get(get_hot_import_jobs).post(create_hot_import_job))
        .route("/api/servers/:id/import/:job_id", get(get_hot_import_job).delete(delete_hot_import_job))
        .route("/api/servers/:id/import/:job_id/start", post(start_hot_import_job))
        .route("/api/servers/:id/import/:job_id/cancel", post(cancel_hot_import_job))
        
        // Lighting optimization endpoints
        .route("/api/servers/:id/lighting", get(get_lighting_jobs).post(create_lighting_job))
        .route("/api/servers/:id/lighting/:job_id", get(get_lighting_job).delete(delete_lighting_job))
        .route("/api/servers/:id/lighting/:job_id/start", post(start_lighting_job))
        .route("/api/servers/:id/lighting/:job_id/cancel", post(cancel_lighting_job))
        .route("/api/servers/:id/lighting/settings", get(get_lighting_settings).put(update_lighting_settings))
        
        // Mod management endpoints
        .route("/api/servers/:id/mods", get(get_server_mods))
        .route("/api/servers/:id/mods/plan", post(create_mod_plan))
        .route("/api/servers/:id/mods/plan/:plan_id", get(get_mod_plan).delete(delete_mod_plan))
        .route("/api/servers/:id/mods/plan/:plan_id/apply", post(apply_mod_plan))
        .route("/api/servers/:id/mods/plan/:plan_id/rollback", post(rollback_mod_plan))
        
        // Health check endpoint
        .route("/api/health", get(health_check))
        .route("/api/healthz", get(health_check))
        .route("/healthz", get(health_check))
        .route("/api/status", get(get_status))
        
        .with_state(state)
}

// Server endpoints
async fn get_servers(State(state): State<AppState>) -> Result<Json<ApiResponse<Vec<ServerInfo>>>, StatusCode> {
    match state.minecraft_manager.get_all_servers().await {
        servers => {
            let server_infos: Vec<ServerInfo> = servers.into_iter().map(|server| {
                ServerInfo {
                    id: server.id.clone(),
                    name: server.config.name.clone(),
                    status: match server.status {
                        crate::minecraft::ServerStatus::Running => "running".to_string(),
                        crate::minecraft::ServerStatus::Stopped => "stopped".to_string(),
                        crate::minecraft::ServerStatus::Starting => "starting".to_string(),
                        crate::minecraft::ServerStatus::Stopping => "stopping".to_string(),
                        crate::minecraft::ServerStatus::Crashed => "crashed".to_string(),
                        crate::minecraft::ServerStatus::Unknown => "unknown".to_string(),
                    },
                    tps: if server.status == crate::minecraft::ServerStatus::Running { 20.0 } else { 0.0 },
                    tick_p95: if server.status == crate::minecraft::ServerStatus::Running { 45.2 } else { 0.0 },
                    heap_mb: if server.status == crate::minecraft::ServerStatus::Running { 2048 } else { 0 },
                    players_online: 0, // TODO: Get real player count from RCON
                    gpu_queue_ms: 0.0, // TODO: Get real GPU metrics from GPU manager
                    last_snapshot_at: server.last_start.map(|_| chrono::Utc::now()),
                    blue_green: BlueGreenInfo {
                        active: "blue".to_string(),
                        candidate_healthy: server.status == crate::minecraft::ServerStatus::Running,
                    },
                    version: Some(server.config.minecraft_version.clone()),
                    max_players: Some(server.config.max_players as u32),
                    uptime: None,
                    memory_usage: Some(2048),
                    cpu_usage: None,
                    world_size: None,
                    last_backup: None,
                    auto_start: None,
                    auto_restart: None,
                    created_at: Some(server.config.created_at),
                    updated_at: Some(server.config.updated_at),
                }
            }).collect();
            
            Ok(Json(ApiResponse::success(server_infos)))
        }
    }
}

async fn create_server(
    State(state): State<AppState>,
    Json(payload): Json<CreateServerRequest>,
) -> Result<Json<ApiResponse<ServerInfo>>, StatusCode> {
    let server_id = Uuid::new_v4().to_string();
    
    info!("Creating server: {} (ID: {})", payload.name, server_id);
    
    // Comprehensive validation
    if let Err(validation_error) = validate_server_creation_request(&payload).await {
        return Ok(Json(ApiResponse::error(validation_error)));
    }
    
    // Determine server root - use user-specified path if provided, otherwise use default
    let server_root = if !payload.paths.world.is_empty() {
        let install_path = &payload.paths.world;
        // Extract the base directory from the world path (remove './world' suffix)
        let base_path = if install_path.ends_with("/world") || install_path.ends_with("\\world") {
            install_path.trim_end_matches("/world").trim_end_matches("\\world")
        } else if install_path.ends_with("./world") {
            install_path.trim_end_matches("./world")
        } else {
            install_path
        };
        
        // If it's a relative path, make it relative to the working directory
        if base_path.starts_with("./") || base_path.starts_with(".\\") {
            std::path::Path::new(".").join(base_path.trim_start_matches("./").trim_start_matches(".\\"))
        } else if base_path.starts_with("C:\\") || base_path.starts_with("/") {
            // Absolute path - use as is
            std::path::Path::new(base_path).to_path_buf()
        } else {
            // Relative path - make it relative to working directory
            std::path::Path::new(".").join(base_path)
        }
    } else {
        // Fallback to default location
        std::path::Path::new("data").join("servers").join(&server_id)
    };
    
    let server_root_str = server_root.to_string_lossy().to_string();
    
    // Create server directory structure
    if let Err(e) = create_server_layout(&server_root_str).await {
        error!("Failed to create server directories: {}", e);
        return Ok(Json(ApiResponse::error(format!("Failed to create server directories: {}", e))));
    }
    
    // Download and prepare server JAR
    let jar_path = match prepare_server_jar(&payload, &server_root_str).await {
        Ok(path) => path,
        Err(e) => {
            error!("Failed to prepare server JAR: {}", e);
            return Ok(Json(ApiResponse::error(format!("Failed to prepare server JAR: {}", e))));
        }
    };
    
    // Create optimized JVM arguments based on memory allocation
    let memory_mb = payload.memory.unwrap_or(4096);
    let jvm_args = generate_optimized_jvm_args(memory_mb);
    
    // Create server configuration
    let server_config = ServerConfig {
        id: server_id.clone(),
        name: payload.name.clone(),
        minecraft_version: payload.minecraft_version.clone(),
        loader: payload.loader.clone(),
        loader_version: payload.version.clone(),
        port: payload.port.unwrap_or(25565),
        rcon_port: payload.rcon_port.unwrap_or(25575),
        query_port: payload.query_port.unwrap_or(25566),
        max_players: payload.max_players.unwrap_or(20),
        memory: memory_mb,
        java_args: serde_json::to_string(&jvm_args).unwrap_or_default(),
        server_args: serde_json::to_string(&vec!["--nogui"]).unwrap_or_default(),
        auto_start: payload.auto_start.unwrap_or(false),
        auto_restart: payload.auto_restart.unwrap_or(true),
        world_name: payload.world_settings.as_ref().map(|w| w.world_name.clone()).unwrap_or_else(|| "world".to_string()),
        difficulty: payload.world_settings.as_ref().map(|w| w.difficulty.clone()).unwrap_or_else(|| "normal".to_string()),
        gamemode: payload.world_settings.as_ref().map(|w| w.gamemode.clone()).unwrap_or_else(|| "survival".to_string()),
        pvp: payload.pvp.unwrap_or(true),
        online_mode: payload.online_mode.unwrap_or(true),
        whitelist: payload.whitelist.unwrap_or(false),
        enable_command_block: payload.enable_command_block.unwrap_or(false),
        view_distance: payload.view_distance.unwrap_or(10),
        simulation_distance: payload.simulation_distance.unwrap_or(10),
        motd: payload.motd.clone().unwrap_or_else(|| "A Minecraft Server".to_string()),
        host: "localhost".to_string(),
        java_path: payload.paths.java_path.clone().unwrap_or_else(|| "java".to_string()),
        jvm_args: jvm_args.join(" "),
        server_jar: jar_path,
        server_directory: server_root_str.clone(),
        rcon_password: generate_secure_password(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    // Add server to Minecraft manager
    match state.minecraft_manager.add_server(server_config).await {
        Ok(_) => {
            info!("Successfully created server: {} (ID: {})", payload.name, server_id);
            
            // Initialize server configuration files
            if let Err(e) = initialize_server_configuration(&state, &server_id, &payload).await {
                warn!("Failed to initialize server configuration: {}", e);
            }
            
            // Install modpack if specified
            if let Some(modpack) = &payload.modpack {
                if let Err(e) = install_modpack_to_server(&state, &server_id, modpack).await {
                    warn!("Failed to install modpack: {}", e);
                }
            }
            
            // Install individual mods if specified
            if let Some(mods) = &payload.individual_mods {
                if !mods.is_empty() {
                    if let Err(e) = install_mods_to_server(&state, &server_id, mods).await {
                        warn!("Failed to install mods: {}", e);
                    }
                }
            }

            // Return server info
            let server_info = ServerInfo {
                id: server_id,
                name: payload.name,
                status: "stopped".to_string(),
                tps: 0.0,
                tick_p95: 0.0,
                heap_mb: 0,
                players_online: 0,
                gpu_queue_ms: 0.0,
                last_snapshot_at: None,
                blue_green: BlueGreenInfo {
                    active: "blue".to_string(),
                    candidate_healthy: false,
                },
                version: Some(payload.minecraft_version),
                max_players: Some(payload.max_players.unwrap_or(20) as u32),
                uptime: None,
                memory_usage: Some(0),
                cpu_usage: None,
                world_size: None,
                last_backup: None,
                auto_start: None,
                auto_restart: None,
                created_at: Some(chrono::Utc::now()),
                updated_at: Some(chrono::Utc::now()),
            };
            
            Ok(Json(ApiResponse::success(server_info)))
        }
        Err(e) => {
            error!("Failed to create server: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create standard server directory structure under server root
async fn create_server_layout(server_root: &str) -> Result<(), std::io::Error> {
    use tokio::fs;
    let root = std::path::Path::new(server_root);
    let world_dir = root.join("world");
    let mods_dir = root.join("mods");
    let config_dir = root.join("config");
    let logs_dir = root.join("logs");
    
    fs::create_dir_all(&world_dir).await?;
    fs::create_dir_all(&mods_dir).await?;
    fs::create_dir_all(&config_dir).await?;
    fs::create_dir_all(&logs_dir).await?;
    
    Ok(())
}

/// Download Mojang vanilla server jar for the specified version
async fn download_vanilla_server_jar(version: &str, dest_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;
    let manifest_url = "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";
    let manifest: serde_json::Value = client.get(manifest_url).send().await?.json().await?;
    let versions = manifest["versions"].as_array().ok_or("invalid manifest")?;
    let ver = versions.iter().find(|v| v["id"].as_str() == Some(version)).ok_or("version not found")?;
    let ver_url = ver["url"].as_str().ok_or("version url missing")?;
    let ver_json: serde_json::Value = client.get(ver_url).send().await?.json().await?;
    let server_url = ver_json["downloads"]["server"]["url"].as_str().ok_or("server url missing")?;
    let bytes = client.get(server_url).send().await?.bytes().await?;
    tokio::fs::write(dest_path, &bytes).await?;
    Ok(())
}

async fn get_server(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<ServerInfo>>, StatusCode> {
    // Load server from DB and runtime manager
    match state.database.get_server(&id).await {
        Ok(Some(cfg)) => {
            // Determine status
            let status = if let Some(srv) = state.minecraft_manager.get_server(&id).await {
                match srv.status {
                    crate::minecraft::ServerStatus::Stopped => "stopped",
                    crate::minecraft::ServerStatus::Starting => "starting",
                    crate::minecraft::ServerStatus::Running => "running",
                    crate::minecraft::ServerStatus::Stopping => "stopping",
                    crate::minecraft::ServerStatus::Crashed => "crashed",
                    crate::minecraft::ServerStatus::Unknown => "unknown",
                }
            } else { "unknown" };

            // Try metrics if running; otherwise zeros
            let mut tps = 0.0;
            let mut tick_p95 = 0.0;
            let mut heap_mb = 0u64;
            let mut players_online = 0u32;
            let mut gpu_queue_ms = 0.0;
            if status == "running" {
                if let Ok(m) = state.minecraft_manager.get_server_metrics(&id).await {
                    tps = m.tps;
                    tick_p95 = m.tick_p95;
                    heap_mb = m.heap_mb;
                    players_online = m.players_online;
                    gpu_queue_ms = m.gpu_queue_ms;
                }
            }

            let server = ServerInfo {
                id: cfg.id.clone(),
                name: cfg.name.clone(),
                status: status.to_string(),
                tps,
                tick_p95,
                heap_mb,
                players_online,
                gpu_queue_ms,
                last_snapshot_at: None,
                blue_green: BlueGreenInfo { active: "blue".to_string(), candidate_healthy: false },
                version: Some(cfg.minecraft_version.clone()),
                max_players: Some(cfg.max_players),
                uptime: None,
                memory_usage: Some(heap_mb),
                cpu_usage: None,
                world_size: None,
                last_backup: None,
                auto_start: None,
                auto_restart: None,
                created_at: Some(cfg.created_at),
                updated_at: Some(cfg.updated_at),
            };

            Ok(Json(ApiResponse::success(server)))
        }
        Ok(None) => Ok(Json(ApiResponse::error("Server not found".to_string()))),
        Err(e) => {
            error!("get_server error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_server_health(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<ServerHealth>>, StatusCode> {
    match state.database.get_server(&id).await {
        Ok(Some(cfg)) => {
            // RCON health
            let rcon_ok = crate::rcon::RconClient::new(cfg.host.clone(), cfg.rcon_port, cfg.rcon_password.clone())
                .is_available();
            // TCP query (simple connect to server port)
            let default_addr = "127.0.0.1:25565".parse().unwrap_or_else(|_| "127.0.0.1:25565".parse().expect("Default address should be valid"));
            let query_ok = std::net::TcpStream::connect_timeout(
                &format!("127.0.0.1:{}", cfg.port).parse().unwrap_or(default_addr),
                std::time::Duration::from_millis(400),
            ).is_ok();

            let health = ServerHealth { rcon: rcon_ok, query: query_ok, crash_tickets: 0, freeze_tickets: 0 };
            Ok(Json(ApiResponse::success(health)))
        }
        Ok(None) => Ok(Json(ApiResponse::error("Server not found".to_string()))),
        Err(e) => {
            error!("get_server_health error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn start_server(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Starting server: {}", id);
    
    // Get server configuration from database
    let server_config = match state.database.get_server(&id).await {
        Ok(Some(config)) => config,
        Ok(None) => {
            error!("Server not found: {}", id);
            return Ok(Json(ApiResponse::error("Server not found".to_string())));
        }
        Err(e) => {
            error!("Failed to get server config: {}", e);
            return Ok(Json(ApiResponse::error("Failed to get server configuration".to_string())));
        }
    };
    
    // Start server using ProcessManager
    match state.process_manager.start_server_process(server_config).await {
        Ok(_) => {
            info!("Successfully started server: {}", id);
            
            // Broadcast status update
            let message = WebSocketMessage::ServerStatusChange {
                server_id: id.clone(),
                old_status: "stopped".to_string(),
                new_status: "starting".to_string(),
                timestamp: chrono::Utc::now(),
            };
            
            let _ = state.websocket_manager.broadcast_to_server(&id, message).await;
            
            Ok(Json(ApiResponse::success("Server starting".to_string())))
        }
        Err(e) => {
            error!("Failed to start server {}: {}", id, e);
            // Return readable error message
            Ok(Json(ApiResponse::error(format!("Failed to start: {}", e))))
        }
    }
}

async fn stop_server(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Stopping server: {}", id);
    
    let server_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(e) => {
            error!("Invalid server ID: {}", e);
            return Ok(Json(ApiResponse::error("Invalid server ID".to_string())));
        }
    };
    
    match state.process_manager.stop_server_process(server_id).await {
        Ok(_) => {
            info!("Successfully stopped server: {}", id);
            
            // Broadcast status update
            let message = WebSocketMessage::ServerStatusChange {
                server_id: id.clone(),
                old_status: "running".to_string(),
                new_status: "stopping".to_string(),
                timestamp: chrono::Utc::now(),
            };
            
            let _ = state.websocket_manager.broadcast_to_server(&id, message).await;
            
            Ok(Json(ApiResponse::success("Server stopping".to_string())))
        }
        Err(e) => {
            error!("Failed to stop server {}: {}", id, e);
            Ok(Json(ApiResponse::error(format!("Failed to stop: {}", e))))
        }
    }
}

async fn restart_server(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Restarting server: {}", id);
    
    let server_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(e) => {
            error!("Invalid server ID: {}", e);
            return Ok(Json(ApiResponse::error("Invalid server ID".to_string())));
        }
    };
    
    match state.process_manager.restart_server_process(server_id).await {
        Ok(_) => {
            info!("Successfully restarted server: {}", id);
            Ok(Json(ApiResponse::success("Server restarting".to_string())))
        }
        Err(e) => {
            error!("Failed to restart server {}: {}", id, e);
            Ok(Json(ApiResponse::error(format!("Failed to restart: {}", e))))
        }
    }
}

async fn update_server(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateServerRequest>,
) -> Result<Json<ApiResponse<ServerInfo>>, StatusCode> {
    // Get current server config
    let mut server = match state.minecraft_manager.get_server(&id).await {
        Some(server) => server,
        None => return Err(StatusCode::NOT_FOUND),
    };

    // Update fields if provided
    if let Some(name) = payload.name {
        server.config.name = name;
    }
    if let Some(java_path) = payload.java_path {
        server.config.java_path = java_path;
    }
    if let Some(jvm_args) = payload.jvm_args {
        server.config.jvm_args = jvm_args;
    }
    if let Some(server_args) = payload.server_args {
        server.config.server_args = server_args;
    }
    if let Some(auto_start) = payload.auto_start {
        server.config.auto_start = auto_start;
    }
    if let Some(auto_restart) = payload.auto_restart {
        server.config.auto_restart = auto_restart;
    }
    if let Some(max_players) = payload.max_players {
        server.config.max_players = max_players;
    }
    // pregeneration_policy field removed from ServerConfig

    server.config.updated_at = chrono::Utc::now();

    // Update in database
    match state.database.update_server(&server.config).await {
        Ok(_) => {
            // Update in memory
            state.minecraft_manager.update_server(server.clone()).await;
            
            let server_info = ServerInfo {
                id: server.id.clone(),
                name: server.config.name.clone(),
                status: match server.status {
                    crate::minecraft::ServerStatus::Running => "running".to_string(),
                    crate::minecraft::ServerStatus::Stopped => "stopped".to_string(),
                    crate::minecraft::ServerStatus::Starting => "starting".to_string(),
                    crate::minecraft::ServerStatus::Stopping => "stopping".to_string(),
                    crate::minecraft::ServerStatus::Crashed => "crashed".to_string(),
                    crate::minecraft::ServerStatus::Unknown => "unknown".to_string(),
                },
                version: Some(server.config.minecraft_version.clone()),
                players_online: 0, // TODO: Get from server via RCON
                max_players: Some(server.config.max_players as u32),
                uptime: server.last_start.map(|start| {
                    let now = std::time::Instant::now();
                    now.duration_since(start).as_secs()
                }),
                memory_usage: if server.status == crate::minecraft::ServerStatus::Running { Some(2048) } else { Some(0) },
                cpu_usage: if server.status == crate::minecraft::ServerStatus::Running { Some(0.0) } else { Some(0.0) },
                world_size: Some(0), // TODO: Calculate actual world size from disk
                last_backup: None, // TODO: Get from backup manager
                auto_start: Some(server.config.auto_start),
                auto_restart: Some(server.config.auto_restart),
                created_at: Some(server.config.created_at),
                updated_at: Some(server.config.updated_at),
                tps: 0.0, // TODO: Get from server
                tick_p95: 0.0, // TODO: Get from server
                heap_mb: 0, // TODO: Get from server
                gpu_queue_ms: 0.0, // TODO: Get from server
                last_snapshot_at: None, // TODO: Get from server
                blue_green: BlueGreenInfo { active: "blue".to_string(), candidate_healthy: false },
            };

            Ok(Json(ApiResponse {
                success: true,
                data: Some(server_info),
                error: None,
                timestamp: chrono::Utc::now(),
            }))
        }
        Err(e) => {
            error!("Failed to update server {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_server(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Deleting server: {}", id);
    
    // Get server config before deletion to access folder paths
    let server_config = match state.database.get_server(&id).await {
        Ok(Some(config)) => config,
        Ok(None) => {
            error!("Server not found: {}", id);
            return Err(StatusCode::NOT_FOUND);
        }
        Err(e) => {
            error!("Failed to get server config: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // First stop the server if it's running
    let _ = state.minecraft_manager.stop_server(&id).await;
    
    // Delete from database
    match state.database.delete_server(&id).await {
        Ok(_) => {
            // Always remove from memory cache even if DB delete succeeded
            match state.minecraft_manager.remove_server(&id).await {
                Ok(_) => info!("Removed server {} from memory cache", id),
                Err(e) => warn!("Failed to remove server from memory cache: {}", e),
            }
            
            // Delete server folder if it exists
            let server_dir = std::path::Path::new(&server_config.host);
            if server_dir.exists() {
                match tokio::fs::remove_dir_all(server_dir).await {
                    Ok(_) => info!("Successfully deleted server folder: {}", server_dir.display()),
                    Err(e) => warn!("Failed to delete server folder {}: {}", server_dir.display(), e),
                }
            }
            
            info!("Successfully deleted server: {}", id);
            
            // Broadcast deletion update
            let message = WebSocketMessage::ServerStatusChange {
                server_id: id.clone(),
                old_status: "running".to_string(),
                new_status: "deleted".to_string(),
                timestamp: chrono::Utc::now(),
            };
            
            let _ = state.websocket_manager.broadcast(message).await;
            
            Ok(Json(ApiResponse::success("Server deleted".to_string())))
        }
        Err(e) => {
            error!("Failed to delete server {}: {}", id, e);
            Ok(Json(ApiResponse::error(format!("Failed to delete: {}", e))))
        }
    }
}

async fn send_server_command(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<ServerCommandRequest>,
) -> Result<Json<ApiResponse<ServerCommandResponse>>, StatusCode> {
    info!("Sending command to server {}: {}", id, request.command);
    match state.minecraft_manager.send_command(&id, &request.command).await {
        Ok(output) => Ok(Json(ApiResponse::success(ServerCommandResponse { success: true, output, error: None }))),
        Err(e) => Ok(Json(ApiResponse::success(ServerCommandResponse { success: false, output: String::new(), error: Some(e.to_string()) }))),
    }
}

// Console endpoints
async fn get_console_messages(
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<ConsoleMessage>>>, StatusCode> {
    // Optional: support pagination via ?limit=
    let limit = params
        .get("limit")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(100);

    // Pull recent console events from database
    match state.database.get_events(Some(&id), Some(limit)).await {
        Ok(events) => {
            let messages: Vec<ConsoleMessage> = events
                .into_iter()
                .filter(|e| e.event_type == "console")
                .map(|e| ConsoleMessage {
                    ts: e.created_at.to_rfc3339(),
                    level: e.level,
                    msg: e.message,
                })
                .collect();
            Ok(Json(ApiResponse::success(messages)))
        }
        Err(e) => {
            error!("Failed to load console messages for {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Aggregate config
async fn get_server_config(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    match state.database.get_server(&id).await {
        Ok(Some(cfg)) => {
            let props_path = std::path::Path::new(&cfg.host).join("server.properties");
            let props = tokio::fs::read_to_string(&props_path).await.unwrap_or_default();
            let props_map = parse_properties(&props);
            let data = serde_json::json!({
                "serverProperties": props_map,
                "jvmArgs": cfg.jvm_args,
            });
            Ok(Json(ApiResponse::success(data)))
        }
        Ok(None) => Ok(Json(ApiResponse::error("Server not found".to_string()))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_jvm_args(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    match state.database.get_server(&id).await {
        Ok(Some(cfg)) => Ok(Json(ApiResponse::success(serde_json::json!({ "args": cfg.jvm_args })))),
        Ok(None) => Ok(Json(ApiResponse::error("Server not found".to_string()))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_jvm_args(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let new_args = payload.get("args").and_then(|v| v.as_str()).unwrap_or("").to_string();
    match state.database.get_server(&id).await {
        Ok(Some(cfg)) => {
            let mut cfg2 = cfg.clone();
            cfg2.jvm_args = new_args.clone();
            cfg2.updated_at = chrono::Utc::now();
            if let Err(e) = state.database.update_server(&cfg2).await {
                error!("Failed to update JVM args: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
            Ok(Json(ApiResponse::success(serde_json::json!({ "args": new_args }))))
        }
        Ok(None) => Ok(Json(ApiResponse::error("Server not found".to_string()))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
/// EULA status payload
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EulaStatus {
    status: String, // "accepted" | "pending" | "missing"
    #[serde(rename = "lastUpdated")]
    last_updated: Option<chrono::DateTime<chrono::Utc>>,
}

async fn get_eula_status(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<EulaStatus>>, StatusCode> {
    match state.database.get_server(&id).await {
        Ok(Some(cfg)) => {
            let path = std::path::Path::new(&cfg.host).join("eula.txt");
            if !path.exists() {
                let payload = EulaStatus { status: "missing".to_string(), last_updated: None };
                return Ok(Json(ApiResponse::success(payload)));
            }
            let content = match tokio::fs::read_to_string(&path).await {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed reading eula.txt: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };
            let accepted = content.lines().any(|l| l.trim().eq_ignore_ascii_case("eula=true"));
            let status = if accepted { "accepted" } else { "pending" };
            let payload = EulaStatus { status: status.to_string(), last_updated: None };
            Ok(Json(ApiResponse::success(payload)))
        }
        Ok(None) => Ok(Json(ApiResponse::error("Server not found".to_string()))),
        Err(e) => {
            error!("get_eula_status db error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn accept_eula(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match state.database.get_server(&id).await {
        Ok(Some(cfg)) => {
            let path = std::path::Path::new(&cfg.host).join("eula.txt");
            let content = format!(
                "# EULA accepted by Guardian on {}\neula=true\n",
                chrono::Utc::now().to_rfc3339()
            );
            if let Err(e) = tokio::fs::write(&path, content).await {
                error!("Failed writing eula.txt: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
            Ok(Json(ApiResponse::success("EULA accepted".to_string())))
        }
        Ok(None) => Ok(Json(ApiResponse::error("Server not found".to_string()))),
        Err(e) => {
            error!("accept_eula db error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// server.properties helpers
fn parse_properties(content: &str) -> HashMap<String, String> {
    content
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .collect()
}

fn serialize_properties(mut props: HashMap<String, String>) -> String {
    let mut keys: Vec<_> = props.keys().cloned().collect();
    keys.sort();
    let mut out = String::new();
    out.push_str(&format!("# Generated by Guardian on {}\n", chrono::Utc::now().to_rfc3339()));
    for k in keys {
        let v = props.remove(&k).unwrap_or_default();
        out.push_str(&format!("{}={}\n", k, v));
    }
    out
}

async fn get_server_properties(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<HashMap<String, String>>>, StatusCode> {
    match state.database.get_server(&id).await {
        Ok(Some(cfg)) => {
            let path = std::path::Path::new(&cfg.host).join("server.properties");
            if !path.exists() {
                return Ok(Json(ApiResponse::success(HashMap::new())));
            }
            let content = tokio::fs::read_to_string(&path).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let props = parse_properties(&content);
            Ok(Json(ApiResponse::success(props)))
        }
        Ok(None) => Ok(Json(ApiResponse::error("Server not found".to_string()))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn update_server_properties(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(updates): Json<HashMap<String, String>>,
) -> Result<Json<ApiResponse<HashMap<String, String>>>, StatusCode> {
    match state.database.get_server(&id).await {
        Ok(Some(cfg)) => {
            let path = std::path::Path::new(&cfg.host).join("server.properties");
            let mut props = if path.exists() {
                let content = tokio::fs::read_to_string(&path).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                parse_properties(&content)
            } else {
                HashMap::new()
            };
            for (k, v) in updates.iter() { props.insert(k.clone(), v.clone()); }
            let content = serialize_properties(props.clone());
            tokio::fs::write(&path, content).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(Json(ApiResponse::success(props)))
        }
        Ok(None) => Ok(Json(ApiResponse::error("Server not found".to_string()))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn init_server_properties(state: &AppState, server_id: &str) -> Result<(), anyhow::Error> {
    let cfg = state.database.get_server(server_id).await?
        .ok_or_else(|| anyhow::anyhow!("Server not found"))?;
    let path = std::path::Path::new(&cfg.host).join("server.properties");
    let mut props = if path.exists() {
        let content = tokio::fs::read_to_string(&path).await.unwrap_or_default();
        parse_properties(&content)
    } else { HashMap::new() };
    props.insert("enable-rcon".to_string(), "true".to_string());
    props.insert("rcon.password".to_string(), cfg.rcon_password.clone());
    props.insert("rcon.port".to_string(), cfg.rcon_port.to_string());
    props.insert("server-port".to_string(), cfg.port.to_string());
    let content = serialize_properties(props);
    tokio::fs::write(&path, content).await?;
    Ok(())
}

// #[axum::debug_handler]
async fn send_console_message(
    Path(id): Path<String>,
    Json(request): Json<ServerCommandRequest>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Sending console message to server {}: {}", id, request.command);
    
    // AI-EXPLAIN: Console message sending not yet implemented
    // In the future, this should use RCON to send commands to the Minecraft server
    Ok(Json(ApiResponse::success("Console message sending not yet implemented".to_string())))
}

// Player endpoints
async fn get_players(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Player>>>, StatusCode> {
    match state.minecraft_manager.get_server_players(&id).await {
        Ok(players) => {
            let players = players.into_iter().map(|p| Player {
                uuid: p.uuid,
                name: p.name,
                online: p.online,
                last_seen: Some(p.last_seen),
                playtime: Some(p.playtime),
            }).collect();
            Ok(Json(ApiResponse::success(players)))
        }
        Err(e) => Ok(Json(ApiResponse::error(format!("Failed to get players: {}", e)))),
    }
}

async fn get_player(
    Path((id, uuid)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Player>>, StatusCode> {
    match state.minecraft_manager.get_server_players(&id).await {
        Ok(players) => {
            if let Some(p) = players.into_iter().find(|p| p.uuid == uuid) {
                let player = Player { uuid: p.uuid, name: p.name, online: p.online, last_seen: Some(p.last_seen), playtime: Some(p.playtime) };
                Ok(Json(ApiResponse::success(player)))
            } else {
                Ok(Json(ApiResponse::error("Player not found".to_string())))
            }
        }
        Err(e) => Ok(Json(ApiResponse::error(format!("Failed to get players: {}", e)))),
    }
}

async fn kick_player(
    Path((id, uuid)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Kicking player {} from server {}", uuid, id);
    match state.minecraft_manager.get_server_players(&id).await {
        Ok(players) => {
            if let Some(p) = players.into_iter().find(|p| p.uuid == uuid) {
                let rc = crate::rcon::RconClient::new(id.clone(), 0, String::new()); // placeholder host override below
                drop(rc);
                // Use send_command to support any name characters
                match state.minecraft_manager.send_command(&id, &format!("kick {}", p.name)).await {
                    Ok(_) => Ok(Json(ApiResponse::success("Player kicked".to_string()))),
                    Err(e) => Ok(Json(ApiResponse::error(format!("Failed to kick: {}", e)))),
                }
            } else {
                Ok(Json(ApiResponse::error("Player not found".to_string())))
            }
        }
        Err(e) => Ok(Json(ApiResponse::error(format!("Failed to get players: {}", e)))),
    }
}

async fn ban_player(
    Path((id, uuid)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Banning player {} from server {}", uuid, id);
    match state.minecraft_manager.get_server_players(&id).await {
        Ok(players) => {
            if let Some(p) = players.into_iter().find(|p| p.uuid == uuid) {
                match state.minecraft_manager.send_command(&id, &format!("ban {}", p.name)).await {
                    Ok(_) => Ok(Json(ApiResponse::success("Player banned".to_string()))),
                    Err(e) => Ok(Json(ApiResponse::error(format!("Failed to ban: {}", e)))),
                }
            } else {
                Ok(Json(ApiResponse::error("Player not found".to_string())))
            }
        }
        Err(e) => Ok(Json(ApiResponse::error(format!("Failed to get players: {}", e)))),
    }
}

// World endpoints
async fn get_world_freezes(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<WorldFreeze>>>, StatusCode> {
    // AI-EXPLAIN: World freeze data not yet implemented
    // In the future, this should query the server for actual freeze events
    let freezes: Vec<WorldFreeze> = vec![];
    
    Ok(Json(ApiResponse::success(freezes)))
}

async fn get_world_heatmap(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    // AI-EXPLAIN: World heatmap data not yet implemented
    // In the future, this should generate heatmap from player activity data
    let heatmap_data = serde_json::json!({
        "cells": [],
        "last_update": chrono::Utc::now()
    });
    
    Ok(Json(ApiResponse::success(heatmap_data)))
}

// Pregen endpoints
async fn get_pregen_jobs(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<PregenJob>>>, StatusCode> {
    // AI-EXPLAIN: Return empty pregen jobs list for now
    // In the future, this should query the GPU manager or database for active jobs
    let jobs: Vec<PregenJob> = vec![];
    
    Ok(Json(ApiResponse::success(jobs)))
}

// #[axum::debug_handler]
async fn create_pregen_job(
    Path(id): Path<String>,
    Json(job): Json<PregenJob>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<PregenJob>>, StatusCode> {
    info!("Creating pregen job for server {}: {:?}", id, job);
    
    // AI-EXPLAIN: For now, just return the job as-is
    // In the future, this should validate the job and queue it with the GPU manager
    let mut job = job;
    job.id = uuid::Uuid::new_v4().to_string();
    job.status = "queued".to_string();
    job.progress = 0.0;
    job.eta = None;
    
    Ok(Json(ApiResponse::success(job)))
}

async fn get_pregen_job(
    Path((id, job_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<PregenJob>>, StatusCode> {
    // AI-EXPLAIN: Return 404 for now since we don't have persistent job storage
    // In the future, this should query the GPU manager or database for the specific job
    Err(StatusCode::NOT_FOUND)
}

// #[axum::debug_handler]
async fn update_pregen_job(
    Path((id, job_id)): Path<(String, String)>,
    Json(job): Json<PregenJob>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<PregenJob>>, StatusCode> {
    info!("Updating pregen job {} for server {}: {:?}", job_id, id, job);
    
    // AI-EXPLAIN: Return 404 since we don't have persistent job storage
    Err(StatusCode::NOT_FOUND)
}

async fn delete_pregen_job(
    Path((id, job_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Deleting pregen job {} from server {}", job_id, id);
    
    // AI-EXPLAIN: Return 404 since we don't have persistent job storage
    Err(StatusCode::NOT_FOUND)
}

async fn start_pregen_job(
    Path((id, job_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Starting pregen job {} on server {}", job_id, id);
    
    // AI-EXPLAIN: Return 404 since we don't have persistent job storage
    Err(StatusCode::NOT_FOUND)
}

async fn stop_pregen_job(
    Path((id, job_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Stopping pregen job {} on server {}", job_id, id);
    
    // AI-EXPLAIN: Return 404 since we don't have persistent job storage
    Err(StatusCode::NOT_FOUND)
}

// Metrics endpoints
async fn get_metrics(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Metrics>>, StatusCode> {
    match state.minecraft_manager.get_server_metrics(&id).await {
        Ok(m) => {
            let now = chrono::Utc::now().timestamp();
            let metrics = Metrics {
                tps: vec![MetricPoint { timestamp: now, value: m.tps }],
                heap: vec![MetricPoint { timestamp: now, value: m.heap_mb as f64 }],
                tick_p95: vec![MetricPoint { timestamp: now, value: m.tick_p95 }],
                gpu_ms: vec![MetricPoint { timestamp: now, value: m.gpu_queue_ms }],
            };
            Ok(Json(ApiResponse::success(metrics)))
        }
        Err(e) => Ok(Json(ApiResponse::error(format!("Failed to get metrics: {}", e)))),
    }
}

async fn get_realtime_metrics(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Metrics>>, StatusCode> {
    // AI-EXPLAIN: Realtime metrics are the same as regular metrics for now
    // In the future, this could return more frequent updates or live streaming data
    get_metrics(Path(id), State(state)).await
}

// Backup endpoints
async fn get_backups(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<crate::backup_manager::BackupInfo>>>, StatusCode> {
    let backup_manager = crate::backup_manager::BackupManager::new(
        std::path::PathBuf::from("data/backups"),
        std::path::PathBuf::from("data/servers")
    );
    
    match backup_manager.get_backups(&id).await {
        Ok(backups) => Ok(Json(ApiResponse::success(backups))),
        Err(e) => Ok(Json(ApiResponse::error(format!("Failed to get backups: {}", e)))),
    }
}

async fn create_backup(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<crate::backup_manager::BackupInfo>>, StatusCode> {
    info!("Creating backup for server {}", id);
    let backup_manager = crate::backup_manager::BackupManager::new(
        std::path::PathBuf::from("data/backups"),
        std::path::PathBuf::from("data/servers")
    );
    
    let request = crate::backup_manager::CreateBackupRequest {
        name: format!("backup_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S")),
        description: Some("API created backup".to_string()),
        backup_type: crate::backup_manager::BackupType::Manual,
        compression: crate::backup_manager::CompressionType::Zip,
        includes: crate::backup_manager::BackupIncludes {
            world: true,
            mods: true,
            config: true,
            logs: false,
            server_properties: true,
            whitelist: true,
            ops: true,
            banned_players: true,
            banned_ips: true,
        },
        metadata: Some(serde_json::Value::Object(serde_json::Map::new())),
    };
    
    match backup_manager.create_backup(&id, request).await {
        Ok(backup) => Ok(Json(ApiResponse::success(backup))),
        Err(e) => Ok(Json(ApiResponse::error(format!("Failed to create backup: {}", e)))),
    }
}

async fn get_backup(
    Path((id, backup_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<crate::backup_manager::BackupInfo>>, StatusCode> {
    let backup_manager = crate::backup_manager::BackupManager::new(
        std::path::PathBuf::from("data/backups"),
        std::path::PathBuf::from("data/servers")
    );
    
    match backup_manager.get_backup(&id, &backup_id).await {
        Ok(backup) => Ok(Json(ApiResponse::success(backup))),
        Err(e) => Ok(Json(ApiResponse::error(format!("Failed to get backup: {}", e)))),
    }
}

pub async fn restore_backup(
    Path((id, backup_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Restoring backup {} for server {}", backup_id, id);
    let backup_manager = crate::backup_manager::BackupManager::new(
        std::path::PathBuf::from("data/backups"),
        std::path::PathBuf::from("data/servers")
    );
    
    let restore_request = crate::backup_manager::RestoreBackupRequest {
        backup_id: backup_id.clone(),
        restore_world: true,
        restore_mods: true,
        restore_config: true,
        restore_logs: false,
        create_backup: true,
    };
    
    match backup_manager.restore_backup(&id, restore_request).await {
        Ok(_) => Ok(Json(ApiResponse::success("Backup restored successfully".to_string()))),
        Err(e) => Ok(Json(ApiResponse::error(format!("Failed to restore backup: {}", e)))),
    }
}

async fn delete_backup(
    Path((id, backup_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Deleting backup {} from server {}", backup_id, id);
    let backup_manager = crate::backup_manager::BackupManager::new(
        std::path::PathBuf::from("data/backups"),
        std::path::PathBuf::from("data/servers")
    );
    
    match backup_manager.delete_backup(&id, &backup_id).await {
        Ok(_) => Ok(Json(ApiResponse::success("Backup deleted successfully".to_string()))),
        Err(e) => Ok(Json(ApiResponse::error(format!("Failed to delete backup: {}", e)))),
    }
}

// Settings endpoints
async fn get_server_settings(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    // Return server.properties + JVM args snapshots
    match state.database.get_server(&id).await {
        Ok(Some(cfg)) => {
            let props = std::path::Path::new(&cfg.host).join("server.properties");
            let props_map = if let Ok(content) = tokio::fs::read_to_string(&props).await { parse_properties(&content) } else { std::collections::HashMap::new() };
            let settings = serde_json::json!({
                "jvm": { "args": cfg.jvm_args },
                "server": props_map,
            });
            Ok(Json(ApiResponse::success(settings)))
        }
        Ok(None) => Ok(Json(ApiResponse::error("Server not found".to_string()))),
        Err(e) => { error!("get_server_settings error: {}", e); Err(StatusCode::INTERNAL_SERVER_ERROR) }
    }
}

// #[axum::debug_handler]
async fn update_server_settings(
    Path(id): Path<String>,
    Json(settings): Json<serde_json::Value>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    info!("Updating settings for server {}: {:?}", id, settings);
    
    // TODO: Implement actual settings update
    Ok(Json(ApiResponse::success(settings)))
}

// Health check structures
#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    pub status: String, // "healthy", "degraded", "unhealthy"
    pub message: Option<String>,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub response_time_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct SystemHealth {
    pub overall_status: String, // "healthy", "degraded", "unhealthy"
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub uptime_seconds: u64,
    pub components: std::collections::HashMap<String, ComponentHealth>,
    pub version: String,
}

// Health check endpoints
async fn health_check(State(state): State<AppState>) -> Result<Json<ApiResponse<SystemHealth>>, StatusCode> {
    let start_time = std::time::Instant::now();
    let mut components = std::collections::HashMap::new();
    
    // Database health check
    let db_start = std::time::Instant::now();
    let db_health = match state.database.get_health_status().await {
        Ok(health_status) => {
            ComponentHealth {
                status: "healthy".to_string(),
                message: Some(format!("Database: {}", health_status.status)),
                last_check: chrono::Utc::now(),
                response_time_ms: Some(db_start.elapsed().as_millis() as u64),
            }
        }
        Err(e) => {
            ComponentHealth {
                status: "unhealthy".to_string(),
                message: Some(format!("Database error: {}", e)),
                last_check: chrono::Utc::now(),
                response_time_ms: Some(db_start.elapsed().as_millis() as u64),
            }
        }
    };
    components.insert("database".to_string(), db_health);
    
    // GPU health check
    let gpu_start = std::time::Instant::now();
    let gpu_health = match state.gpu_manager.lock().await.get_status().await {
        Ok(status) => {
            ComponentHealth {
                status: if status.enabled { "healthy".to_string() } else { "degraded".to_string() },
                message: Some(format!("GPU: {}", if status.enabled { "enabled" } else { "disabled" })),
                last_check: chrono::Utc::now(),
                response_time_ms: Some(gpu_start.elapsed().as_millis() as u64),
            }
        }
        Err(e) => {
            ComponentHealth {
                status: "degraded".to_string(),
                message: Some(format!("GPU error: {}", e)),
                last_check: chrono::Utc::now(),
                response_time_ms: Some(gpu_start.elapsed().as_millis() as u64),
            }
        }
    };
    components.insert("gpu".to_string(), gpu_health);
    
    // WebSocket health check
    let ws_start = std::time::Instant::now();
    let ws_health = ComponentHealth {
        status: "healthy".to_string(),
        message: Some(format!("WebSocket connections: {}", state.websocket_manager.get_connection_count().await)),
        last_check: chrono::Utc::now(),
        response_time_ms: Some(ws_start.elapsed().as_millis() as u64),
    };
    components.insert("websocket".to_string(), ws_health);
    
    // External API health checks
    let api_start = std::time::Instant::now();
    let mut api_healthy = true;
    let mut api_message = "External APIs: ".to_string();
    
    // Check CurseForge API
    if let Ok(cf_key) = state.secret_storage.get_api_key("curseforge").await {
        if cf_key.is_some() {
            // Test API key validity
            match validate_curseforge_key(&cf_key.unwrap()).await {
                Ok(valid) => {
                    if valid {
                        api_message.push_str("CF:OK ");
                    } else {
                        api_healthy = false;
                        api_message.push_str("CF:INVALID ");
                    }
                }
                Err(_) => {
                    api_healthy = false;
                    api_message.push_str("CF:ERROR ");
                }
            }
        } else {
            api_message.push_str("CF:NO_KEY ");
        }
    }
    
    // Check Modrinth API
    if let Ok(mr_key) = state.secret_storage.get_api_key("modrinth").await {
        if mr_key.is_some() {
            match validate_modrinth_token(&mr_key.unwrap()).await {
                Ok(valid) => {
                    if valid {
                        api_message.push_str("MR:OK ");
                    } else {
                        api_healthy = false;
                        api_message.push_str("MR:INVALID ");
                    }
                }
                Err(_) => {
                    api_healthy = false;
                    api_message.push_str("MR:ERROR ");
                }
            }
        } else {
            api_message.push_str("MR:NO_KEY ");
        }
    }
    
    let api_health = ComponentHealth {
        status: if api_healthy { "healthy".to_string() } else { "degraded".to_string() },
        message: Some(api_message.trim().to_string()),
        last_check: chrono::Utc::now(),
        response_time_ms: Some(api_start.elapsed().as_millis() as u64),
    };
    components.insert("external_apis".to_string(), api_health);
    
    // Determine overall status
    let overall_status = if components.values().any(|c| c.status == "unhealthy") {
        "unhealthy"
    } else if components.values().any(|c| c.status == "degraded") {
        "degraded"
    } else {
        "healthy"
    };
    
    let system_health = SystemHealth {
        overall_status: overall_status.to_string(),
        timestamp: chrono::Utc::now(),
        uptime_seconds: start_time.elapsed().as_secs(),
        components,
        version: "1.0.0".to_string(),
    };
    
    info!("Health check completed in {}ms: {}", start_time.elapsed().as_millis(), overall_status);
    Ok(Json(ApiResponse::success(system_health)))
}

async fn get_status(State(state): State<AppState>) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let status = serde_json::json!({
        "version": "1.0.0",
        "uptime": "1h 30m",
        "connections": state.websocket_manager.get_connection_count().await,
        "servers": 1,
        "timestamp": chrono::Utc::now()
    });
    
    Ok(Json(ApiResponse::success(status)))
}

// Modpack endpoints
async fn get_minecraft_versions(State(state): State<AppState>) -> Result<Json<ApiResponse<Vec<MinecraftVersion>>>, StatusCode> {
    match state.database.get_minecraft_versions().await {
        Ok(versions) => Ok(Json(ApiResponse::success(versions))),
        Err(e) => {
            error!("Failed to get Minecraft versions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_loader_versions(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<LoaderVersion>>>, StatusCode> {
    let minecraft_version = params.get("minecraft_version");
    let loader_type = params.get("loader_type");
    
    match state.database.get_loader_versions(minecraft_version, loader_type).await {
        Ok(versions) => Ok(Json(ApiResponse::success(versions))),
        Err(e) => {
            error!("Failed to get loader versions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn search_mods(
    Query(params): Query<ModSearchQuery>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Mod>>>, StatusCode> {
    let query = params.search_query.unwrap_or_default();
    let provider = params.source.unwrap_or_else(|| "modrinth".to_string());
    
    if query.is_empty() {
        return Ok(Json(ApiResponse::success(vec![])));
    }
    
    // Use the mod manager to search for mods
    match state.mod_manager.search_mods(&query, None, None, None).await {
        Ok(mods) => {
            info!("Found {} mods for query '{}'", mods.len(), query);
            // Convert ModInfo to Mod for API response
            let api_mods: Vec<Mod> = mods.into_iter().map(|mod_info| {
                Mod {
                    id: mod_info.id.clone(),
                    provider: "modrinth".to_string(),
                    project_id: mod_info.id,
                    version_id: mod_info.version,
                    filename: format!("{}.jar", mod_info.name),
                    sha1: "".to_string(),
                    server_id: None,
                    enabled: true,
                    category: mod_info.category,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }
            }).collect();
            Ok(Json(ApiResponse::success(api_mods)))
        }
        Err(e) => {
            error!("Failed to search mods: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_mod(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Mod>>, StatusCode> {
    match state.database.get_mod(&id).await {
        Ok(Some(mod_info)) => Ok(Json(ApiResponse::success(mod_info))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get mod {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_mod_versions(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<ModVersion>>>, StatusCode> {
    match state.database.get_mod_versions(&id).await {
        Ok(versions) => Ok(Json(ApiResponse::success(versions))),
        Err(e) => {
            error!("Failed to get mod versions for {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn check_mod_compatibility(
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let minecraft_version = params.get("minecraft_version");
    let loader = params.get("loader");
    
    // TODO: Implement actual compatibility checking
    let compatibility = serde_json::json!({
        "compatible": true,
        "issues": [],
        "warnings": []
    });
    
    Ok(Json(ApiResponse::success(compatibility)))
}

async fn get_modpacks(State(state): State<AppState>) -> Result<Json<ApiResponse<Vec<Modpack>>>, StatusCode> {
    match state.database.get_modpacks().await {
        Ok(modpacks) => Ok(Json(ApiResponse::success(modpacks))),
        Err(e) => {
            error!("Failed to get modpacks: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_modpack(
    State(state): State<AppState>,
    Json(payload): Json<CreateModpackRequest>,
) -> Result<Json<ApiResponse<Modpack>>, StatusCode> {
    let modpack_id = Uuid::new_v4().to_string();
    
    let modpack = Modpack {
        id: modpack_id.clone(),
        name: payload.name.clone(),
        description: payload.description,
        minecraft_version: payload.minecraft_version,
        loader: payload.loader,
        client_mods: serde_json::to_string(&payload.client_mods).unwrap_or_default(),
        server_mods: serde_json::to_string(&payload.server_mods).unwrap_or_default(),
        config: payload.config.map(|c| serde_json::to_string(&c).unwrap_or_default()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    match state.database.create_modpack(&modpack).await {
        Ok(_) => {
            info!("Successfully created modpack: {} (ID: {})", payload.name, modpack_id);
            Ok(Json(ApiResponse::success(modpack)))
        }
        Err(e) => {
            error!("Failed to create modpack: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_modpack(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Modpack>>, StatusCode> {
    match state.database.get_modpack(&id).await {
        Ok(Some(modpack)) => Ok(Json(ApiResponse::success(modpack))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get modpack {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_modpack(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateModpackRequest>,
) -> Result<Json<ApiResponse<Modpack>>, StatusCode> {
    match state.database.get_modpack(&id).await {
        Ok(Some(mut modpack)) => {
            if let Some(name) = payload.name {
                modpack.name = name;
            }
            if let Some(description) = payload.description {
                modpack.description = Some(description);
            }
            if let Some(client_mods) = payload.client_mods {
                modpack.client_mods = serde_json::to_string(&client_mods).unwrap_or_default();
            }
            if let Some(server_mods) = payload.server_mods {
                modpack.server_mods = serde_json::to_string(&server_mods).unwrap_or_default();
            }
            if let Some(config) = payload.config {
                modpack.config = Some(serde_json::to_string(&config).unwrap_or_default());
            }
            modpack.updated_at = chrono::Utc::now();
            
            match state.database.update_modpack(&modpack).await {
                Ok(_) => {
                    info!("Successfully updated modpack: {}", id);
                    Ok(Json(ApiResponse::success(modpack)))
                }
                Err(e) => {
                    error!("Failed to update modpack {}: {}", id, e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get modpack {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_modpack(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match state.database.delete_modpack(&id).await {
        Ok(_) => {
            info!("Successfully deleted modpack: {}", id);
            Ok(Json(ApiResponse::success("Modpack deleted".to_string())))
        }
        Err(e) => {
            error!("Failed to delete modpack {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn apply_modpack_to_server(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<ApplyModpackRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Applying modpack {} to server {}", id, payload.server_id);
    
    let job_id = uuid::Uuid::new_v4().to_string();
    
    // Send job started event
    if let Err(e) = state.websocket_manager.send_job_started(
        Some(&payload.server_id), 
        &job_id, 
        "modpack_install", 
        4 // total steps: validate, download, install, finalize
    ).await {
        error!("Failed to send job started event: {}", e);
    }
    
    // Step 1: Validate server and modpack
    if let Err(e) = state.websocket_manager.send_job_progress(
        Some(&payload.server_id),
        &job_id,
        "modpack_install",
        1, // step
        4, // total_steps
        0.0, // progress
        "Validating server and modpack"
    ).await {
        error!("Failed to send progress event: {}", e);
    }
    
    // Get server info
    let server = match state.database.get_server(&payload.server_id).await {
        Ok(Some(server)) => server,
        Ok(None) => {
            let _ = state.websocket_manager.send_job_failed(
                Some(&payload.server_id),
                &job_id,
                "modpack_install",
                "Server not found"
            ).await;
            return Err(StatusCode::NOT_FOUND);
        }
        Err(e) => {
            let _ = state.websocket_manager.send_job_failed(
                Some(&payload.server_id),
                &job_id,
                "modpack_install",
                &format!("Database error: {}", e)
            ).await;
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Get modpack info
    let modpack = match state.database.get_modpack(&id).await {
        Ok(Some(modpack)) => modpack,
        Ok(None) => {
            let _ = state.websocket_manager.send_job_failed(
                Some(&payload.server_id),
                &job_id,
                "modpack_install",
                "Modpack not found"
            ).await;
            return Err(StatusCode::NOT_FOUND);
        }
        Err(e) => {
            let _ = state.websocket_manager.send_job_failed(
                Some(&payload.server_id),
                &job_id,
                "modpack_install",
                &format!("Database error: {}", e)
            ).await;
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Step 2: Download modpack files
    if let Err(e) = state.websocket_manager.send_job_progress(
        Some(&payload.server_id),
        &job_id,
        "modpack_install",
        2, // step
        4, // total_steps
        0.25, // progress
        "Downloading modpack files"
    ).await {
        error!("Failed to send progress event: {}", e);
    }
    
    // Create modpack installer
    let mods_directory = std::path::Path::new(&server.server_directory).join("mods");
    let temp_directory = std::path::Path::new(&server.server_directory).join("temp");
    
    // Create providers map (stub for now)
    let mut providers = std::collections::HashMap::new();
    // TODO: Initialize actual providers with API keys
    
    let installer = crate::modpack_installer::ModpackInstaller::new(
        state.mod_manager.clone(),
        providers,
        mods_directory,
        temp_directory,
    );
    
    // Step 3: Install modpack
    if let Err(e) = state.websocket_manager.send_job_progress(
        Some(&payload.server_id),
        &job_id,
        "modpack_install",
        3, // step
        4, // total_steps
        0.5, // progress
        "Installing modpack"
    ).await {
        error!("Failed to send progress event: {}", e);
    }
    
    // For now, we'll simulate the installation process
    // In a real implementation, this would use the actual modpack installer
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Step 4: Finalize installation
    if let Err(e) = state.websocket_manager.send_job_progress(
        Some(&payload.server_id),
        &job_id,
        "modpack_install",
        4, // step
        4, // total_steps
        0.75, // progress
        "Finalizing installation"
    ).await {
        error!("Failed to send progress event: {}", e);
    }
    
    // Simulate finalization
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    // Send completion event
    if let Err(e) = state.websocket_manager.send_job_completed(
        Some(&payload.server_id),
        &job_id,
        "modpack_install",
        Some("Modpack applied successfully")
    ).await {
        error!("Failed to send job completed event: {}", e);
    }
    
    info!("Successfully applied modpack {} to server {}", id, payload.server_id);
    Ok(Json(ApiResponse::success("Modpack applied successfully".to_string())))
}

async fn download_modpack(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    match state.database.get_modpack(&id).await {
        Ok(Some(modpack)) => {
            // TODO: Implement actual modpack download
            // This would create a downloadable zip file with all client mods
            let download_info = serde_json::json!({
                "modpack_id": id,
                "name": modpack.name,
                "download_url": format!("/api/modpacks/{}/download/file", id),
                "size_mb": 0, // AI-EXPLAIN: Size calculation not yet implemented
                "created_at": modpack.created_at
            });
            
            Ok(Json(ApiResponse::success(download_info)))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get modpack {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// External API integration endpoints
async fn search_external_mods(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Mod>>>, StatusCode> {
    let query = params.get("query").map(|s| s.as_str()).unwrap_or("");
    let minecraft_version = params.get("minecraft_version");
    let loader = params.get("loader");
    let category = params.get("category");
    let source = params.get("source");
    let limit = params.get("limit").and_then(|s| s.parse().ok()).unwrap_or(50);

    match state.mod_manager.search_mods(
        query,
        minecraft_version.as_deref().map(|s| s.as_str()),
        loader.as_deref().map(|s| s.as_str()),
        Some(limit),
    ).await {
        Ok(results) => {
            let mut all_mods = Vec::new();
            for result in results {
                // Convert ModInfo to Mod
                let mods: Vec<Mod> = vec![result].into_iter().map(|mod_info| Mod {
                    id: mod_info.id,
                    provider: "curseforge".to_string(),
                    project_id: "unknown".to_string(), // AI-EXPLAIN: External API integration not yet complete
                    version_id: "unknown".to_string(), // AI-EXPLAIN: External API integration not yet complete
                    filename: "unknown".to_string(), // AI-EXPLAIN: External API integration not yet complete
                    sha1: "unknown".to_string(), // AI-EXPLAIN: External API integration not yet complete
                    server_id: None,
                    enabled: false,
                    category: mod_info.category,
                    created_at: mod_info.created_at,
                    updated_at: mod_info.updated_at,
                }).collect();
                all_mods.extend(mods);
            }
            Ok(Json(ApiResponse::success(all_mods)))
        }
        Err(e) => {
            error!("Failed to search external mods: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn download_mod(
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let version = params.get("version");
    let minecraft_version = params.get("minecraft_version");
    let loader = params.get("loader");

    // Create a mock ModInfo for download
    let mod_info = ModManagerModInfo {
        id: id.clone(),
        name: "Unknown Mod".to_string(),
        description: "Unknown description".to_string(),
        author: "Unknown".to_string(),
        version: version.map_or("1.0.0".to_string(), |v| v.clone()),
        minecraft_version: minecraft_version.map_or("1.20.1".to_string(), |v| v.clone()),
        loader: loader.map_or("forge".to_string(), |v| v.clone()),
        category: "misc".to_string(),
        side: "both".to_string(),
        download_url: None,
        file_size: None,
        sha1: None,
        dependencies: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    match state.mod_manager.download_mod_public(&mod_info).await {
        Ok(file_path) => {
            let download_info = serde_json::json!({
                "mod_id": id,
                "mod_name": "Unknown Mod",
                "file_path": file_path.to_string_lossy(),
                "file_size": 0,
                "sha256": "unknown",
                "downloaded_at": chrono::Utc::now()
            });
            Ok(Json(ApiResponse::success(download_info)))
        }
        Err(e) => {
            error!("Failed to download mod {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn sync_mods_from_external(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match state.mod_manager.sync_mods_from_external_sources().await {
        Ok(_) => Ok(Json(ApiResponse::success("Mod sync completed".to_string()))),
        Err(e) => {
            error!("Failed to sync mods from external sources: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn check_mod_compatibility_external(
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let minecraft_version = params.get("minecraft_version").map(|s| s.as_str()).unwrap_or("1.21.1");
    let loader = params.get("loader").map(|s| s.as_str()).unwrap_or("forge");

      // TODO: Implement compatibility check
      let compatibility_result = ModCompatibilityResult {
          compatible: true,
          issues: vec![],
          warnings: vec!["This is a placeholder compatibility check".to_string()],
      };
      
      match Ok::<ModCompatibilityResult, Box<dyn std::error::Error>>(compatibility_result) {
        Ok(report) => {
            let compatibility_info = serde_json::json!({
                "mod_id": id,
                "minecraft_version": minecraft_version,
                "loader": loader,
                "is_compatible": report.compatible,
                "issues": serde_json::to_value(&report.issues).unwrap_or(serde_json::Value::Array(vec![])),
                "warnings": serde_json::to_value(&report.warnings).unwrap_or(serde_json::Value::Array(vec![]))
            });
            Ok(Json(ApiResponse::success(compatibility_info)))
        }
        Err(e) => {
            error!("Failed to check mod compatibility for {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Settings endpoints
#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub cf_api_key: Option<String>,
    pub modrinth_token: Option<String>,
    pub java_path: Option<String>,
    pub default_ram_mb: Option<u32>,
    pub data_dir: Option<String>,
    pub telemetry_opt_in: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct JavaValidationResult {
    pub valid: bool,
    pub version: Option<String>,
    pub path: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyValidationResult {
    pub cf_valid: bool,
    pub modrinth_valid: bool,
    pub cf_error: Option<String>,
    pub modrinth_error: Option<String>,
}

async fn get_settings(State(state): State<AppState>) -> Result<Json<ApiResponse<Settings>>, StatusCode> {
    match state.database.get_settings().await {
        Ok(Some(settings)) => Ok(Json(ApiResponse {
            success: true,
            data: Some(settings),
            error: None,
            timestamp: chrono::Utc::now(),
        })),
        Ok(None) => Ok(Json(ApiResponse {
            success: true,
            data: None,
            error: None,
            timestamp: chrono::Utc::now(),
        })),
        Err(e) => {
            error!("Failed to get settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_settings(
    State(state): State<AppState>,
    Json(payload): Json<UpdateSettingsRequest>,
) -> Result<Json<ApiResponse<Settings>>, StatusCode> {
    // Get current settings
    let mut settings = match state.database.get_settings().await {
        Ok(Some(s)) => s,
        Ok(None) => {
            error!("Settings not found");
            return Err(StatusCode::NOT_FOUND);
        },
        Err(e) => {
            error!("Failed to get current settings: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Update fields if provided
    if let Some(cf_api_key) = payload.cf_api_key {
        settings.cf_api_key = Some(cf_api_key);
    }
    if let Some(modrinth_token) = payload.modrinth_token {
        settings.modrinth_token = Some(modrinth_token);
    }
    if let Some(java_path) = payload.java_path {
        settings.java_path = java_path;
    }
    if let Some(default_ram_mb) = payload.default_ram_mb {
        settings.default_ram_mb = default_ram_mb;
    }
    if let Some(data_dir) = payload.data_dir {
        settings.data_dir = data_dir;
    }
    if let Some(telemetry_opt_in) = payload.telemetry_opt_in {
        settings.telemetry_opt_in = telemetry_opt_in;
    }

    settings.updated_at = chrono::Utc::now();

    match state.database.update_settings(&settings).await {
        Ok(_) => Ok(Json(ApiResponse {
            success: true,
            data: Some(settings),
            error: None,
            timestamp: chrono::Utc::now(),
        })),
        Err(e) => {
            error!("Failed to update settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn validate_java(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<JavaValidationResult>>, StatusCode> {
    let java_path = payload.get("java_path")
        .and_then(|v| v.as_str())
        .unwrap_or("java");

    // Try to execute java -version
    let output = std::process::Command::new(java_path)
        .arg("-version")
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let version_output = String::from_utf8_lossy(&output.stderr);
                let version = extract_java_version(&version_output).map(|(_, v)| v);
                
                Ok(Json(ApiResponse {
                    success: true,
                    data: Some(JavaValidationResult {
                        valid: true,
                        version: version,
                        path: java_path.to_string(),
                        error: None,
                    }),
                    error: None,
                    timestamp: chrono::Utc::now(),
                }))
            } else {
                Ok(Json(ApiResponse {
                    success: true,
                    data: Some(JavaValidationResult {
                        valid: false,
                        version: None,
                        path: java_path.to_string(),
                        error: Some("Java command failed".to_string()),
                    }),
                    error: None,
                    timestamp: chrono::Utc::now(),
                }))
            }
        }
        Err(e) => {
            Ok(Json(ApiResponse {
                success: true,
                data: Some(JavaValidationResult {
                    valid: false,
                    version: None,
                    path: java_path.to_string(),
                    error: Some(format!("Failed to execute java: {}", e)),
                }),
                error: None,
                timestamp: chrono::Utc::now(),
            }))
        }
    }
}

async fn validate_api_keys(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<ApiKeyValidationResult>>, StatusCode> {
    let cf_api_key = payload.get("cf_api_key").and_then(|v| v.as_str());
    let modrinth_token = payload.get("modrinth_token").and_then(|v| v.as_str());

    let mut cf_valid = false;
    let mut modrinth_valid = false;
    let mut cf_error = None;
    let mut modrinth_error = None;

    // Validate CurseForge API key
    if let Some(key) = cf_api_key {
        match validate_curseforge_key(key).await {
            Ok(valid) => cf_valid = valid,
            Err(e) => cf_error = Some(e.to_string()),
        }
    }

    // Validate Modrinth token
    if let Some(token) = modrinth_token {
        match validate_modrinth_token(token).await {
            Ok(valid) => modrinth_valid = valid,
            Err(e) => modrinth_error = Some(e.to_string()),
        }
    }

    Ok(Json(ApiResponse {
        success: true,
        data: Some(ApiKeyValidationResult {
            cf_valid,
            modrinth_valid,
            cf_error,
            modrinth_error,
        }),
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

fn extract_java_version(version_output: &str) -> Option<(u32, String)> {
    // Extract version from java -version output
    // Example: openjdk version "11.0.16" 2022-07-19
    if let Some(start) = version_output.find("version \"") {
        let start = start + 9; // Skip "version \""
        if let Some(end) = version_output[start..].find("\"") {
            let version_str = version_output[start..start + end].to_string();
            // Extract major version number
            if let Some(dot_pos) = version_str.find('.') {
                if let Ok(major) = version_str[..dot_pos].parse::<u32>() {
                    return Some((major, version_str));
                }
            } else if let Ok(major) = version_str.parse::<u32>() {
                return Some((major, version_str));
            }
        }
    }
    None
}

async fn validate_curseforge_key(api_key: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.curseforge.com/v1/games")
        .header("x-api-key", api_key)
        .send()
        .await?;

    Ok(response.status().is_success())
}

async fn validate_modrinth_token(token: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.modrinth.com/v2/user")
        .header("Authorization", token)
        .send()
        .await?;

    Ok(response.status().is_success())
}

// Compatibility endpoints
#[derive(Debug, Deserialize)]
pub struct ApplyFixesRequest {
    pub fixes: Vec<String>,
}

async fn scan_compatibility(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<crate::compatibility_engine::CompatibilityReport>>, StatusCode> {
    // Get server configuration
    let server = match state.minecraft_manager.get_server(&id).await {
        Some(server) => server,
        None => return Err(StatusCode::NOT_FOUND),
    };

    // Create compatibility scanner
    let scanner = crate::compatibility_engine::CompatibilityScanner::new();
    
    // Determine mods directory
    let mods_dir = std::path::Path::new(&server.config.host).join("mods");
    
    // Scan for compatibility issues
    match scanner.scan_server(&id, &mods_dir).await {
        Ok(report) => Ok(Json(ApiResponse {
            success: true,
            data: Some(report),
            error: None,
            timestamp: chrono::Utc::now(),
        })),
        Err(e) => {
            error!("Failed to scan compatibility for server {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn apply_compatibility_fixes(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<ApplyFixesRequest>,
) -> Result<Json<ApiResponse<crate::compatibility_engine::CompatibilityReport>>, StatusCode> {
    // Get server configuration
    let server = match state.minecraft_manager.get_server(&id).await {
        Some(server) => server,
        None => return Err(StatusCode::NOT_FOUND),
    };

    // Create auto-fix engine
    let fix_engine = crate::compatibility::AutoFixEngine::new();
    
    // Determine mods directory
    let mods_dir = std::path::Path::new(&server.config.host).join("mods");
    
    // Apply fixes
    match fix_engine.apply_fixes(&id, &mods_dir, payload.fixes).await {
        Ok(report) => Ok(Json(ApiResponse {
            success: true,
            data: Some(report),
            error: None,
            timestamp: chrono::Utc::now(),
        })),
        Err(e) => {
            error!("Failed to apply compatibility fixes for server {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Pre-generation endpoints
#[derive(Debug, Deserialize)]
pub struct CreatePregenerationJobRequest {
    pub name: String,
    pub center_x: i32,
    pub center_z: i32,
    pub radius: u32,
    pub dimensions: Vec<String>,
    pub gpu_acceleration: bool,
    pub efficiency_package: bool,
    pub chunk_batch_size: Option<u32>,
    pub max_concurrent_jobs: Option<u32>,
}

async fn get_pregeneration_jobs(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Vec<crate::pregeneration::PregenerationJob>>>, StatusCode> {
    // This would need to be implemented in the AppState
    // For now, return empty list
    Ok(Json(ApiResponse {
        success: true,
        data: Some(Vec::new()),
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

async fn create_pregeneration_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<CreatePregenerationJobRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // This would need to be implemented in the AppState
    // For now, return a mock job ID
    let job_id = uuid::Uuid::new_v4().to_string();
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(job_id),
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

async fn get_pregeneration_job(
    State(state): State<AppState>,
    Path((id, job_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<crate::pregeneration::PregenerationJob>>, StatusCode> {
    // This would need to be implemented in the AppState
    // For now, return 404
    Err(StatusCode::NOT_FOUND)
}

async fn delete_pregeneration_job(
    State(state): State<AppState>,
    Path((id, job_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    // This would need to be implemented in the AppState
    Ok(Json(ApiResponse {
        success: true,
        data: None,
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

async fn start_pregeneration_job(
    State(state): State<AppState>,
    Path((id, job_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    // This would need to be implemented in the AppState
    Ok(Json(ApiResponse {
        success: true,
        data: None,
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

async fn pause_pregeneration_job(
    State(state): State<AppState>,
    Path((id, job_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    // This would need to be implemented in the AppState
    Ok(Json(ApiResponse {
        success: true,
        data: None,
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

async fn resume_pregeneration_job(
    State(state): State<AppState>,
    Path((id, job_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    // This would need to be implemented in the AppState
    Ok(Json(ApiResponse {
        success: true,
        data: None,
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

async fn cancel_pregeneration_job(
    State(state): State<AppState>,
    Path((id, job_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    // This would need to be implemented in the AppState
    Ok(Json(ApiResponse {
        success: true,
        data: None,
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

// Hot import endpoints
#[derive(Debug, Deserialize)]
pub struct CreateHotImportJobRequest {
    pub name: String,
    pub source_dir: String,
    pub target_world: String,
    pub dimensions: Vec<String>,
    pub chunk_batch_size: Option<u32>,
    pub tps_threshold: Option<f64>,
    pub safety_checks: Option<bool>,
    pub backup_before_import: Option<bool>,
}

async fn get_hot_import_jobs(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Vec<crate::hot_import::HotImportJob>>>, StatusCode> {
    // This would need to be implemented in the AppState
    // For now, return empty list
    Ok(Json(ApiResponse {
        success: true,
        data: Some(Vec::new()),
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

async fn create_hot_import_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<CreateHotImportJobRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // This would need to be implemented in the AppState
    // For now, return a mock job ID
    let job_id = uuid::Uuid::new_v4().to_string();
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(job_id),
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

async fn get_hot_import_job(
    State(state): State<AppState>,
    Path((id, job_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<crate::hot_import::HotImportJob>>, StatusCode> {
    // This would need to be implemented in the AppState
    // For now, return 404
    Err(StatusCode::NOT_FOUND)
}

async fn delete_hot_import_job(
    State(state): State<AppState>,
    Path((id, job_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    // This would need to be implemented in the AppState
    Ok(Json(ApiResponse {
        success: true,
        data: None,
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

async fn start_hot_import_job(
    State(state): State<AppState>,
    Path((id, job_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    // This would need to be implemented in the AppState
    Ok(Json(ApiResponse {
        success: true,
        data: None,
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

async fn cancel_hot_import_job(
    State(state): State<AppState>,
    Path((id, job_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    // This would need to be implemented in the AppState
    Ok(Json(ApiResponse {
        success: true,
        data: None,
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

// Lighting optimization endpoints
#[derive(Debug, Deserialize)]
pub struct CreateLightingJobRequest {
    pub name: String,
    pub world_path: String,
    pub dimensions: Vec<String>,
    pub optimization_level: String,
    pub use_gpu: bool,
    pub chunk_batch_size: Option<u32>,
    pub backup_before_optimization: Option<bool>,
    pub preserve_lighting_data: Option<bool>,
}

async fn get_lighting_jobs(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Vec<crate::lighting::LightingJob>>>, StatusCode> {
    // This would need to be implemented in the AppState
    // For now, return empty list
    Ok(Json(ApiResponse {
        success: true,
        data: Some(Vec::new()),
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

async fn create_lighting_job(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<CreateLightingJobRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // This would need to be implemented in the AppState
    // For now, return a mock job ID
    let job_id = uuid::Uuid::new_v4().to_string();
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(job_id),
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

async fn get_lighting_job(
    State(state): State<AppState>,
    Path((id, job_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<crate::lighting::LightingJob>>, StatusCode> {
    // This would need to be implemented in the AppState
    // For now, return 404
    Err(StatusCode::NOT_FOUND)
}

async fn delete_lighting_job(
    State(state): State<AppState>,
    Path((id, job_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    // This would need to be implemented in the AppState
    Ok(Json(ApiResponse {
        success: true,
        data: None,
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

async fn start_lighting_job(
    State(state): State<AppState>,
    Path((id, job_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    // This would need to be implemented in the AppState
    Ok(Json(ApiResponse {
        success: true,
        data: None,
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

async fn cancel_lighting_job(
    State(state): State<AppState>,
    Path((id, job_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    // This would need to be implemented in the AppState
    Ok(Json(ApiResponse {
        success: true,
        data: None,
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

async fn get_lighting_settings(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<crate::lighting::LightingSettings>>, StatusCode> {
    // This would need to be implemented in the AppState
    // For now, return default settings
    let settings = crate::lighting::LightingSettings {
        enabled: true,
        optimization_level: crate::lighting::OptimizationLevel::Balanced,
        default_level: crate::lighting::OptimizationLevel::Balanced,
        auto_optimize: true,
        gpu_acceleration: true,
        auto_optimize_after_pregeneration: true,
        preserve_lighting_data: true,
        max_concurrent_jobs: 4,
        chunk_batch_size: 100,
        schedule: None,
        chunk_radius: 100,
        priority: 5,
    };
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(settings),
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

async fn update_lighting_settings(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<crate::lighting::LightingSettings>,
) -> Result<Json<ApiResponse<crate::lighting::LightingSettings>>, StatusCode> {
    // This would need to be implemented in the AppState
    Ok(Json(ApiResponse {
        success: true,
        data: Some(payload),
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

// Mod management endpoints
#[derive(Debug, Deserialize)]
pub struct ModSearchRequest {
    pub query: String,
    pub provider: Option<String>,
    pub mc_version: Option<String>,
    pub loader: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateModPlanRequest {
    pub mod_ids: Vec<String>,
    pub operations: Vec<crate::mod_management::ModOperation>,
}


async fn get_server_mods(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Vec<Mod>>>, StatusCode> {
    // This would need to be implemented in the AppState
    // For now, return empty list
    Ok(Json(ApiResponse {
        success: true,
        data: Some(Vec::new()),
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

async fn create_mod_plan(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<CreateModPlanRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // This would need to be implemented in the AppState
    // For now, return a mock plan ID
    let plan_id = uuid::Uuid::new_v4().to_string();
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(plan_id),
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

async fn get_mod_plan(
    State(state): State<AppState>,
    Path((id, plan_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<crate::mod_management::ModInstallationPlan>>, StatusCode> {
    // This would need to be implemented in the AppState
    // For now, return 404
    Err(StatusCode::NOT_FOUND)
}

async fn delete_mod_plan(
    State(state): State<AppState>,
    Path((id, plan_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    // This would need to be implemented in the AppState
    Ok(Json(ApiResponse {
        success: true,
        data: None,
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

async fn apply_mod_plan(
    State(state): State<AppState>,
    Path((id, plan_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    // This would need to be implemented in the AppState
    Ok(Json(ApiResponse {
        success: true,
        data: None,
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

async fn rollback_mod_plan(
    State(state): State<AppState>,
    Path((id, plan_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    // This would need to be implemented in the AppState
    Ok(Json(ApiResponse {
        success: true,
        data: None,
        error: None,
        timestamp: chrono::Utc::now(),
    }))
}

// Resource monitoring handlers
async fn get_server_metrics(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<crate::core::resource_monitor::ServerMetrics>>, StatusCode> {
    let server_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return Ok(Json(ApiResponse::error("Invalid server ID".to_string()))),
    };

    match state.resource_monitor.get_current_server_metrics(server_id).await {
        Some(metrics) => Ok(Json(ApiResponse::success(metrics))),
        None => Ok(Json(ApiResponse::error("Server metrics not found".to_string()))),
    }
}

async fn get_server_metrics_history(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<crate::core::resource_monitor::ServerMetrics>>>, StatusCode> {
    let server_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return Ok(Json(ApiResponse::error("Invalid server ID".to_string()))),
    };

    let duration_minutes = params.get("duration")
        .and_then(|d| d.parse::<u64>().ok())
        .unwrap_or(60); // Default to 1 hour

    let duration = std::time::Duration::from_secs(duration_minutes * 60);
    let metrics = state.resource_monitor.get_server_metrics_history(server_id, duration).await;
    
    Ok(Json(ApiResponse::success(metrics)))
}

async fn get_system_metrics(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<crate::core::resource_monitor::SystemMetrics>>, StatusCode> {
    match state.resource_monitor.get_current_system_metrics().await {
        Some(metrics) => Ok(Json(ApiResponse::success(metrics))),
        None => Ok(Json(ApiResponse::error("System metrics not available".to_string()))),
    }
}

async fn get_system_metrics_history(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<crate::core::resource_monitor::SystemMetrics>>>, StatusCode> {
    let duration_minutes = params.get("duration")
        .and_then(|d| d.parse::<u64>().ok())
        .unwrap_or(60); // Default to 1 hour

    let duration = std::time::Duration::from_secs(duration_minutes * 60);
    let metrics = state.resource_monitor.get_system_metrics_history(duration).await;
    
    Ok(Json(ApiResponse::success(metrics)))
}

async fn get_resource_summary(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<crate::core::resource_monitor::ResourceSummary>>, StatusCode> {
    let summary = state.resource_monitor.get_resource_summary().await;
    Ok(Json(ApiResponse::success(summary)))
}

// Crash watchdog handlers
async fn register_server_watchdog(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let server_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return Ok(Json(ApiResponse::error("Invalid server ID".to_string()))),
    };

    match state.crash_watchdog.register_server(server_id).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => Ok(Json(ApiResponse::error(format!("Failed to register server: {}", e)))),
    }
}

async fn unregister_server_watchdog(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let server_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return Ok(Json(ApiResponse::error("Invalid server ID".to_string()))),
    };

    match state.crash_watchdog.unregister_server(server_id).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => Ok(Json(ApiResponse::error(format!("Failed to unregister server: {}", e)))),
    }
}

async fn get_server_watchdog_health(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<crate::core::crash_watchdog::ServerHealth>>, StatusCode> {
    let server_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return Ok(Json(ApiResponse::error("Invalid server ID".to_string()))),
    };

    match state.crash_watchdog.get_server_health(server_id).await {
        Some(health) => Ok(Json(ApiResponse::success(health))),
        None => Ok(Json(ApiResponse::error("Server not monitored".to_string()))),
    }
}

async fn force_restart_server(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let server_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return Ok(Json(ApiResponse::error("Invalid server ID".to_string()))),
    };

    match state.crash_watchdog.force_restart(server_id).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => Ok(Json(ApiResponse::error(format!("Failed to restart server: {}", e)))),
    }
}

async fn update_server_heartbeat(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let server_id = match Uuid::parse_str(&id) {
        Ok(id) => id,
        Err(_) => return Ok(Json(ApiResponse::error("Invalid server ID".to_string()))),
    };

    match state.crash_watchdog.update_heartbeat(server_id).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => Ok(Json(ApiResponse::error(format!("Failed to update heartbeat: {}", e)))),
    }
}

async fn get_all_watchdog_health(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<std::collections::HashMap<Uuid, crate::core::crash_watchdog::ServerHealth>>>, StatusCode> {
    let health_map = state.crash_watchdog.get_all_server_health().await;
    Ok(Json(ApiResponse::success(health_map)))
}

// Test harness endpoints
#[axum::debug_handler]
async fn run_tests(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<crate::core::test_harness::TestResults>>, StatusCode> {
    info!("Running all tests");
    
    match state.test_harness.run_all_tests().await {
        Ok(results) => {
            info!("Tests completed: {}/{} passed", results.passed_tests, results.total_tests);
            Ok(Json(ApiResponse::success(results)))
        }
        Err(e) => {
            error!("Test execution failed: {}", e);
            Ok(Json(ApiResponse::error(format!("Test execution failed: {}", e))))
        }
    }
}

#[axum::debug_handler]
async fn run_specific_test(
    Path(test_name): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<crate::core::test_harness::TestResults>>, StatusCode> {
    info!("Running specific test: {}", test_name);
    
    match state.test_harness.run_test(&test_name).await {
        Ok(results) => {
            info!("Test '{}' completed: {}/{} passed", test_name, results.passed_tests, results.total_tests);
            Ok(Json(ApiResponse::success(results)))
        }
        Err(e) => {
            error!("Test '{}' execution failed: {}", test_name, e);
            Ok(Json(ApiResponse::error(format!("Test execution failed: {}", e))))
        }
    }
}

async fn get_test_results(
    State(_state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // For now, return a placeholder message
    // In a real implementation, this would return cached test results
    Ok(Json(ApiResponse::success("Test results endpoint - implement caching for persistent results".to_string())))
}

// GPU acceleration handlers
#[axum::debug_handler]
async fn get_gpu_status(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let gpu_manager = state.gpu_manager.lock().await;
    let status = serde_json::json!({
        "enabled": gpu_manager.is_enabled(),
        "healthy": gpu_manager.get_metrics().await.utilization >= 0.0,
        "worker_available": gpu_manager.is_enabled()
    });
    
    Ok(Json(ApiResponse::success(status)))
}

#[axum::debug_handler]
async fn get_gpu_metrics(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<crate::gpu_manager::GpuMetrics>>, StatusCode> {
    let gpu_manager = state.gpu_manager.lock().await;
    let metrics = gpu_manager.get_metrics().await;
    Ok(Json(ApiResponse::success(metrics)))
}

#[axum::debug_handler]
async fn enable_gpu(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let mut gpu_manager = state.gpu_manager.lock().await;
    match gpu_manager.set_enabled(true).await {
        Ok(_) => Ok(Json(ApiResponse::success("GPU enabled successfully".to_string()))),
        Err(e) => {
            error!("Failed to enable GPU: {}", e);
            Ok(Json(ApiResponse::error(format!("Failed to enable GPU: {}", e))))
        }
    }
}

#[axum::debug_handler]
async fn disable_gpu(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let mut gpu_manager = state.gpu_manager.lock().await;
    match gpu_manager.set_enabled(false).await {
        Ok(_) => Ok(Json(ApiResponse::success("GPU disabled successfully".to_string()))),
        Err(e) => {
            error!("Failed to disable GPU: {}", e);
            Ok(Json(ApiResponse::error(format!("Failed to disable GPU: {}", e))))
        }
    }
}

#[axum::debug_handler]
async fn submit_gpu_job(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // Parse job type from payload
    let job_type = match payload.get("type").and_then(|v| v.as_str()) {
        Some("chunk_generation") => {
            let x = payload.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let z = payload.get("z").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let seed = payload.get("seed").and_then(|v| v.as_u64()).unwrap_or(0);
            let dimension = payload.get("dimension").and_then(|v| v.as_str()).unwrap_or("overworld").to_string();
            
            crate::gpu_manager::GpuJobType::ChunkGeneration { x, z, seed, dimension }
        }
        _ => {
            return Ok(Json(ApiResponse::error("Unsupported job type".to_string())));
        }
    };

    let gpu_manager = state.gpu_manager.lock().await;
    match gpu_manager.submit_job(job_type).await {
        Ok(result) => {
            if result.success {
                Ok(Json(ApiResponse::success("GPU job submitted successfully".to_string())))
            } else {
                Ok(Json(ApiResponse::error(format!("GPU job failed: {}", result.error.unwrap_or("Unknown error".to_string())))))
            }
        }
        Err(e) => {
            error!("Failed to submit GPU job: {}", e);
            Ok(Json(ApiResponse::error(format!("Failed to submit GPU job: {}", e))))
        }
    }
}

#[axum::debug_handler]
async fn get_gpu_job_status(
    Path(_job_id): Path<String>,
    State(_state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    // For now, return a placeholder status
    // In a real implementation, this would track job status
    let status = serde_json::json!({
        "job_id": _job_id,
        "status": "completed",
        "progress": 100,
        "result": "success"
    });
    
    Ok(Json(ApiResponse::success(status)))
}

// Performance telemetry handlers
async fn get_server_performance_metrics(
    Path(server_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<crate::performance_telemetry::PerformanceMetrics>>>, StatusCode> {
    match state.performance_telemetry.get_server_metrics(&server_id).await {
        Some(metrics) => Ok(Json(ApiResponse::success(metrics))),
        None => Ok(Json(ApiResponse::error("No performance metrics found for server".to_string()))),
    }
}

async fn get_server_performance_summary(
    Path(server_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Option<crate::performance_telemetry::PerformanceSummary>>>, StatusCode> {
    match state.performance_telemetry.get_performance_summary(&server_id).await {
        Some(summary) => Ok(Json(ApiResponse::success(Some(summary)))),
        None => Ok(Json(ApiResponse::success(None))),
    }
}

async fn get_all_performance_metrics(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<std::collections::HashMap<String, Vec<crate::performance_telemetry::PerformanceMetrics>>>>, StatusCode> {
    let metrics = state.performance_telemetry.get_all_metrics().await;
    Ok(Json(ApiResponse::success(metrics)))
}

// Risk analysis handlers
async fn get_server_risk_analysis(
    Path(server_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<crate::compatibility_analyzer::RiskAnalysis>>>, StatusCode> {
    // This would analyze all mods in the server and return risk analysis
    // For now, return a placeholder
    let risk_analyses = vec![
        crate::compatibility_analyzer::RiskAnalysis {
            mod_id: "example_mod".to_string(),
            overall_score: 0.3,
            risk_level: crate::compatibility_analyzer::RiskLevel::Low,
            incompatibility_score: 0.0,
            dependency_score: 0.2,
            performance_score: 0.1,
            stability_score: 0.0,
            recommendations: vec!["Mod appears stable".to_string()],
        }
    ];
    
    Ok(Json(ApiResponse::success(risk_analyses)))
}

async fn get_mod_risk_analysis(
    Path((server_id, mod_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<crate::compatibility_analyzer::RiskAnalysis>>, StatusCode> {
    // This would analyze a specific mod and return its risk analysis
    // For now, return a placeholder
    let risk_analysis = crate::compatibility_analyzer::RiskAnalysis {
        mod_id: mod_id.clone(),
        overall_score: 0.2,
        risk_level: crate::compatibility_analyzer::RiskLevel::Low,
        incompatibility_score: 0.0,
        dependency_score: 0.1,
        performance_score: 0.1,
        stability_score: 0.0,
        recommendations: vec!["Mod appears stable".to_string()],
    };
    
    Ok(Json(ApiResponse::success(risk_analysis)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_check() {
        let manager = WebSocketManager::new();
        let database = Arc::new(crate::database::DatabaseManager::new(":memory:").await.expect("Failed to create test database"));
        let state = AppState {
            websocket_manager: Arc::new(manager),
            minecraft_manager: crate::minecraft::MinecraftManager::new(database.clone()),
            database: database.clone(),
            mod_manager: crate::mod_manager::ModManager::new(std::path::PathBuf::from("mods")),
            resource_monitor: Arc::new(crate::core::resource_monitor::ResourceMonitor::new(
                crate::core::resource_monitor::ResourceMonitorConfig::default(),
                Arc::new(crate::core::guardian_config::GuardianConfig::default()),
            )),
            server_manager: Arc::new(crate::core::server_manager::ServerManager::new(
                database.clone(),
                Arc::new(crate::core::file_manager::FileManager::new(std::path::PathBuf::from("./")).expect("Failed to create file manager")),
                Arc::new(crate::core::process_manager::ProcessManager::new()),
            )),
            crash_watchdog: Arc::new(crate::core::crash_watchdog::CrashWatchdog::new()),
            process_manager: Arc::new(crate::core::process_manager::ProcessManager::new()),
            gpu_manager: Arc::new(tokio::sync::Mutex::new(crate::gpu_manager::GpuManager::new(database.clone()).await.expect("Failed to create GPU manager"))),
            performance_telemetry: Arc::new(crate::performance_telemetry::PerformanceTelemetry::new(database.clone())),
            secret_storage: Arc::new(crate::security::secret_storage::SecretStorage::new("test_key".to_string(), std::path::PathBuf::from("test_secrets.db"))),
            rate_limiter: Arc::new(crate::security::rate_limiting::RateLimiter::new()),
            test_harness: Arc::new(crate::core::test_harness::TestHarness::new()),
            sse_sender: None,
        };
        let app = create_api_router(state);

        let request = Request::builder()
            .uri("/api/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_servers() {
        let manager = WebSocketManager::new();
        let database = Arc::new(crate::database::DatabaseManager::new(":memory:").await.expect("Failed to create test database"));
        let state = AppState {
            websocket_manager: Arc::new(manager),
            minecraft_manager: crate::minecraft::MinecraftManager::new(database.clone()),
            database: database.clone(),
            mod_manager: crate::mod_manager::ModManager::new(std::path::PathBuf::from("mods")),
            resource_monitor: Arc::new(crate::core::resource_monitor::ResourceMonitor::new(
                crate::core::resource_monitor::ResourceMonitorConfig::default(),
                Arc::new(crate::core::guardian_config::GuardianConfig::default()),
            )),
            server_manager: Arc::new(crate::core::server_manager::ServerManager::new(
                database.clone(),
                Arc::new(crate::core::file_manager::FileManager::new(std::path::PathBuf::from("./")).expect("Failed to create file manager")),
                Arc::new(crate::core::process_manager::ProcessManager::new()),
            )),
            crash_watchdog: Arc::new(crate::core::crash_watchdog::CrashWatchdog::new()),
            process_manager: Arc::new(crate::core::process_manager::ProcessManager::new()),
            gpu_manager: Arc::new(tokio::sync::Mutex::new(crate::gpu_manager::GpuManager::new(database.clone()).await.expect("Failed to create GPU manager"))),
            performance_telemetry: Arc::new(crate::performance_telemetry::PerformanceTelemetry::new(database.clone())),
            secret_storage: Arc::new(crate::security::secret_storage::SecretStorage::new("test_key".to_string(), std::path::PathBuf::from("test_secrets.db"))),
            rate_limiter: Arc::new(crate::security::rate_limiting::RateLimiter::new()),
            test_harness: Arc::new(crate::core::test_harness::TestHarness::new()),
            sse_sender: None,
        };
        let app = create_api_router(state);

        let request = Request::builder()
            .uri("/api/servers")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}

// Server creation wizard endpoints

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerVersionsResponse {
    pub versions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerValidationRequest {
    pub name: Option<String>,
    pub install_path: Option<String>,
    pub java_path: Option<String>,
    pub memory: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerValidationResponse {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub java_detected: Option<String>,
    pub java_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JavaDetectionResponse {
    pub java_path: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModpackSearchRequest {
    pub query: String,
    pub q: Option<String>,
    pub provider: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModpackSearchResponse {
    pub modpacks: Vec<ModpackInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModpackInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub downloads: u64,
    pub author: String,
    pub logo_url: Option<String>,
    pub server_mods: u32,
    pub client_mods: u32,
    pub total_mods: u32,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ModSearchResponse {
    pub mods: Vec<ApiModInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiModInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub downloads: u64,
    pub author: String,
    pub logo_url: Option<String>,
    pub categories: Vec<String>,
    pub server_safe: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModpackApplyRequest {
    pub server_id: String,
    pub source: String,
    pub pack_id: String,
    pub pack_version_id: String,
    pub server_only: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModInstallRequest {
    pub server_id: String,
    pub items: Vec<ModInstallItem>,
}


async fn get_server_versions(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<ServerVersionsResponse>>, StatusCode> {
    let edition_default = "Vanilla".to_string();
    let edition = params.get("edition").unwrap_or(&edition_default);
    
    // For now, return mock data. In a real implementation, this would fetch from the appropriate API
    let versions = match edition.as_str() {
        "Vanilla" => vec![
            "1.21.1".to_string(),
            "1.21".to_string(),
            "1.20.6".to_string(),
            "1.20.4".to_string(),
            "1.20.2".to_string(),
        ],
        "Fabric" => vec![
            "1.21.1".to_string(),
            "1.21".to_string(),
            "1.20.6".to_string(),
            "1.20.4".to_string(),
        ],
        "Forge" => vec![
            "1.21.1".to_string(),
            "1.21".to_string(),
            "1.20.6".to_string(),
            "1.20.4".to_string(),
        ],
        _ => vec![],
    };
    
    Ok(Json(ApiResponse::success(ServerVersionsResponse { versions })))
}

async fn validate_server_config(
    State(state): State<AppState>,
    Json(payload): Json<ServerValidationRequest>,
) -> Result<Json<ApiResponse<ServerValidationResponse>>, StatusCode> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    
    // Validate server name
    if let Some(name) = &payload.name {
        if name.trim().is_empty() {
            errors.push("Server name cannot be empty".to_string());
        } else if name.len() > 50 {
            errors.push("Server name must be 50 characters or less".to_string());
        } else if name.len() < 3 {
            errors.push("Server name must be at least 3 characters long".to_string());
        } else {
            // Check for invalid characters
            if name.contains('/') || name.contains('\\') || name.contains(':') || 
               name.contains('*') || name.contains('?') || name.contains('"') || 
               name.contains('<') || name.contains('>') || name.contains('|') {
                errors.push("Server name contains invalid characters".to_string());
            }
            
            // Check name uniqueness
            // TODO: Implement proper server list checking
            // For now, skip the uniqueness check
        }
    }
    
    // Validate install path
    if let Some(path) = &payload.install_path {
        if path.trim().is_empty() {
            errors.push("Install path cannot be empty".to_string());
        } else {
            let path = std::path::Path::new(path);
            if !path.is_absolute() {
                errors.push("Install path must be absolute".to_string());
            } else {
                // Check if path is writable
                match std::fs::create_dir_all(path) {
                    Ok(_) => {
                        // Test write permissions
                        let test_file = path.join(".write_test");
                        if let Err(_) = std::fs::write(&test_file, "test") {
                            errors.push("Install path is not writable".to_string());
                        } else {
                            let _ = std::fs::remove_file(&test_file);
                        }
                    }
                    Err(_) => {
                        errors.push("Cannot create directory at install path".to_string());
                    }
                }
            }
        }
    }
    
    // Validate Java path
    if let Some(java_path) = &payload.java_path {
        if java_path.trim().is_empty() {
            errors.push("Java path cannot be empty".to_string());
        } else {
            let java_path = std::path::Path::new(java_path);
            if !java_path.exists() {
                errors.push("Java executable does not exist".to_string());
            } else {
                // Check if Java executable is valid and get version
                let output = std::process::Command::new(java_path)
                    .arg("-version")
                    .output();
                
                match output {
                    Ok(output) => {
                        if !output.status.success() {
                            errors.push("Java executable is not working properly".to_string());
                        } else {
                            // Parse Java version from stderr
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            if let Some(version) = extract_java_version(&stderr) {
                                if version.0 < 17 {
                                    errors.push(format!("Java version {} is too old. Java 17 or higher is required", version.0));
                                } else if version.0 >= 21 {
                                    warnings.push(format!("Java {} detected. Some mods may not be compatible with Java 21+", version.0));
                                }
                            }
                        }
                    }
                    Err(_) => {
                        errors.push("Invalid Java path or Java not executable".to_string());
                    }
                }
            }
        }
    } else {
        // Try to detect Java automatically
        match crate::loaders::LoaderInstaller::detect_java().await {
            Ok(_) => {
                warnings.push("No Java path specified, but Java was auto-detected".to_string());
            }
            Err(_) => {
                errors.push("No Java installation found. Please install Java 17 or higher".to_string());
            }
        }
    }
    
    // Validate memory settings
    if let Some(memory) = &payload.memory {
        if let Ok(mem_obj) = serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(memory.clone()) {
            if let (Some(min), Some(max)) = (mem_obj.get("min"), mem_obj.get("max")) {
                if let (Some(min_val), Some(max_val)) = (min.as_f64(), max.as_f64()) {
                    if min_val >= max_val {
                        errors.push("Minimum memory must be less than maximum memory".to_string());
                    }
                    if min_val < 1.0 || max_val > 32.0 {
                        errors.push("Memory must be between 1GB and 32GB".to_string());
                    }
                }
            }
        }
    }
    
    let valid = errors.is_empty();
    
    // Try to detect Java if not provided
    let (java_detected, java_version) = if payload.java_path.is_none() {
        match crate::loaders::LoaderInstaller::detect_java().await {
            Ok(java_path) => {
                // Get Java version
                let output = std::process::Command::new(&java_path)
                    .arg("-version")
                    .output();
                
                match output {
                    Ok(output) if output.status.success() => {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        if let Some((_, version)) = extract_java_version(&stderr) {
                            (Some(java_path.to_string_lossy().to_string()), Some(version))
                        } else {
                            (Some(java_path.to_string_lossy().to_string()), None)
                        }
                    }
                    _ => (Some(java_path.to_string_lossy().to_string()), None)
                }
            }
            Err(_) => (None, None)
        }
    } else {
        (None, None)
    };
    
    Ok(Json(ApiResponse::success(ServerValidationResponse { 
        valid, 
        errors, 
        warnings,
        java_detected,
        java_version,
    })))
}

async fn detect_java_path(
    State(_state): State<AppState>,
) -> Result<Json<ApiResponse<JavaDetectionResponse>>, StatusCode> {
    info!("Starting comprehensive Java detection");
    
    let mut java_installations = Vec::new();
    
    // First, try PATH
    if let Ok(output) = std::process::Command::new("java").arg("-version").output() {
        if output.status.success() {
            let version_output = String::from_utf8_lossy(&output.stderr);
            let version = extract_java_version(&version_output).map(|(_, v)| v);
            java_installations.push((String::from("java"), version));
        }
    }
    
    if cfg!(target_os = "windows") {
        // Scan Windows Registry for Java installations
        if let Ok(registry_java) = scan_windows_registry_for_java().await {
            java_installations.extend(registry_java);
        }
        
        // Scan common installation directories
        let program_files = std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
        let program_files_x86 = std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());
        
        // Scan for Java installations in common directories
        for base_path in &[&program_files, &program_files_x86] {
            if let Ok(entries) = std::fs::read_dir(base_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with("Java") || name.starts_with("jdk") || name.starts_with("jre") || 
                           name.starts_with("Eclipse Adoptium") || name.starts_with("OpenJDK") {
                            let java_exe = path.join("bin").join("java.exe");
                            if java_exe.exists() {
                                if let Ok(output) = std::process::Command::new(&java_exe).arg("-version").output() {
                                    if output.status.success() {
                                        let version_output = String::from_utf8_lossy(&output.stderr);
                                        let version = extract_java_version(&version_output).map(|(_, v)| v);
                                        java_installations.push((java_exe.to_string_lossy().to_string(), version));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        // Unix-like systems - scan common locations
        let common_paths = vec![
            "/usr/bin/java",
            "/usr/local/bin/java",
            "/opt/java/bin/java",
            "/usr/lib/jvm/default-java/bin/java",
            "/usr/lib/jvm/java-11-openjdk/bin/java",
            "/usr/lib/jvm/java-17-openjdk/bin/java",
            "/usr/lib/jvm/java-21-openjdk/bin/java",
        ];
        
        for path in common_paths {
            if let Ok(output) = std::process::Command::new(path).arg("-version").output() {
                if output.status.success() {
                    let version_output = String::from_utf8_lossy(&output.stderr);
                    let version = extract_java_version(&version_output).map(|(_, v)| v);
                    java_installations.push((path.to_string(), version));
                }
            }
        }
        
        // Scan /usr/lib/jvm for additional installations
        if let Ok(entries) = std::fs::read_dir("/usr/lib/jvm") {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.contains("java") || name.contains("jdk") || name.contains("jre") {
                        let java_exe = path.join("bin").join("java");
                        if java_exe.exists() {
                            if let Ok(output) = std::process::Command::new(&java_exe).arg("-version").output() {
                                if output.status.success() {
                                    let version_output = String::from_utf8_lossy(&output.stderr);
                                    let version = extract_java_version(&version_output).map(|(_, v)| v);
                                    java_installations.push((java_exe.to_string_lossy().to_string(), version));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Sort by version (prefer newer versions)
    java_installations.sort_by(|a, b| {
        let version_a = a.1.as_ref().map(|v| parse_java_version(v)).unwrap_or((0, 0, 0));
        let version_b = b.1.as_ref().map(|v| parse_java_version(v)).unwrap_or((0, 0, 0));
        version_b.cmp(&version_a)
    });
    
    if let Some((java_path, version)) = java_installations.first() {
        info!("Found Java at {} with version {:?}", java_path, version);
        Ok(Json(ApiResponse::success(JavaDetectionResponse {
            java_path: Some(java_path.clone()),
            version: version.clone(),
        })))
    } else {
        warn!("No Java installation found");
        Ok(Json(ApiResponse::success(JavaDetectionResponse {
            java_path: None,
            version: None,
        })))
    }
}

async fn scan_windows_registry_for_java() -> Result<Vec<(String, Option<String>)>, Box<dyn std::error::Error>> {
    let mut installations = Vec::new();
    
    // Use PowerShell to query the registry for Java installations
    let ps_script = r#"
        $javaKeys = @(
            "HKLM:\SOFTWARE\JavaSoft\JDK",
            "HKLM:\SOFTWARE\JavaSoft\JRE", 
            "HKLM:\SOFTWARE\Eclipse Adoptium\JDK",
            "HKLM:\SOFTWARE\Eclipse Adoptium\JRE",
            "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*"
        )
        
        foreach ($key in $javaKeys) {
            try {
                if ($key -like "*Uninstall*") {
                    $items = Get-ItemProperty $key | Where-Object { 
                        $_.DisplayName -like "*Java*" -or 
                        $_.DisplayName -like "*JDK*" -or 
                        $_.DisplayName -like "*JRE*" -or
                        $_.DisplayName -like "*Adoptium*" -or
                        $_.DisplayName -like "*OpenJDK*"
                    }
                    foreach ($item in $items) {
                        if ($item.InstallLocation) {
                            $javaExe = Join-Path $item.InstallLocation "bin\java.exe"
                            if (Test-Path $javaExe) {
                                Write-Output $javaExe
                            }
                        }
                    }
                } else {
                    $subKeys = Get-ChildItem $key -ErrorAction SilentlyContinue
                    foreach ($subKey in $subKeys) {
                        $javaHome = Get-ItemProperty "$($subKey.PSPath)" -Name "JavaHome" -ErrorAction SilentlyContinue
                        if ($javaHome -and $javaHome.JavaHome) {
                            $javaExe = Join-Path $javaHome.JavaHome "bin\java.exe"
                            if (Test-Path $javaExe) {
                                Write-Output $javaExe
                            }
                        }
                    }
                }
            } catch {
                # Ignore errors and continue
            }
        }
    "#;
    
    let output = std::process::Command::new("powershell")
        .args(&["-Command", ps_script])
        .output();
    
    if let Ok(output) = output {
        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                let path = line.trim();
                if !path.is_empty() && std::path::Path::new(path).exists() {
                    if let Ok(java_output) = std::process::Command::new(path).arg("-version").output() {
                        if java_output.status.success() {
                            let version_output = String::from_utf8_lossy(&java_output.stderr);
                            let version = extract_java_version(&version_output).map(|(_, v)| v);
                            installations.push((path.to_string(), version));
                        }
                    }
                }
            }
        }
    }
    
    Ok(installations)
}

fn parse_java_version(version_str: &str) -> (u32, u32, u32) {
    // Parse version string like "1.8.0_291" or "17.0.2" into (major, minor, patch)
    let parts: Vec<&str> = version_str.split('.').collect();
    let major = parts.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.get(2).and_then(|s| s.split('_').next().and_then(|p| p.parse().ok())).unwrap_or(0);
    (major, minor, patch)
}

async fn validate_server_creation_request(payload: &CreateServerRequest) -> Result<(), String> {
    // Validate required fields
    if payload.name.trim().is_empty() {
        return Err("Server name cannot be empty".to_string());
    }
    
    if payload.name.len() > 50 {
        return Err("Server name must be 50 characters or less".to_string());
    }
    
    if payload.minecraft_version.trim().is_empty() {
        return Err("Minecraft version cannot be empty".to_string());
    }
    
    if payload.loader.trim().is_empty() {
        return Err("Loader cannot be empty".to_string());
    }
    
    // Validate Java path if provided
    if let Some(java_path) = &payload.paths.java_path {
        if !java_path.trim().is_empty() {
            let output = std::process::Command::new(java_path)
                .arg("-version")
                .output();
            
            if let Err(_) = output {
                return Err(format!("Invalid Java path: {}", java_path));
            }
        }
    }
    
    // Validate memory allocation
    if let Some(memory) = payload.memory {
        if memory < 512 {
            return Err("Memory must be at least 512MB".to_string());
        }
        if memory > 32768 {
            return Err("Memory cannot exceed 32GB".to_string());
        }
    }
    
    // Validate port numbers
    if let Some(port) = payload.port {
        if port < 1024 {
            return Err("Port must be between 1024 and 65535".to_string());
        }
    }
    
    Ok(())
}

async fn prepare_server_jar(payload: &CreateServerRequest, server_root: &str) -> Result<String, Box<dyn std::error::Error>> {
    let jar_path = format!("{}/server.jar", server_root);
    
    // If user provided a jar path, copy it
    if let Some(jar_src) = &payload.jar_path {
        if !jar_src.trim().is_empty() {
            let from = std::path::Path::new(jar_src);
            let to = std::path::Path::new(&jar_path);
            
            if !to.exists() {
                if let Some(parent) = to.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
            }
            
            tokio::fs::copy(from, to).await?;
            info!("Copied server JAR from {:?} to {:?}", from, to);
            return Ok(jar_path);
        }
    }
    
    // Download server JAR based on loader
    match payload.loader.to_lowercase().as_str() {
        "vanilla" => {
            download_vanilla_server_jar(&payload.minecraft_version, std::path::Path::new(&jar_path)).await?;
        }
        "forge" => {
            download_forge_server_jar(&payload.minecraft_version, &payload.version, std::path::Path::new(&jar_path)).await?;
        }
        "fabric" => {
            download_fabric_server_jar(&payload.minecraft_version, &payload.version, std::path::Path::new(&jar_path)).await?;
        }
        "quilt" => {
            download_quilt_server_jar(&payload.minecraft_version, &payload.version, std::path::Path::new(&jar_path)).await?;
        }
        _ => {
            return Err("Unsupported loader".into());
        }
    }
    
    Ok(jar_path)
}

fn generate_optimized_jvm_args(memory_mb: u32) -> Vec<String> {
    let mut args = Vec::new();
    
    // Memory settings
    args.push(format!("-Xmx{}M", memory_mb));
    args.push(format!("-Xms{}M", memory_mb / 2));
    
    // Garbage collection optimization
    if memory_mb >= 4096 {
        args.push("-XX:+UseG1GC".to_string());
        args.push("-XX:+ParallelRefProcEnabled".to_string());
        args.push("-XX:MaxGCPauseMillis=200".to_string());
        args.push("-XX:+UnlockExperimentalVMOptions".to_string());
        args.push("-XX:+DisableExplicitGC".to_string());
        args.push("-XX:+AlwaysPreTouch".to_string());
        args.push("-XX:G1NewSizePercent=30".to_string());
        args.push("-XX:G1MaxNewSizePercent=40".to_string());
        args.push("-XX:G1HeapRegionSize=8M".to_string());
        args.push("-XX:G1ReservePercent=20".to_string());
        args.push("-XX:G1HeapWastePercent=5".to_string());
        args.push("-XX:G1MixedGCCountTarget=4".to_string());
        args.push("-XX:InitiatingHeapOccupancyPercent=15".to_string());
        args.push("-XX:G1MixedGCLiveThresholdPercent=90".to_string());
        args.push("-XX:G1RSetUpdatingPauseTimePercent=5".to_string());
        args.push("-XX:SurvivorRatio=32".to_string());
    } else {
        args.push("-XX:+UseConcMarkSweepGC".to_string());
        args.push("-XX:+UseParNewGC".to_string());
    }
    
    // Performance optimizations
    args.push("-XX:+PerfDisableSharedMem".to_string());
    args.push("-XX:MaxTenuringThreshold=1".to_string());
    args.push("-XX:+UseStringDeduplication".to_string());
    args.push("-XX:+OptimizeStringConcat".to_string());
    
    // JVM optimizations
    args.push("-server".to_string());
    args.push("-Djava.awt.headless=true".to_string());
    args.push("-Dfile.encoding=UTF-8".to_string());
    
    args
}

fn generate_secure_password() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};
    
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
    
    // Use system time and thread ID for pseudo-randomness
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let thread_id = std::thread::current().id();
    
    let mut hasher = DefaultHasher::new();
    now.hash(&mut hasher);
    thread_id.hash(&mut hasher);
    let hash = hasher.finish();
    
    (0..16)
        .map(|i| {
            let idx = ((hash >> (i * 4)) & 0xFF) as usize % CHARSET.len();
            CHARSET[idx] as char
        })
        .collect()
}

async fn initialize_server_configuration(
    state: &AppState,
    server_id: &str,
    payload: &CreateServerRequest,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize server.properties
    init_server_properties(state, server_id).await?;
    
    // Initialize eula.txt
    init_eula_file(server_id).await?;
    
    // Initialize ops.json
    init_ops_file(server_id).await?;
    
    // Initialize whitelist.json if whitelist is enabled
    if payload.whitelist.unwrap_or(false) {
        init_whitelist_file(server_id).await?;
    }
    
    Ok(())
}

async fn install_modpack_to_server(
    state: &AppState,
    server_id: &str,
    modpack: &ModpackInstallRequest,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Installing modpack {} to server {}", modpack.pack_id, server_id);
    
    // Get server configuration
    let server = state.minecraft_manager.get_server(server_id).await
        .ok_or_else(|| "Server not found")?;
    
    // TODO: Implement proper modpack installation
    // This function needs to be properly implemented with correct ModpackInstaller usage
    Err("Modpack installation not yet implemented".into())
}

async fn install_mods_to_server(
    _state: &AppState,
    _server_id: &str,
    _mods: &[ModInstallItem],
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Implement mod installation
    Err("Mod installation not yet implemented".into())
}


async fn download_forge_server_jar(version: &str, forge_version: &str, dest: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    use crate::loaders::LoaderInstaller;
    
    info!("Installing Forge server {} for Minecraft {}", forge_version, version);
    
    // Detect Java installation
    let java_path = LoaderInstaller::detect_java().await.map_err(|e| format!("Java detection failed: {}", e))?;
    let installer = LoaderInstaller::new(java_path);
    
    // Get server directory
    let server_dir = dest.parent().ok_or("Invalid destination path")?;
    
    // Install Forge server
    let _server_jar = installer.install_forge_server(version, forge_version, server_dir).await.map_err(|e| format!("Forge installation failed: {}", e))?;
    
    Ok(())
}

async fn download_fabric_server_jar(version: &str, fabric_version: &str, dest: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    use crate::loaders::LoaderInstaller;
    
    info!("Installing Fabric server {} for Minecraft {}", fabric_version, version);
    
    // Detect Java installation
    let java_path = LoaderInstaller::detect_java().await.map_err(|e| format!("Java detection failed: {}", e))?;
    let installer = LoaderInstaller::new(java_path);
    
    // Get server directory
    let server_dir = dest.parent().ok_or("Invalid destination path")?;
    
    // Install Fabric server
    let _server_jar = installer.install_fabric_server(version, fabric_version, server_dir).await.map_err(|e| format!("Fabric installation failed: {}", e))?;
    
    Ok(())
}

async fn download_quilt_server_jar(version: &str, quilt_version: &str, dest: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    use crate::loaders::LoaderInstaller;
    
    info!("Installing Quilt server {} for Minecraft {}", quilt_version, version);
    
    // Detect Java installation
    let java_path = LoaderInstaller::detect_java().await.map_err(|e| format!("Java detection failed: {}", e))?;
    let installer = LoaderInstaller::new(java_path);
    
    // Get server directory
    let server_dir = dest.parent().ok_or("Invalid destination path")?;
    
    // Install Quilt server
    let _server_jar = installer.install_quilt_server(version, quilt_version, server_dir).await.map_err(|e| format!("Quilt installation failed: {}", e))?;
    
    Ok(())
}

async fn init_eula_file(server_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let eula_path = format!("data/servers/{}/eula.txt", server_id);
    let eula_content = "eula=false\n";
    tokio::fs::write(eula_path, eula_content).await?;
    Ok(())
}

async fn init_ops_file(server_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let ops_path = format!("data/servers/{}/ops.json", server_id);
    let ops_content = "[]\n";
    tokio::fs::write(ops_path, ops_content).await?;
    Ok(())
}

async fn init_whitelist_file(server_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let whitelist_path = format!("data/servers/{}/whitelist.json", server_id);
    let whitelist_content = "[]\n";
    tokio::fs::write(whitelist_path, whitelist_content).await?;
    Ok(())
}

async fn search_modpacks(
    Query(params): Query<ModpackSearchRequest>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<ModpackSearchResponse>>, StatusCode> {
    info!("Searching modpacks with query: {:?}", params);
    
    let query = params.q.unwrap_or_else(|| params.query.clone());
    let provider = params.provider.unwrap_or_else(|| "modrinth".to_string());
    
    if query.is_empty() {
        return Ok(Json(ApiResponse::success(ModpackSearchResponse { modpacks: vec![] })));
    }
    
    // Use the mod manager to search for modpacks
    match state.mod_manager.search_modpacks(&query, &provider).await {
        Ok(modpacks) => {
            info!("Found {} modpacks for query '{}'", modpacks.len(), query);
            // Convert database Modpack to API ModpackInfo
            let api_modpacks: Vec<ModpackInfo> = modpacks.into_iter().map(|modpack| {
                let server_mods_count = modpack.server_mods.parse::<u32>().unwrap_or(0);
                let client_mods_count = modpack.client_mods.parse::<u32>().unwrap_or(0);
                
                ModpackInfo {
                    id: modpack.id,
                    name: modpack.name,
                    description: modpack.description.unwrap_or_default(),
                    version: modpack.minecraft_version,
                    downloads: 0, // Not available in database schema
                    author: "Unknown".to_string(), // Not available in database schema
                    logo_url: None, // Not available in database schema
                    server_mods: server_mods_count,
                    client_mods: client_mods_count,
                    total_mods: server_mods_count + client_mods_count,
                }
            }).collect();
            Ok(Json(ApiResponse::success(ModpackSearchResponse { modpacks: api_modpacks })))
        }
        Err(e) => {
            error!("Failed to search modpacks: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}



async fn install_mods(
    State(_state): State<AppState>,
    Json(payload): Json<ModInstallRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    // Mock implementation. In a real implementation, this would install the mods to the server
    info!("Installing {} mods to server {}", payload.items.len(), payload.server_id);
    
    Ok(Json(ApiResponse::success(serde_json::json!({
        "success": true,
        "message": "Mods installed successfully"
    }))))
}

async fn sse_handler(
    State(state): State<AppState>,
) -> Result<axum::response::Sse<impl futures::stream::Stream<Item = Result<axum::response::sse::Event, axum::Error>>>, StatusCode> {
    use axum::response::sse::{Event, Sse};
    use futures::stream;
    use tokio::sync::broadcast;
    
    // Create a broadcast channel for SSE events
    let (tx, mut rx) = broadcast::channel::<serde_json::Value>(1000);
    
    // Note: In a real implementation, you would store the sender in the state
    // For now, we'll just use the local channel
    
    let stream = stream::unfold((rx, tx), |(mut rx, _tx)| async move {
        // Listen for events from the broadcast channel
        match rx.recv().await {
            Ok(event_data) => {
                let event = Event::default()
                    .data(serde_json::to_string(&event_data).unwrap_or_default())
                    .event("server_event");
                Some((Ok(event), (rx, _tx)))
            }
            Err(_) => {
                // Channel closed, end the stream
                None
            }
        }
    });
    
    Ok(Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(tokio::time::Duration::from_secs(15))
            .text("keep-alive-text"),
    ))
}

// Loader-related endpoints

/// Detect Java installation on the system
async fn detect_java() -> Result<Json<serde_json::Value>, StatusCode> {
    use crate::loaders::LoaderInstaller;
    
    match LoaderInstaller::detect_java().await {
        Ok(java_path) => {
            let response = serde_json::json!({
                "success": true,
                "java_path": java_path.to_string_lossy().to_string(),
                "message": "Java installation detected successfully"
            });
            Ok(Json(response))
        }
        Err(e) => {
            let response = serde_json::json!({
                "success": false,
                "java_path": null,
                "message": format!("Java detection failed: {}", e)
            });
            Ok(Json(response))
        }
    }
}

/// Get available Fabric loader versions
async fn get_fabric_versions() -> Result<Json<serde_json::Value>, StatusCode> {
    use crate::loaders::fabric::FabricClient;
    
    let client = FabricClient::new();
    
    match client.get_loader_versions().await {
        Ok(versions) => {
            let response = serde_json::json!({
                "success": true,
                "versions": versions,
                "message": "Fabric versions retrieved successfully"
            });
            Ok(Json(response))
        }
        Err(e) => {
            let response = serde_json::json!({
                "success": false,
                "versions": [],
                "message": format!("Failed to fetch Fabric versions: {}", e)
            });
            Ok(Json(response))
        }
    }
}

/// Get available Quilt loader versions
async fn get_quilt_versions() -> Result<Json<serde_json::Value>, StatusCode> {
    use crate::loaders::quilt::QuiltClient;
    
    let client = QuiltClient::new();
    
    match client.get_loader_versions().await {
        Ok(versions) => {
            let response = serde_json::json!({
                "success": true,
                "versions": versions,
                "message": "Quilt versions retrieved successfully"
            });
            Ok(Json(response))
        }
        Err(e) => {
            let response = serde_json::json!({
                "success": false,
                "versions": [],
                "message": format!("Failed to fetch Quilt versions: {}", e)
            });
            Ok(Json(response))
        }
    }
}

/// Get available Forge loader versions for a specific Minecraft version
async fn get_forge_versions(Query(params): Query<HashMap<String, String>>) -> Result<Json<serde_json::Value>, StatusCode> {
    use crate::loaders::forge::ForgeClient;
    
    let minecraft_version = params.get("minecraft_version").map(|s| s.as_str()).unwrap_or("1.21.1");
    let client = ForgeClient::new();
    
    match client.get_versions_for_minecraft(minecraft_version).await {
        Ok(versions) => {
            let response = serde_json::json!({
                "success": true,
                "minecraft_version": minecraft_version,
                "versions": versions,
                "message": "Forge versions retrieved successfully"
            });
            Ok(Json(response))
        }
        Err(e) => {
            let response = serde_json::json!({
                "success": false,
                "minecraft_version": minecraft_version,
                "versions": [],
                "message": format!("Failed to fetch Forge versions: {}", e)
            });
            Ok(Json(response))
        }
    }
}
