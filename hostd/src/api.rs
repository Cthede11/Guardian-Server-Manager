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
use uuid::Uuid;
use tracing::{info, warn, error, debug};
use chrono::{self, Utc};

use crate::websocket::{WebSocketManager, WebSocketMessage};
use crate::database::{ServerConfig, MinecraftVersion, LoaderVersion, ModVersion, Modpack, Settings, Mod};
use crate::mod_manager::{ModManager, ModCompatibilityResult, ModInfo};
use crate::compatibility_engine::CompatibilityIssue;

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

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
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
    pub mc_version: String,
    pub paths: ServerPaths,
    #[serde(rename = "jarPath")]
    pub jar_path: Option<String>,
    #[serde(rename = "maxPlayers")]
    pub max_players: Option<u32>,
    #[serde(rename = "pregenerationPolicy")]
    pub pregeneration_policy: Option<serde_json::Value>,
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
    pub websocket_manager: WebSocketManager,
    pub minecraft_manager: crate::minecraft::MinecraftManager,
    pub database: crate::database::DatabaseManager,
    pub mod_manager: ModManager,
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
        .route("/api/servers/:id/metrics", get(get_metrics))
        .route("/api/servers/:id/metrics/realtime", get(get_realtime_metrics))
        
        // Backup endpoints
        .route("/api/servers/:id/backups", get(get_backups))
        .route("/api/servers/:id/backups", post(create_backup))
        .route("/api/servers/:id/backups/:backup_id", get(get_backup))
        // .route("/api/servers/:id/backups/:backup_id/restore", post(restore_backup))
        .route("/api/servers/:id/backups/:backup_id", delete(delete_backup))
        
        // Settings endpoints
        .route("/api/servers/:id/settings", get(get_server_settings))
        // .route("/api/servers/:id/settings", put(update_server_settings))
        
        // Modpack endpoints
        .route("/api/modpacks/versions", get(get_minecraft_versions))
        .route("/api/modpacks/loaders", get(get_loader_versions))
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
        .route("/api/mods/search", get(search_mods))
        .route("/api/servers/:id/mods", get(get_server_mods))
        .route("/api/servers/:id/mods/plan", post(create_mod_plan))
        .route("/api/servers/:id/mods/plan/:plan_id", get(get_mod_plan).delete(delete_mod_plan))
        .route("/api/servers/:id/mods/plan/:plan_id/apply", post(apply_mod_plan))
        .route("/api/servers/:id/mods/plan/:plan_id/rollback", post(rollback_mod_plan))
        
        // Health check endpoint
        .route("/api/health", get(health_check))
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
                    tps: 20.0, // TODO: Get real TPS from monitoring
                    tick_p95: 45.2, // TODO: Get real tick data
                    heap_mb: 2048, // TODO: Get real heap usage
                    players_online: 0, // TODO: Get real player count
                    gpu_queue_ms: 0.0, // TODO: Get real GPU metrics
                    last_snapshot_at: server.last_start.map(|_| chrono::Utc::now()),
                    blue_green: BlueGreenInfo {
                        active: "blue".to_string(),
                        candidate_healthy: server.status == crate::minecraft::ServerStatus::Running,
                    },
                    version: Some(server.config.mc_version.clone()),
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
    
    // Determine server root under working directory (packaged app sets cwd to resource dir)
    let server_root = std::path::Path::new("data").join("servers").join(&server_id);
    let server_root_str = server_root.to_string_lossy().to_string();
    
    // Create server configuration - use server root as working directory
    let server_config = ServerConfig {
        id: server_id.clone(),
        name: payload.name.clone(),
        host: server_root_str.clone(),
        port: 25565,
        rcon_port: 25575,
        rcon_password: Uuid::new_v4().to_string(),
        java_path: "java".to_string(), // TODO: Make configurable
        // If user provided a jar path, copy to server root as server.jar; otherwise default name and autodetect
        server_jar: "server.jar".to_string(),
        jvm_args: "-Xmx4G -Xms2G -XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:G1HeapWastePercent=5 -XX:G1MixedGCCountTarget=4 -XX:InitiatingHeapOccupancyPercent=15 -XX:G1MixedGCLiveThresholdPercent=90 -XX:G1RSetUpdatingPauseTimePercent=5 -XX:SurvivorRatio=32 -XX:+PerfDisableSharedMem -XX:MaxTenuringThreshold=1".to_string(),
        server_args: "--nogui".to_string(),
        auto_start: false,
        auto_restart: true,
        max_players: payload.max_players.unwrap_or(20),
        mc_version: payload.mc_version.clone(),
        pregeneration_policy: payload.pregeneration_policy.unwrap_or(serde_json::json!({
            "enabled": false,
            "radius": 0,
            "dimensions": ["overworld"],
            "gpu_acceleration": true,
            "efficiency_package": false
        })),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    // Add server to Minecraft manager
    match state.minecraft_manager.add_server(server_config).await {
        Ok(_) => {
            info!("Successfully created server: {} (ID: {})", payload.name, server_id);
            
            // Create server directory structure (root with world/mods/config)
            if let Err(e) = create_server_layout(&server_root_str).await {
                error!("Failed to create server directories: {}", e);
            }
            
            // If user provided a jar path (non-empty), copy it into server root as server.jar
            if let Some(jar_src) = payload.jar_path.as_ref().filter(|p| !p.trim().is_empty()) {
                let from = std::path::Path::new(jar_src);
                let to = std::path::Path::new(&server_root_str).join("server.jar");
                if !to.exists() {
                    if let Some(parent) = to.parent() { let _ = tokio::fs::create_dir_all(parent).await; }
                }
                if let Err(e) = tokio::fs::copy(from, &to).await {
                    warn!("Failed to copy provided jar from {:?} to {:?}: {}", from, to, e);
                } else {
                    info!("Copied provided server jar to {:?}", to);
                }
            } else if payload.loader.to_lowercase() == "vanilla" {
                // Attempt to download vanilla server JAR automatically
                let dest = std::path::Path::new(&server_root_str).join("server.jar");
                if !dest.exists() {
                    match download_vanilla_server_jar(&payload.version, &dest).await {
                        Ok(_) => info!("Downloaded vanilla server jar for {}", payload.version),
                        Err(e) => warn!("Failed to download vanilla server jar: {}", e),
                    }
                }
            }
            // Initialize default server.properties with RCON enabled
            if let Err(e) = init_server_properties(&state, &server_id).await {
                warn!("Failed to initialize server.properties for {}: {}", server_id, e);
            }
            
            // Initialize server.properties with RCON enabled (best-effort)
            if let Err(e) = init_server_properties(&state, &server_id).await {
                warn!("Failed to initialize server.properties for {}: {}", server_id, e);
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
                version: Some(payload.mc_version),
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
    let client = reqwest::Client::new();
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
                version: Some("1.20.1".to_string()), // TODO: Get from server config
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
            let query_ok = std::net::TcpStream::connect_timeout(
                &format!("127.0.0.1:{}", cfg.port).parse().unwrap_or("127.0.0.1:25565".parse().unwrap()),
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
    
    match state.minecraft_manager.start_server(&id).await {
        Ok(_) => {
            info!("Successfully started server: {}", id);
            
            // Broadcast status update
            let message = WebSocketMessage::ServerStatus {
                server_id: id.clone(),
                status: "starting".to_string(),
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
    
    match state.minecraft_manager.stop_server(&id).await {
        Ok(_) => {
            info!("Successfully stopped server: {}", id);
            
            // Broadcast status update
            let message = WebSocketMessage::ServerStatus {
                server_id: id.clone(),
                status: "stopping".to_string(),
                timestamp: chrono::Utc::now(),
            };
            
            let _ = state.websocket_manager.broadcast_to_server(&id, message).await;
            
            Ok(Json(ApiResponse::success("Server stopping".to_string())))
        }
        Err(e) => {
            error!("Failed to stop server {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn restart_server(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Restarting server: {}", id);
    match state.minecraft_manager.restart_server(&id).await {
        Ok(_) => Ok(Json(ApiResponse::success("Server restarting".to_string()))),
        Err(e) => Ok(Json(ApiResponse::error(format!("Failed to restart: {}", e)))),
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
    if let Some(pregeneration_policy) = payload.pregeneration_policy {
        server.config.pregeneration_policy = pregeneration_policy;
    }

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
                version: Some("Unknown".to_string()), // TODO: Get from server
                players_online: 0, // TODO: Get from server
                max_players: Some(server.config.max_players as u32),
                uptime: server.last_start.map(|start| {
                    let now = std::time::Instant::now();
                    now.duration_since(start).as_secs()
                }),
                memory_usage: Some(0), // TODO: Get from server
                cpu_usage: Some(0.0), // TODO: Get from server
                world_size: Some(0), // TODO: Get from server
                last_backup: None, // TODO: Get from server
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
            let message = WebSocketMessage::ServerEvent {
                server_id: id.clone(),
                event: "deleted".to_string(),
                data: serde_json::json!({}),
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
        Ok(Some(mut cfg)) => {
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
    
    // TODO: Implement actual console message sending
    Ok(Json(ApiResponse::success("Message sent".to_string())))
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
                last_seen: p.last_seen.map(|t| t.to_rfc3339()),
                playtime: p.playtime,
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
                let player = Player { uuid: p.uuid, name: p.name, online: p.online, last_seen: p.last_seen.map(|t| t.to_rfc3339()), playtime: p.playtime };
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
    // TODO: Implement actual world freeze retrieval
    let freezes = vec![
        WorldFreeze {
            x: 100,
            z: 200,
            duration_ms: 1500,
            timestamp: chrono::Utc::now(),
        },
    ];
    
    Ok(Json(ApiResponse::success(freezes)))
}

async fn get_world_heatmap(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    // TODO: Implement actual world heatmap data
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
    // TODO: Implement actual pregen job retrieval
    let jobs = vec![
        PregenJob {
            id: "pregen-1".to_string(),
            region: RegionInfo {
                x: 0,
                z: 0,
                radius: 1000,
            },
            dimension: "minecraft:overworld".to_string(),
            priority: "normal".to_string(),
            status: "running".to_string(),
            progress: 45.0,
            eta: Some("2h 30m".to_string()),
            gpu_assist: true,
        },
    ];
    
    Ok(Json(ApiResponse::success(jobs)))
}

// #[axum::debug_handler]
async fn create_pregen_job(
    Path(id): Path<String>,
    Json(job): Json<PregenJob>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<PregenJob>>, StatusCode> {
    info!("Creating pregen job for server {}: {:?}", id, job);
    
    // TODO: Implement actual pregen job creation
    Ok(Json(ApiResponse::success(job)))
}

async fn get_pregen_job(
    Path((id, job_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<PregenJob>>, StatusCode> {
    // TODO: Implement actual pregen job retrieval
    let job = PregenJob {
        id: job_id.clone(),
        region: RegionInfo {
            x: 0,
            z: 0,
            radius: 1000,
        },
        dimension: "minecraft:overworld".to_string(),
        priority: "normal".to_string(),
        status: "running".to_string(),
        progress: 45.0,
        eta: Some("2h 30m".to_string()),
        gpu_assist: true,
    };
    
    Ok(Json(ApiResponse::success(job)))
}

// #[axum::debug_handler]
async fn update_pregen_job(
    Path((id, job_id)): Path<(String, String)>,
    Json(job): Json<PregenJob>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<PregenJob>>, StatusCode> {
    info!("Updating pregen job {} for server {}: {:?}", job_id, id, job);
    
    // TODO: Implement actual pregen job update
    Ok(Json(ApiResponse::success(job)))
}

async fn delete_pregen_job(
    Path((id, job_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Deleting pregen job {} from server {}", job_id, id);
    
    // TODO: Implement actual pregen job deletion
    Ok(Json(ApiResponse::success("Pregen job deleted".to_string())))
}

async fn start_pregen_job(
    Path((id, job_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Starting pregen job {} on server {}", job_id, id);
    
    // TODO: Implement actual pregen job start
    Ok(Json(ApiResponse::success("Pregen job started".to_string())))
}

async fn stop_pregen_job(
    Path((id, job_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Stopping pregen job {} on server {}", job_id, id);
    
    // TODO: Implement actual pregen job stop
    Ok(Json(ApiResponse::success("Pregen job stopped".to_string())))
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
    // TODO: Implement actual realtime metrics retrieval
    get_metrics(Path(id), State(state)).await
}

// Backup endpoints
async fn get_backups(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<serde_json::Value>>>, StatusCode> {
    // List local backups in data/backups
    let backup_dir = std::path::Path::new("data").join("backups");
    let mut result = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&backup_dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if let Some(name) = p.file_stem().and_then(|s| s.to_str()) {
                if p.extension().and_then(|e| e.to_str()) == Some("zip") {
                    let size_mb = p.metadata().map(|m| (m.len() / (1024*1024)) as u64).unwrap_or(0);
                    result.push(serde_json::json!({
                        "id": name,
                        "name": name,
                        "created_at": chrono::Utc::now(),
                        "size_mb": size_mb,
                        "status": "completed"
                    }));
                }
            }
        }
    }
    Ok(Json(ApiResponse::success(result)))
}

async fn create_backup(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Creating backup for server {}", id);
    let server_root = format!("data/servers/{}", id);
    let backup_dir = std::path::Path::new("data").join("backups");
    let config = crate::backup::BackupConfig {
        strategy: crate::backup::BackupStrategy::Full,
        retention: crate::backup::RetentionPolicy::default(),
        storage: crate::backup::StorageConfig {
            local_path: backup_dir.clone(),
            remote: None,
            compression_level: 6,
            encryption_enabled: false,
            encryption_key: None,
        },
        schedule: "manual".to_string(),
        enabled: true,
        include_paths: vec![std::path::PathBuf::from(server_root)],
        exclude_paths: vec![],
        max_size_bytes: 0,
        compression_threads: 2,
    };
    let manager = crate::backup::BackupManager::new(config);
    if let Err(e) = manager.start().await { warn!("backup manager start: {}", e); }
    match manager.create_backup().await {
        Ok(r) => Ok(Json(ApiResponse::success(r.backup_id))),
        Err(e) => Ok(Json(ApiResponse::error(format!("Failed to create backup: {}", e)))),
    }
}

async fn get_backup(
    Path((id, backup_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let backup_dir = std::path::Path::new("data").join("backups");
    let path = backup_dir.join(format!("{}.zip", backup_id));
    if !path.exists() { return Ok(Json(ApiResponse::error("Backup not found".to_string()))); }
    let size_mb = path.metadata().map(|m| (m.len() / (1024*1024)) as u64).unwrap_or(0);
    let backup = serde_json::json!({ "id": backup_id, "name": backup_id, "created_at": chrono::Utc::now(), "size_mb": size_mb, "status": "completed" });
    Ok(Json(ApiResponse::success(backup)))
}

pub async fn restore_backup(
    Path((id, backup_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Restoring backup {} for server {}", backup_id, id);
    let server_root = std::path::Path::new("data").join("servers").join(&id);
    let config = crate::backup::BackupConfig {
        strategy: crate::backup::BackupStrategy::Full,
        retention: crate::backup::RetentionPolicy::default(),
        storage: crate::backup::StorageConfig { local_path: std::path::PathBuf::from("data/backups"), remote: None, compression_level: 6, encryption_enabled: false, encryption_key: None },
        schedule: "manual".to_string(),
        enabled: true,
        include_paths: vec![server_root.clone()],
        exclude_paths: vec![],
        max_size_bytes: 0,
        compression_threads: 2,
    };
    let manager = crate::backup::BackupManager::new(config);
    match manager.restore_from_backup(&backup_id, &server_root).await {
        Ok(_) => Ok(Json(ApiResponse::success("Backup restored".to_string()))),
        Err(e) => Ok(Json(ApiResponse::error(format!("Failed to restore: {}", e)))),
    }
}

async fn delete_backup(
    Path((id, backup_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Deleting backup {} from server {}", backup_id, id);
    let manager = crate::backup::BackupManager::new(crate::backup::BackupConfig {
        strategy: crate::backup::BackupStrategy::Full,
        retention: crate::backup::RetentionPolicy::default(),
        storage: crate::backup::StorageConfig { local_path: std::path::PathBuf::from("data/backups"), remote: None, compression_level: 6, encryption_enabled: false, encryption_key: None },
        schedule: "manual".to_string(),
        enabled: true,
        include_paths: vec![],
        exclude_paths: vec![],
        max_size_bytes: 0,
        compression_threads: 2,
    });
    match manager.delete_backup(&backup_id).await {
        Ok(_) => Ok(Json(ApiResponse::success("Backup deleted".to_string()))),
        Err(e) => Ok(Json(ApiResponse::error(format!("Failed to delete: {}", e)))),
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

// Health check endpoints
async fn health_check(State(state): State<AppState>) -> Result<Json<ApiResponse<String>>, StatusCode> {
    Ok(Json(ApiResponse::success("OK".to_string())))
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
    match state.database.search_mods(&params).await {
        Ok(mods) => Ok(Json(ApiResponse::success(mods))),
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
    
    // TODO: Implement actual modpack application
    // This would involve:
    // 1. Downloading mods from Modrinth/CurseForge
    // 2. Installing server mods to the server's mods directory
    // 3. Optionally creating a client modpack for players
    
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
                "size_mb": 0, // TODO: Calculate actual size
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
                    project_id: "unknown".to_string(), // TODO: Get from mod_info
                    version_id: "unknown".to_string(), // TODO: Get from mod_info
                    filename: "unknown".to_string(), // TODO: Get from mod_info
                    sha1: "unknown".to_string(), // TODO: Get from mod_info
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
    let mod_info = ModInfo {
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
                let version = extract_java_version(&version_output);
                
                Ok(Json(ApiResponse {
                    success: true,
                    data: Some(JavaValidationResult {
                        valid: true,
                        version: Some(version),
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

fn extract_java_version(version_output: &str) -> String {
    // Extract version from java -version output
    // Example: openjdk version "11.0.16" 2022-07-19
    if let Some(start) = version_output.find("version \"") {
        let start = start + 9; // Skip "version \""
        if let Some(end) = version_output[start..].find("\"") {
            return version_output[start..start + end].to_string();
        }
    }
    "Unknown".to_string()
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
) -> Result<Json<ApiResponse<crate::compatibility::CompatibilityReport>>, StatusCode> {
    // Get server configuration
    let server = match state.minecraft_manager.get_server(&id).await {
        Some(server) => server,
        None => return Err(StatusCode::NOT_FOUND),
    };

    // Create compatibility scanner
    let scanner = crate::compatibility::CompatibilityScanner::new();
    
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
) -> Result<Json<ApiResponse<crate::compatibility::CompatibilityReport>>, StatusCode> {
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
        default_level: crate::lighting::OptimizationLevel::Balanced,
        gpu_acceleration: true,
        auto_optimize_after_pregeneration: true,
        preserve_lighting_data: true,
        max_concurrent_jobs: 4,
        chunk_batch_size: 100,
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
        let state = AppState {
            websocket_manager: manager,
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
        let state = AppState {
            websocket_manager: manager,
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
