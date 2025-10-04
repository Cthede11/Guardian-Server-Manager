use axum::{routing::{get, post, patch, delete}, Router, Json, extract::{Path, Query, State}, http::StatusCode};
use tracing_subscriber::{fmt, EnvFilter};
use serde::{Deserialize, Serialize};
use tower_http::cors::{CorsLayer, Any};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;
mod routes;
mod boot;
mod metrics_collector;
mod websocket_manager;
mod console_streamer;
mod world_manager;
mod mod_manager;
mod backup_manager;

// Simple in-memory server storage
type ServerStore = Arc<Mutex<HashMap<String, ServerInfo>>>;

// Combined state for the router
#[derive(Clone)]
struct AppState {
    server_store: ServerStore,
    metrics_hub: routes::metrics::MetricsHub,
    metrics_collector: Arc<metrics_collector::MetricsCollector>,
    websocket_manager: Arc<websocket_manager::WebSocketManager>,
    console_streamer: Arc<console_streamer::ConsoleStreamer>,
    world_manager: Arc<world_manager::WorldManager>,
    mod_manager: Arc<mod_manager::ModManager>,
    backup_manager: Arc<backup_manager::BackupManager>,
}

#[tokio::main]
async fn main() {
    println!("Starting hostd...");
    let _ = fmt().with_env_filter(EnvFilter::from_default_env()).try_init();
    println!("Tracing initialized");
    
    // Initialize server storage
    let server_store: ServerStore = Arc::new(Mutex::new(HashMap::new()));

    // Try to reuse existing healthy hostd
    println!("Checking for existing hostd...");
    if let Some(h) = boot::try_attach_existing().await {
        println!("hostd already running on port {}", h.port);
        return;
    }
    println!("No existing hostd found, starting new instance");

    // Try new port discovery first, fallback to original range
    println!("Choosing port...");
    let port = match boot::choose_port().await {
        0 => 8080, // fallback to original default if port discovery fails
        p => p,
    };
    println!("Selected port: {}", port);

    // state: metrics hub etc.
    println!("Creating metrics hub...");
    let hub = routes::metrics::MetricsHub::new();
    {
        let hub2 = hub.clone();
        println!("Spawning metrics task...");
        tokio::spawn(async move {
            loop {
                hub2.push(routes::metrics::MetricsPoint { timestamp: chrono::Utc::now().timestamp_millis(), tps: 20.0, tick_p95_ms: 45.0, heap_mb: None, gpu_latency_ms: None }).await;
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });
    }
    println!("Metrics hub created");

    // Initialize metrics collector
    let metrics_collector = Arc::new(metrics_collector::MetricsCollector::new());
    
    // Start metrics collection task
    let collector_task = metrics_collector::MetricsCollectorTask::new(
        metrics_collector.clone(),
        std::time::Duration::from_secs(1),
    );
    let _task_handle = collector_task.start().await;

    // Initialize WebSocket manager
    let websocket_manager = Arc::new(websocket_manager::WebSocketManager::new());

    // Initialize console streamer
    let console_streamer = Arc::new(console_streamer::ConsoleStreamer::new(1000));

    // Initialize world manager
    let world_manager = Arc::new(world_manager::WorldManager::new(std::path::PathBuf::from("./worlds")));

    // Initialize mod manager
    let mod_manager = Arc::new(mod_manager::ModManager::new(std::path::PathBuf::from("./mods")));

    // Initialize backup manager
    let backup_manager = Arc::new(backup_manager::BackupManager::new(
        std::path::PathBuf::from("./backups"),
        std::path::PathBuf::from("./servers"),
    ));

    // Create combined app state
    let app_state = AppState {
        server_store: server_store.clone(),
        metrics_hub: hub.clone(),
        metrics_collector: metrics_collector.clone(),
        websocket_manager: websocket_manager.clone(),
        console_streamer: console_streamer.clone(),
        world_manager: world_manager.clone(),
        mod_manager: mod_manager.clone(),
        backup_manager: backup_manager.clone(),
    };

            // Initialize modpack state
            let modpack_state = routes::modpack::ModpackState::new();
            
            // core routes
            println!("Setting up routes...");
            let core = Router::new()
                // Basic server endpoints (minimal implementation with persistence)
                .route("/servers", get(get_servers_minimal))
                .route("/servers", post(create_server_minimal))
                .route("/servers/:id", get(get_server_minimal))
                .route("/servers/:id", patch(update_server_minimal))
                .route("/servers/:id", delete(delete_server_minimal))
                // Existing specialized routes (world routes will be added separately)
                .route("/servers/:id/pregen/status", get(routes::pregen::status))
                .route("/servers/:id/pregen/plan", post(routes::pregen::plan))
                .route("/servers/:id/pregen/start", post(routes::pregen::start))
                .route("/servers/:id/pregen/pause", post(routes::pregen::pause))
                .route("/servers/:id/pregen/resume", post(routes::pregen::resume))
                .route("/servers/:id/pregen/cancel", post(routes::pregen::cancel))
                .route("/servers/:id/import/scan", post(routes::import::scan))
                .route("/servers/:id/import/apply", post(routes::import::apply))
                .route("/servers/:id/metrics", get(get_server_metrics))
                .route("/servers/:id/metrics/stream", get(get_server_metrics_stream))
                .route("/servers/:id/console", get(get_console_messages))
                .route("/servers/:id/console/stream", get(get_console_stream))
                .route("/servers/:id/console/stats", get(get_console_stats))
                .route("/servers/:id/mods", get(get_server_mods))
                .route("/servers/:id/mods", post(install_server_mod))
                .route("/servers/:id/mods/:mod_id", delete(uninstall_server_mod))
                .route("/servers/:id/mods/:mod_id/toggle", post(toggle_server_mod))
                .route("/servers/:id/mods/stats", get(get_server_mod_stats))
                .route("/servers/:id/backups", get(get_server_backups))
                .route("/servers/:id/backups", post(create_server_backup))
                .route("/servers/:id/backups/:backup_id", delete(delete_server_backup))
                .route("/servers/:id/backups/:backup_id/restore", post(restore_server_backup))
                .route("/servers/:id/backups/stats", get(get_server_backup_stats))
                .route("/servers/:id/backups/cleanup", post(cleanup_server_backups))
                .route("/ws", get({
                    let ws_manager = app_state.websocket_manager.clone();
                    move |ws: axum::extract::ws::WebSocketUpgrade, State(state): State<AppState>| {
                        websocket_manager::WebSocketManager::handle_websocket(ws, State(state.websocket_manager))
                    }
                }))
                .with_state(app_state.clone());
            
            // Add modpack routes
            let modpack_router = routes::modpack::create_modpack_router(modpack_state);
            
            // Add world routes
            let world_router = Router::new()
                .route("/servers/:id/world", get(routes::world::get_world))
                .route("/servers/:id/dimensions", get(routes::world::list_dimensions))
                .with_state((*world_manager).clone());
    
    println!("Routes configured");

    let app = Router::new()
        .nest("/api", core.clone())
        .merge(core)
        .merge(modpack_router)
        .merge(world_router)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        );
    println!("App router created with CORS support");

    // Bind with retry logic
    println!("Binding to port {}...", port);
    let listener = match tokio::net::TcpListener::bind(("127.0.0.1", port)).await {
        Ok(l) => {
            println!("Successfully bound to port {}", port);
            l
        },
        Err(e) => {
            println!("Failed to bind to port {}: {:?}", port, e);
            // Port unavailable, try again
            let fallback_port = boot::choose_port().await;
            let final_port = if fallback_port == 0 { 8080 } else { fallback_port };
            println!("Trying fallback port {}...", final_port);
            tokio::net::TcpListener::bind(("127.0.0.1", final_port)).await
                .unwrap_or_else(|_| panic!("Failed to bind to any port"))
        }
    };

    let actual_port = listener.local_addr().unwrap().port();
    println!("Writing port file for port {}...", actual_port);
    boot::write_port_file(actual_port).ok();
    boot::write_pid_lock(std::process::id(), actual_port).ok();

    // Add health endpoint with actual port
    println!("Adding health endpoint...");
    let health_router = boot::health_router(actual_port, std::process::id()).await;
    let app = app.merge(health_router);

    println!("hostd listening on http://127.0.0.1:{}", actual_port);
    tracing::info!("hostd listening on http://127.0.0.1:{}", actual_port);
    axum::serve(listener, app).await.unwrap();
}

