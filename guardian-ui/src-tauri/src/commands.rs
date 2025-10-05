use crate::dto::*;
use tauri::AppHandle;
use std::collections::HashMap;
use reqwest;
use log;
use serde_json;

// API Response wrapper to match backend
#[derive(serde::Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
    timestamp: String,
}

// Helper function to make API calls to the backend
async fn make_api_call<T>(endpoint: &str, method: &str, body: Option<serde_json::Value>) -> Result<T, String> 
where
    T: serde::de::DeserializeOwned,
{
    let base_url = get_backend_url().await?;
    let url = format!("{}{}", base_url, endpoint);
    
    let client = reqwest::Client::new();
    let mut request = match method.to_uppercase().as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        "PATCH" => client.patch(&url),
        _ => return Err("Unsupported HTTP method".to_string()),
    };
    
    if let Some(body_data) = body {
        request = request.header("Content-Type", "application/json").json(&body_data);
    }
    
    let response = request.send().await
        .map_err(|e| format!("HTTP request failed: {}", e))?;
    
    let status = response.status();
    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("HTTP {}: {}", status, error_text));
    }
    
    let response_text = response.text().await
        .map_err(|e| format!("Failed to read response: {}", e))?;
    
    // Parse the API response wrapper
    let api_response: ApiResponse<T> = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    if api_response.success {
        api_response.data.ok_or_else(|| "No data in successful response".to_string())
    } else {
        Err(api_response.error.unwrap_or_else(|| "Unknown API error".to_string()))
    }
}

// HTTP request command for frontend
#[tauri::command]
pub async fn make_http_request(url: String, method: String, body: Option<String>) -> Result<String, String> {
    log::info!("Tauri HTTP command called: {} {}", method, url);
    
    let client = reqwest::Client::new();
    
    let mut request = match method.to_uppercase().as_str() {
        "GET" => client.get(&url),
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        "PATCH" => client.patch(&url),
        _ => return Err("Unsupported HTTP method".to_string()),
    };
    
    if let Some(body_data) = body {
        request = request.header("Content-Type", "application/json").body(body_data);
    }
    
    log::info!("Sending HTTP request to: {}", url);
    
    let response = request.send().await
        .map_err(|e| {
            log::error!("HTTP request failed: {}", e);
            format!("HTTP request failed: {}", e)
        })?;
    
    let status = response.status();
    log::info!("HTTP response status: {}", status);
    
    let response_text = response.text().await
        .map_err(|e| {
            log::error!("Failed to read response: {}", e);
            format!("Failed to read response: {}", e)
        })?;
    
    log::info!("HTTP response body: {}", response_text);
    
    if !status.is_success() {
        log::error!("HTTP error {}: {}", status, response_text);
        return Err(format!("HTTP {}: {}", status, response_text));
    }
    
    Ok(response_text)
}

// Server management commands
#[tauri::command]
pub async fn get_server_summary(id: String) -> Result<ServerSummary, String> {
    make_api_call::<ServerSummary>(&format!("/servers/{}", id), "GET", None).await
}

#[tauri::command]
pub async fn get_servers() -> Result<Vec<ServerSummary>, String> {
    make_api_call::<Vec<ServerSummary>>("/servers", "GET", None).await
}

#[tauri::command]
pub async fn create_server(data: CreateServerRequest) -> Result<ServerSummary, String> {
    let body = serde_json::to_value(data).map_err(|e| format!("Failed to serialize request: {}", e))?;
    make_api_call::<ServerSummary>("/servers", "POST", Some(body)).await
}

#[tauri::command]
pub async fn delete_server(id: String) -> Result<(), String> {
    make_api_call::<()>(&format!("/servers/{}", id), "DELETE", None).await
}

// Server control commands
#[tauri::command]
pub async fn start_server(id: String) -> Result<(), String> {
    make_api_call::<()>(&format!("/servers/{}/start", id), "POST", None).await
}

#[tauri::command]
pub async fn stop_server(id: String) -> Result<(), String> {
    make_api_call::<()>(&format!("/servers/{}/stop", id), "POST", None).await
}

