use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use crate::websocket::{WebSocketManager, WebSocketMessage};

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
    pub tick_p95: f64,
    pub heap_mb: u64,
    pub players_online: u32,
    pub gpu_queue_ms: f64,
    pub last_snapshot_at: Option<chrono::DateTime<chrono::Utc>>,
    pub blue_green: BlueGreenInfo,
}

/// Blue-green deployment info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueGreenInfo {
    pub active: String,
    pub candidate_healthy: bool,
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

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub websocket_manager: WebSocketManager,
    // Add other state here as needed
}

/// Create API router
pub fn create_api_router(state: AppState) -> Router {
    Router::new()
        // Server endpoints
        .route("/api/servers", get(get_servers))
        .route("/api/servers/:id", get(get_server))
        .route("/api/servers/:id/health", get(get_server_health))
        .route("/api/servers/:id/start", post(start_server))
        .route("/api/servers/:id/stop", post(stop_server))
        .route("/api/servers/:id/restart", post(restart_server))
        .route("/api/servers/:id/command", post(send_server_command))
        
        // Console endpoints
        .route("/api/servers/:id/console", get(get_console_messages))
        .route("/api/servers/:id/console", post(send_console_message))
        
        // Player endpoints
        .route("/api/servers/:id/players", get(get_players))
        .route("/api/servers/:id/players/:uuid", get(get_player))
        .route("/api/servers/:id/players/:uuid/kick", post(kick_player))
        .route("/api/servers/:id/players/:uuid/ban", post(ban_player))
        
        // World endpoints
        .route("/api/servers/:id/world/freezes", get(get_world_freezes))
        .route("/api/servers/:id/world/heatmap", get(get_world_heatmap))
        
        // Pregen endpoints
        .route("/api/servers/:id/pregen", get(get_pregen_jobs))
        .route("/api/servers/:id/pregen", post(create_pregen_job))
        .route("/api/servers/:id/pregen/:job_id", get(get_pregen_job))
        .route("/api/servers/:id/pregen/:job_id", put(update_pregen_job))
        .route("/api/servers/:id/pregen/:job_id", delete(delete_pregen_job))
        .route("/api/servers/:id/pregen/:job_id/start", post(start_pregen_job))
        .route("/api/servers/:id/pregen/:job_id/stop", post(stop_pregen_job))
        
        // Metrics endpoints
        .route("/api/servers/:id/metrics", get(get_metrics))
        .route("/api/servers/:id/metrics/realtime", get(get_realtime_metrics))
        
        // Backup endpoints
        .route("/api/servers/:id/backups", get(get_backups))
        .route("/api/servers/:id/backups", post(create_backup))
        .route("/api/servers/:id/backups/:backup_id", get(get_backup))
        .route("/api/servers/:id/backups/:backup_id/restore", post(restore_backup))
        .route("/api/servers/:id/backups/:backup_id", delete(delete_backup))
        
        // Settings endpoints
        .route("/api/servers/:id/settings", get(get_server_settings))
        .route("/api/servers/:id/settings", put(update_server_settings))
        
        // Health check endpoint
        .route("/api/health", get(health_check))
        .route("/api/status", get(get_status))
        
        .with_state(state)
}

// Server endpoints
async fn get_servers(State(state): State<AppState>) -> Result<Json<ApiResponse<Vec<ServerInfo>>>, StatusCode> {
    // TODO: Implement actual server list retrieval
    let servers = vec![
        ServerInfo {
            id: "1".to_string(),
            name: "Test Server".to_string(),
            status: "running".to_string(),
            tps: 20.0,
            tick_p95: 45.2,
            heap_mb: 2048,
            players_online: 5,
            gpu_queue_ms: 5.2,
            last_snapshot_at: Some(chrono::Utc::now()),
            blue_green: BlueGreenInfo {
                active: "blue".to_string(),
                candidate_healthy: true,
            },
        },
    ];
    
    Ok(Json(ApiResponse::success(servers)))
}