// Minimal server data structures
#[derive(Serialize, Deserialize, Clone)]
struct ServerInfo {
    id: String,
    name: String,
    status: String,
    tps: f32,
    tick_p95: f32,
    heap_mb: u32,
    players_online: u32,
    gpu_queue_ms: f32,
    last_snapshot_at: Option<String>,
    blue_green: BlueGreenInfo,
    version: Option<String>,
    max_players: Option<u32>,
    uptime: Option<u64>,
    memory_usage: Option<u32>,
    cpu_usage: Option<f32>,
    world_size: Option<u64>,
    last_backup: Option<String>,
    auto_start: Option<bool>,
    config: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone)]
struct BlueGreenInfo {
    active: String,
    candidate_healthy: bool,
}

#[derive(Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

// Minimal server endpoints
async fn get_servers_minimal(State(state): State<AppState>) -> Json<ApiResponse<Vec<ServerInfo>>> {
    let servers = state.server_store.lock().await;
    let server_list: Vec<ServerInfo> = servers.values().cloned().collect();
    
    Json(ApiResponse {
        success: true,
        data: Some(server_list),
        error: None,
    })
}

async fn create_server_minimal(
    State(state): State<AppState>,
    Json(server_data): Json<serde_json::Value>
) -> Result<Json<ApiResponse<ServerInfo>>, StatusCode> {
    // Extract server data from JSON
    let name = server_data.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unnamed Server")
        .to_string();
    
    let mc_version = server_data.get("mc_version")
        .and_then(|v| v.as_str())
        .unwrap_or("1.20.1")
        .to_string();
    
    let max_players = server_data.get("max_players")
        .and_then(|v| v.as_u64())
        .unwrap_or(20) as u32;
    
    // Create a new server with unique ID
    let server_id = Uuid::new_v4().to_string();
    let server = ServerInfo {
        id: server_id.clone(),
        name: name.clone(),
        status: "stopped".to_string(),
        tps: 20.0,
        tick_p95: 45.2,
        heap_mb: 2048,
        players_online: 0,
        gpu_queue_ms: 0.0,
        last_snapshot_at: None,
        blue_green: BlueGreenInfo {
            active: "blue".to_string(),
            candidate_healthy: false,
        },
        version: Some(mc_version),
        max_players: Some(max_players),
        uptime: None,
        memory_usage: Some(2048),
        cpu_usage: None,
        world_size: None,
        last_backup: None,
        auto_start: Some(false),
        config: None,
    };
    
            // Store the server
            {
                let mut servers = state.server_store.lock().await;
                servers.insert(server_id.clone(), server.clone());
                println!("Created and stored server: {} (ID: {})", name, server_id);
            }
            
            // Register server with metrics collector (simulate a process ID)
            // In a real implementation, this would be the actual Minecraft server process PID
            let simulated_pid = std::process::id() as u32;
            state.metrics_collector.register_server(
                server_id.clone(),
                sysinfo::Pid::from_u32(simulated_pid),
                name.clone(),
            ).await;
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(server),
        error: None,
    }))
}