#[tauri::command]
pub async fn restart_server(id: String) -> Result<(), String> {
    make_api_call::<()>(&format!("/servers/{}/restart", id), "POST", None).await
}

#[tauri::command]
pub async fn promote_server(id: String) -> Result<(), String> {
    make_api_call::<()>(&format!("/servers/{}/promote", id), "POST", None).await
}

// Console and commands
#[tauri::command]
pub async fn send_rcon(id: String, cmd: String) -> Result<(), String> {
    let body = serde_json::json!({ "command": cmd });
    make_api_call::<()>(&format!("/servers/{}/command", id), "POST", Some(body)).await
}

#[tauri::command]
pub async fn get_console_messages(id: String) -> Result<Vec<ConsoleLine>, String> {
    make_api_call::<Vec<ConsoleLine>>(&format!("/servers/{}/console", id), "GET", None).await
}

// Server health and metrics
#[tauri::command]
pub async fn get_server_health(id: String) -> Result<ServerHealth, String> {
    make_api_call::<ServerHealth>(&format!("/servers/{}/health", id), "GET", None).await
}

#[tauri::command]
pub async fn get_players(id: String) -> Result<Vec<Player>, String> {
    make_api_call::<Vec<Player>>(&format!("/servers/{}/players", id), "GET", None).await
}

#[tauri::command]
pub async fn get_metrics(id: String) -> Result<Metrics, String> {
    make_api_call::<Metrics>(&format!("/servers/{}/metrics", id), "GET", None).await
}

// Player actions
#[tauri::command]
pub async fn kick_player(id: String, player_uuid: String) -> Result<(), String> {
    make_api_call::<()>(&format!("/servers/{}/players/{}/kick", id, player_uuid), "POST", None).await
}

#[tauri::command]
pub async fn ban_player(id: String, player_uuid: String) -> Result<(), String> {
    make_api_call::<()>(&format!("/servers/{}/players/{}/ban", id, player_uuid), "POST", None).await
}

// Backups
#[tauri::command]
pub async fn get_backups(id: String) -> Result<Vec<Snapshot>, String> {
    make_api_call::<Vec<Snapshot>>(&format!("/servers/{}/backups", id), "GET", None).await
}

#[tauri::command]
pub async fn create_backup(id: String, name: String) -> Result<Snapshot, String> {
    let body = serde_json::json!({ "name": name });
    make_api_call::<Snapshot>(&format!("/servers/{}/backups", id), "POST", Some(body)).await
}

#[tauri::command]
pub async fn delete_backup(id: String, snapshot_id: String) -> Result<(), String> {
    make_api_call::<()>(&format!("/servers/{}/backups/{}", id, snapshot_id), "DELETE", None).await
}

#[tauri::command]
pub async fn restore_backup(id: String, snapshot_id: String) -> Result<(), String> {
    make_api_call::<()>(&format!("/servers/{}/backups/{}/restore", id, snapshot_id), "POST", None).await
}

// World management
#[tauri::command]
pub async fn get_freeze_tickets(id: String) -> Result<Vec<FreezeTicket>, String> {
    make_api_call::<Vec<FreezeTicket>>(&format!("/servers/{}/world/freezes", id), "GET", None).await
}

#[tauri::command]
pub async fn thaw_world(id: String, ticket_id: String) -> Result<(), String> {
    make_api_call::<()>(&format!("/servers/{}/world/thaw/{}", id, ticket_id), "POST", None).await
}

// Pregen jobs
#[tauri::command]
pub async fn get_pregen_jobs(id: String) -> Result<Vec<PregenJob>, String> {
    make_api_call::<Vec<PregenJob>>(&format!("/servers/{}/pregen", id), "GET", None).await
}

#[tauri::command]
pub async fn create_pregen_job(id: String, job: CreatePregenJobRequest) -> Result<PregenJob, String> {
    let body = serde_json::to_value(job).map_err(|e| format!("Failed to serialize request: {}", e))?;
    make_api_call::<PregenJob>(&format!("/servers/{}/pregen", id), "POST", Some(body)).await
}

#[tauri::command]
pub async fn start_pregen_job(id: String, job_id: String) -> Result<(), String> {
    make_api_call::<()>(&format!("/servers/{}/pregen/{}/start", id, job_id), "POST", None).await
}

