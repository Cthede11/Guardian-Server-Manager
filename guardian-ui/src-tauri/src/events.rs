use crate::dto::*;
use tauri::{AppHandle, Emitter};

// Event emitters for real-time data streams
pub fn emit_console(app: &AppHandle, id: &str, payload: ConsoleLines) -> Result<(), String> {
    app.emit(&format!("console:{}", id), payload)
        .map_err(|e| e.to_string())
}

pub fn emit_metrics(app: &AppHandle, id: &str, payload: Metrics) -> Result<(), String> {
    app.emit(&format!("metrics:{}", id), payload)
        .map_err(|e| e.to_string())
}

pub fn emit_players(app: &AppHandle, id: &str, payload: Vec<Player>) -> Result<(), String> {
    app.emit(&format!("players:{}", id), payload)
        .map_err(|e| e.to_string())
}

pub fn emit_freezes(app: &AppHandle, id: &str, payload: Vec<FreezeTicket>) -> Result<(), String> {
    app.emit(&format!("freezes:{}", id), payload)
        .map_err(|e| e.to_string())
}

pub fn emit_pregen(app: &AppHandle, id: &str, payload: Vec<PregenJob>) -> Result<(), String> {
    app.emit(&format!("pregen:{}", id), payload)
        .map_err(|e| e.to_string())
}

pub fn emit_health(app: &AppHandle, id: &str, payload: ServerHealth) -> Result<(), String> {
    app.emit(&format!("health:{}", id), payload)
        .map_err(|e| e.to_string())
}

pub fn emit_server_summary(app: &AppHandle, id: &str, payload: ServerSummary) -> Result<(), String> {
    app.emit(&format!("server_summary:{}", id), payload)
        .map_err(|e| e.to_string())
}

pub fn emit_backups(app: &AppHandle, id: &str, payload: Vec<Snapshot>) -> Result<(), String> {
    app.emit(&format!("backups:{}", id), payload)
        .map_err(|e| e.to_string())
}

pub fn emit_mods(app: &AppHandle, id: &str, payload: Vec<ModInfo>) -> Result<(), String> {
    app.emit(&format!("mods:{}", id), payload)
        .map_err(|e| e.to_string())
}

pub fn emit_rules(app: &AppHandle, id: &str, payload: Vec<Rule>) -> Result<(), String> {
    app.emit(&format!("rules:{}", id), payload)
        .map_err(|e| e.to_string())
}

pub fn emit_conflicts(app: &AppHandle, id: &str, payload: Vec<Conflict>) -> Result<(), String> {
    app.emit(&format!("conflicts:{}", id), payload)
        .map_err(|e| e.to_string())
}

pub fn emit_events(app: &AppHandle, id: &str, payload: Vec<Event>) -> Result<(), String> {
    app.emit(&format!("events:{}", id), payload)
        .map_err(|e| e.to_string())
}

// Global events
pub fn emit_servers_list(app: &AppHandle, payload: Vec<ServerSummary>) -> Result<(), String> {
    app.emit("servers:list", payload)
        .map_err(|e| e.to_string())
}

pub fn emit_sharding_topology(app: &AppHandle, payload: ShardingTopology) -> Result<(), String> {
    app.emit("sharding:topology", payload)
        .map_err(|e| e.to_string())
}

pub fn emit_shard_assignments(app: &AppHandle, payload: Vec<ShardAssignment>) -> Result<(), String> {
    app.emit("sharding:assignments", payload)
        .map_err(|e| e.to_string())
}

// Helper function to emit a single console line
pub fn emit_console_line(app: &AppHandle, id: &str, line: ConsoleLine) -> Result<(), String> {
    emit_console(app, id, ConsoleLines { lines: vec![line] })
}

// Helper function to emit server status change
pub fn emit_server_status_change(app: &AppHandle, id: &str, status: String) -> Result<(), String> {
    app.emit(&format!("server_status:{}", id), status)
        .map_err(|e| e.to_string())
}

// Helper function to emit connection status
pub fn emit_connection_status(app: &AppHandle, connected: bool, connection_type: String) -> Result<(), String> {
    let status = serde_json::json!({
        "connected": connected,
        "type": connection_type
    });
    app.emit("connection:status", status)
        .map_err(|e| e.to_string())
}