async fn get_server_minimal(
    State(state): State<AppState>,
    Path(id): Path<String>
) -> Result<Json<ApiResponse<ServerInfo>>, StatusCode> {
    let servers = state.server_store.lock().await;
    
    match servers.get(&id) {
        Some(server) => Ok(Json(ApiResponse {
            success: true,
            data: Some(server.clone()),
            error: None,
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn update_server_minimal(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(server_data): Json<serde_json::Value>
) -> Result<Json<ApiResponse<ServerInfo>>, StatusCode> {
    let mut servers = state.server_store.lock().await;
    
    if let Some(server) = servers.get_mut(&id) {
        // Update server fields if provided
        if let Some(name) = server_data.get("name").and_then(|v| v.as_str()) {
            server.name = name.to_string();
        }
        if let Some(mc_version) = server_data.get("mc_version").and_then(|v| v.as_str()) {
            server.version = Some(mc_version.to_string());
        }
        if let Some(max_players) = server_data.get("max_players").and_then(|v| v.as_u64()) {
            server.max_players = Some(max_players as u32);
        }
        // Note: ServerInfo doesn't have updated_at field in this minimal implementation
        
        Ok(Json(ApiResponse {
            success: true,
            data: Some(server.clone()),
            error: None,
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[axum::debug_handler]
async fn delete_server_minimal(
    State(state): State<AppState>,
    Path(id): Path<String>
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let mut servers = state.server_store.lock().await;
    
    if servers.remove(&id).is_some() {
        println!("Deleted server: {}", id);
        
        // Unregister from metrics collector
        state.metrics_collector.unregister_server(&id).await;
        
        Ok(Json(ApiResponse {
            success: true,
            data: Some(()),
            error: None,
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// Real-time metrics endpoints
async fn get_server_metrics(
    State(state): State<AppState>,
    Path(id): Path<String>
) -> Result<Json<ApiResponse<metrics_collector::ServerMetrics>>, StatusCode> {
    match state.metrics_collector.collect_server_metrics(&id).await {
        Some(metrics) => Ok(Json(ApiResponse {
            success: true,
            data: Some(metrics),
            error: None,
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn get_server_metrics_stream(
    State(state): State<AppState>,
    Path(id): Path<String>
) -> Result<Json<ApiResponse<Vec<metrics_collector::ServerMetrics>>>, StatusCode> {
    // For now, return a single metrics point
    // In a real implementation, this would be a Server-Sent Events stream
    match state.metrics_collector.collect_server_metrics(&id).await {
        Some(metrics) => Ok(Json(ApiResponse {
            success: true,
            data: Some(vec![metrics]),
            error: None,
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// Console streaming endpoints
async fn get_console_messages(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ApiResponse<Vec<console_streamer::ConsoleMessage>>>, StatusCode> {
    // Parse filter parameters
    let filter = if params.is_empty() {
        None
    } else {
        Some(console_streamer::ConsoleFilter {
            levels: params.get("levels")
                .and_then(|s| serde_json::from_str(s).ok()),
            sources: params.get("sources")
                .and_then(|s| serde_json::from_str(s).ok()),
            tags: params.get("tags")
                .and_then(|s| serde_json::from_str(s).ok()),
            search: params.get("search").map(|s| s.clone()),
            since: params.get("since")
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            limit: params.get("limit")
                .and_then(|s| s.parse().ok()),
        })
    };

    let messages = state.console_streamer.get_history(&id, filter).await;
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(messages),
        error: None,
    }))
}

async fn get_console_stream(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Vec<console_streamer::ConsoleMessage>>>, StatusCode> {
    // For now, return recent messages
    // In a real implementation, this would be a Server-Sent Events stream
    let messages = state.console_streamer.get_history(&id, None).await;
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(messages),
        error: None,
    }))
}

async fn get_console_stats(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<console_streamer::ConsoleStats>>, StatusCode> {
    let stats = state.console_streamer.get_server_stats(&id).await;
    
    Ok(Json(ApiResponse {
        success: true,
        data: Some(stats),
        error: None,
    }))
}

// Backup management endpoints
async fn get_server_backups(
    State(state): State<AppState>,
    Path(server_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<backup_manager::BackupInfo>>>, StatusCode> {
    match state.backup_manager.get_backups(&server_id).await {
        Ok(backups) => Ok(Json(ApiResponse {
            success: true,
            data: Some(backups),
            error: None,
        })),
        Err(e) => {
            tracing::error!("Failed to get backups for server {}: {}", server_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_server_backup(
    State(state): State<AppState>,
    Path(server_id): Path<String>,
    Json(request): Json<backup_manager::CreateBackupRequest>,
) -> Result<Json<ApiResponse<backup_manager::BackupInfo>>, StatusCode> {
    match state.backup_manager.create_backup(&server_id, request).await {
        Ok(backup) => Ok(Json(ApiResponse {
            success: true,
            data: Some(backup),
            error: None,
        })),
        Err(e) => {
            tracing::error!("Failed to create backup for server {}: {}", server_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_server_backup(
    State(state): State<AppState>,
    Path((server_id, backup_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    match state.backup_manager.delete_backup(&server_id, &backup_id).await {
        Ok(_) => Ok(Json(ApiResponse {
            success: true,
            data: Some(()),
            error: None,
        })),
        Err(e) => {
            tracing::error!("Failed to delete backup {} for server {}: {}", backup_id, server_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn restore_server_backup(
    State(state): State<AppState>,
    Path((server_id, backup_id)): Path<(String, String)>,
    Json(request): Json<backup_manager::RestoreBackupRequest>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    match state.backup_manager.restore_backup(&server_id, request).await {
        Ok(_) => Ok(Json(ApiResponse {
            success: true,
            data: Some(()),
            error: None,
        })),
        Err(e) => {
            tracing::error!("Failed to restore backup {} for server {}: {}", backup_id, server_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_server_backup_stats(
    State(state): State<AppState>,
    Path(server_id): Path<String>,
) -> Result<Json<ApiResponse<backup_manager::BackupStats>>, StatusCode> {
    match state.backup_manager.get_backup_stats(&server_id).await {
        Ok(stats) => Ok(Json(ApiResponse {
            success: true,
            data: Some(stats),
            error: None,
        })),
        Err(e) => {
            tracing::error!("Failed to get backup stats for server {}: {}", server_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn cleanup_server_backups(
    State(state): State<AppState>,
    Path(server_id): Path<String>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<backup_manager::CleanupResult>>, StatusCode> {
    let retention_days = request.get("retention_days")
        .and_then(|v| v.as_u64())
        .unwrap_or(30) as u32;

    match state.backup_manager.cleanup_old_backups(&server_id, retention_days).await {
        Ok(result) => Ok(Json(ApiResponse {
            success: true,
            data: Some(result),
            error: None,
        })),
        Err(e) => {
            tracing::error!("Failed to cleanup backups for server {}: {}", server_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Mod management endpoints
async fn get_server_mods(
    State(state): State<AppState>,
    Path(server_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<mod_manager::InstalledMod>>>, StatusCode> {
    match state.mod_manager.get_installed_mods(&server_id).await {
        Ok(mods) => Ok(Json(ApiResponse {
            success: true,
            data: Some(mods),
            error: None,
        })),
        Err(e) => {
            tracing::error!("Failed to get server mods for {}: {}", server_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn install_server_mod(
    State(state): State<AppState>,
    Path(server_id): Path<String>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<mod_manager::ModInstallationResult>>, StatusCode> {
    let mod_id = request.get("mod_id")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    
    let version = request.get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("latest");
    
    let source = request.get("source")
        .and_then(|v| v.as_str())
        .unwrap_or("curseforge");

    match state.mod_manager.install_mod(&server_id, mod_id, version, source).await {
        Ok(result) => Ok(Json(ApiResponse {
            success: true,
            data: Some(result),
            error: None,
        })),
        Err(e) => {
            tracing::error!("Failed to install mod {} for server {}: {}", mod_id, server_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn uninstall_server_mod(
    State(state): State<AppState>,
    Path((server_id, mod_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<mod_manager::ModInstallationResult>>, StatusCode> {
    match state.mod_manager.uninstall_mod(&server_id, &mod_id).await {
        Ok(result) => Ok(Json(ApiResponse {
            success: true,
            data: Some(result),
            error: None,
        })),
        Err(e) => {
            tracing::error!("Failed to uninstall mod {} for server {}: {}", mod_id, server_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn toggle_server_mod(
    State(state): State<AppState>,
    Path((server_id, mod_id)): Path<(String, String)>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<mod_manager::ModInstallationResult>>, StatusCode> {
    let enabled = request.get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    match state.mod_manager.toggle_mod(&server_id, &mod_id, enabled).await {
        Ok(result) => Ok(Json(ApiResponse {
            success: true,
            data: Some(result),
            error: None,
        })),
        Err(e) => {
            tracing::error!("Failed to toggle mod {} for server {}: {}", mod_id, server_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_server_mod_stats(
    State(state): State<AppState>,
    Path(server_id): Path<String>,
) -> Result<Json<ApiResponse<mod_manager::ModStats>>, StatusCode> {
    match state.mod_manager.get_mod_stats(&server_id).await {
        Ok(stats) => Ok(Json(ApiResponse {
            success: true,
            data: Some(stats),
            error: None,
        })),
        Err(e) => {
            tracing::error!("Failed to get mod stats for server {}: {}", server_id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