async fn get_server(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<ServerInfo>>, StatusCode> {
    // TODO: Implement actual server retrieval
    let server = ServerInfo {
        id: id.clone(),
        name: format!("Server {}", id),
        status: "running".to_string(),
        tps: 20.0,
        tick_p95: 45.2,
        heap_mb: 2048,
        players_online: 5,
        gpu_queue_ms: 5.2,
        last_snapshot_at: Some(chrono::Utc::now()),
        blue_green: BlueGreenInfo {
            active: "blue".to_string(),
            candidate_healthy: true,
        },
    };
    
    Ok(Json(ApiResponse::success(server)))
}

async fn get_server_health(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<ServerHealth>>, StatusCode> {
    // TODO: Implement actual health check
    let health = ServerHealth {
        rcon: true,
        query: true,
        crash_tickets: 0,
        freeze_tickets: 0,
    };
    
    Ok(Json(ApiResponse::success(health)))
}

async fn start_server(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Starting server: {}", id);
    
    // TODO: Implement actual server start
    // Broadcast status update
    let message = WebSocketMessage::ServerStatus {
        server_id: id.clone(),
        status: "starting".to_string(),
        timestamp: chrono::Utc::now(),
    };
    
    if let Err(e) = state.websocket_manager.broadcast_to_server(&id, message).await {
        error!("Failed to broadcast server start: {}", e);
    }
    
    Ok(Json(ApiResponse::success("Server starting".to_string())))
}

async fn stop_server(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Stopping server: {}", id);
    
    // TODO: Implement actual server stop
    // Broadcast status update
    let message = WebSocketMessage::ServerStatus {
        server_id: id.clone(),
        status: "stopping".to_string(),
        timestamp: chrono::Utc::now(),
    };
    
    if let Err(e) = state.websocket_manager.broadcast_to_server(&id, message).await {
        error!("Failed to broadcast server stop: {}", e);
    }
    
    Ok(Json(ApiResponse::success("Server stopping".to_string())))
}

async fn restart_server(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Restarting server: {}", id);
    
    // TODO: Implement actual server restart
    Ok(Json(ApiResponse::success("Server restarting".to_string())))
}

async fn send_server_command(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(request): Json<ServerCommandRequest>,
) -> Result<Json<ApiResponse<ServerCommandResponse>>, StatusCode> {
    info!("Sending command to server {}: {}", id, request.command);
    
    // TODO: Implement actual command execution
    let response = ServerCommandResponse {
        success: true,
        output: format!("Command executed: {}", request.command),
        error: None,
    };
    
    Ok(Json(ApiResponse::success(response)))
}

// Console endpoints
async fn get_console_messages(
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<ConsoleMessage>>>, StatusCode> {
    // TODO: Implement actual console message retrieval
    let messages = vec![
        ConsoleMessage {
            ts: chrono::Utc::now().to_rfc3339(),
            level: "info".to_string(),
            msg: "Server started successfully".to_string(),
        },
    ];
    
    Ok(Json(ApiResponse::success(messages)))
}

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
    // TODO: Implement actual player retrieval
    let players = vec![
        Player {
            uuid: "12345678-1234-1234-1234-123456789012".to_string(),
            name: "TestPlayer".to_string(),
            online: true,
            last_seen: Some(chrono::Utc::now().to_rfc3339()),
            playtime: Some(3600),
        },
    ];
    
    Ok(Json(ApiResponse::success(players)))
}

async fn get_player(
    Path((id, uuid)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Player>>, StatusCode> {
    // TODO: Implement actual player retrieval
    let player = Player {
        uuid: uuid.clone(),
        name: "TestPlayer".to_string(),
        online: true,
        last_seen: Some(chrono::Utc::now().to_rfc3339()),
        playtime: Some(3600),
    };
    
    Ok(Json(ApiResponse::success(player)))
}

async fn kick_player(
    Path((id, uuid)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Kicking player {} from server {}", uuid, id);
    
    // TODO: Implement actual player kick
    Ok(Json(ApiResponse::success("Player kicked".to_string())))
}

async fn ban_player(
    Path((id, uuid)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Banning player {} from server {}", uuid, id);
    
    // TODO: Implement actual player ban
    Ok(Json(ApiResponse::success("Player banned".to_string())))
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
    // TODO: Implement actual metrics retrieval
    let metrics = Metrics {
        tps: vec![MetricPoint {
            timestamp: chrono::Utc::now().timestamp(),
            value: 20.0,
        }],
        heap: vec![MetricPoint {
            timestamp: chrono::Utc::now().timestamp(),
            value: 2048.0,
        }],
        tick_p95: vec![MetricPoint {
            timestamp: chrono::Utc::now().timestamp(),
            value: 45.2,
        }],
        gpu_ms: vec![MetricPoint {
            timestamp: chrono::Utc::now().timestamp(),
            value: 5.2,
        }],
    };
    
    Ok(Json(ApiResponse::success(metrics)))
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
    // TODO: Implement actual backup retrieval
    let backups = vec![serde_json::json!({
        "id": "backup-1",
        "name": "Daily Backup",
        "created_at": chrono::Utc::now(),
        "size_mb": 1024,
        "status": "completed"
    })];
    
    Ok(Json(ApiResponse::success(backups)))
}

async fn create_backup(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Creating backup for server {}", id);
    
    // TODO: Implement actual backup creation
    Ok(Json(ApiResponse::success("Backup created".to_string())))
}

async fn get_backup(
    Path((id, backup_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    // TODO: Implement actual backup retrieval
    let backup = serde_json::json!({
        "id": backup_id,
        "name": "Daily Backup",
        "created_at": chrono::Utc::now(),
        "size_mb": 1024,
        "status": "completed"
    });
    
    Ok(Json(ApiResponse::success(backup)))
}

async fn restore_backup(
    Path((id, backup_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Restoring backup {} for server {}", backup_id, id);
    
    // TODO: Implement actual backup restoration
    Ok(Json(ApiResponse::success("Backup restored".to_string())))
}

async fn delete_backup(
    Path((id, backup_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    info!("Deleting backup {} from server {}", backup_id, id);
    
    // TODO: Implement actual backup deletion
    Ok(Json(ApiResponse::success("Backup deleted".to_string())))
}

// Settings endpoints
async fn get_server_settings(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    // TODO: Implement actual settings retrieval
    let settings = serde_json::json!({
        "jvm": {
            "memory": "4G",
            "gc": "G1GC"
        },
        "server": {
            "port": 25565,
            "max_players": 20
        }
    });
    
    Ok(Json(ApiResponse::success(settings)))
}

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
        "connections": state.websocket_manager.connection_count().await,
        "servers": 1,
        "timestamp": chrono::Utc::now()
    });
    
    Ok(Json(ApiResponse::success(status)))
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