#[tauri::command]
pub async fn stop_pregen_job(id: String, job_id: String) -> Result<(), String> {
    make_api_call::<()>(&format!("/servers/{}/pregen/{}/stop", id, job_id), "POST", None).await
}

#[tauri::command]
pub async fn delete_pregen_job(id: String, job_id: String) -> Result<(), String> {
    make_api_call::<()>(&format!("/servers/{}/pregen/{}", id, job_id), "DELETE", None).await
}

// Mods and rules
#[tauri::command]
pub async fn get_mods(id: String) -> Result<Vec<ModInfo>, String> {
    make_api_call::<Vec<ModInfo>>(&format!("/servers/{}/mods", id), "GET", None).await
}

#[tauri::command]
pub async fn get_rules(id: String) -> Result<Vec<Rule>, String> {
    make_api_call::<Vec<Rule>>(&format!("/servers/{}/rules", id), "GET", None).await
}

#[tauri::command]
pub async fn get_conflicts(id: String) -> Result<Vec<Conflict>, String> {
    make_api_call::<Vec<Conflict>>(&format!("/servers/{}/conflicts", id), "GET", None).await
}

// Settings
#[tauri::command]
pub async fn get_server_settings(id: String) -> Result<ServerSettings, String> {
    make_api_call::<ServerSettings>(&format!("/servers/{}/settings", id), "GET", None).await
}

#[tauri::command]
pub async fn update_server_settings(id: String, settings: ServerSettings) -> Result<ServerSettings, String> {
    let body = serde_json::to_value(settings).map_err(|e| format!("Failed to serialize request: {}", e))?;
    make_api_call::<ServerSettings>(&format!("/servers/{}/settings", id), "PUT", Some(body)).await
}

// Sharding
#[tauri::command]
pub async fn get_sharding_topology() -> Result<ShardingTopology, String> {
    make_api_call::<ShardingTopology>("/sharding/topology", "GET", None).await
}

#[tauri::command]
pub async fn get_shard_assignments() -> Result<Vec<ShardAssignment>, String> {
    make_api_call::<Vec<ShardAssignment>>("/sharding/assignments", "GET", None).await
}

// Events
#[tauri::command]
pub async fn get_events(id: String) -> Result<Vec<Event>, String> {
    make_api_call::<Vec<Event>>(&format!("/servers/{}/events", id), "GET", None).await
}

#[tauri::command]
pub async fn create_event(id: String, event: CreateEventRequest) -> Result<Event, String> {
    let body = serde_json::to_value(event).map_err(|e| format!("Failed to serialize request: {}", e))?;
    make_api_call::<Event>(&format!("/servers/{}/events", id), "POST", Some(body)).await
}

// GPU status command
#[tauri::command]
pub async fn get_gpu_status() -> Result<GpuStatus, String> {
    make_api_call::<GpuStatus>("/gpu/status", "GET", None).await
}

// Request types for commands
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateServerRequest {
    pub name: String,
    pub version: String,
    pub max_players: Option<u32>,
    pub memory: Option<u32>,
    pub paths: PathSettings,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreatePregenJobRequest {
    pub region: Region,
    pub dimension: String,
    pub priority: String,
    pub gpu_assist: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateEventRequest {
    pub name: String,
    pub description: String,
    pub scheduled_at: String,
    pub actions: Vec<String>,
}

// Backend connection command
#[tauri::command]
pub async fn get_backend_url() -> Result<String, String> {
    log::info!("get_backend_url command called");
    // Try to find existing healthy backend first
    for port in 52100..=52150 {
        let url = format!("http://127.0.0.1:{}/healthz", port);
        log::info!("Trying port: {}", port);
        if let Ok(resp) = reqwest::get(&url).await {
            log::info!("Response status: {}", resp.status());
            if resp.status().is_success() {
                let result = format!("http://127.0.0.1:{}", port);
                log::info!("Found healthy backend at: {}", result);
                return Ok(result);
            }
        } else {
            log::info!("Failed to connect to port: {}", port);
        }
    }
    log::error!("No healthy backend found in port range 52100-52150");
    Err("No healthy backend found".to_string())
}