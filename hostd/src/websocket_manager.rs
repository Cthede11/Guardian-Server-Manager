use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{Duration, Instant};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    /// Console message from server
    ConsoleMessage {
        server_id: String,
        timestamp: DateTime<Utc>,
        level: String,
        message: String,
    },
    /// Server metrics update
    MetricsUpdate {
        server_id: String,
        timestamp: DateTime<Utc>,
        tps: f64,
        tick_p95_ms: f64,
        heap_mb: f64,
        players_online: u32,
        memory_usage_mb: f64,
        cpu_usage_percent: f64,
    },
    /// Player join/leave event
    PlayerEvent {
        server_id: String,
        timestamp: DateTime<Utc>,
        event_type: String, // join, leave, kick, ban
        player_name: String,
        player_uuid: String,
        reason: Option<String>,
    },
    /// Server status change
    ServerStatusChange {
        server_id: String,
        timestamp: DateTime<Utc>,
        old_status: String,
        new_status: String,
    },
    /// World freeze event
    WorldFreeze {
        server_id: String,
        timestamp: DateTime<Utc>,
        x: i32,
        z: i32,
        duration_ms: u64,
    },
    /// Pregen progress update
    PregenProgress {
        server_id: String,
        timestamp: DateTime<Utc>,
        job_id: String,
        progress: f64,
        eta_seconds: Option<u64>,
    },
    /// Health check response
    Ping {
        timestamp: DateTime<Utc>,
    },
    /// Health check response
    Pong {
        timestamp: DateTime<Utc>,
    },
    /// Error message
    Error {
        message: String,
        timestamp: DateTime<Utc>,
    },
    /// Progress event for long-running operations
    ProgressEvent {
        server_id: Option<String>,
        job_id: String,
        job_type: String, // "modpack_install", "mod_install", "server_creation", etc.
        status: String,   // "started", "in_progress", "completed", "failed"
        progress: f32,    // 0.0 to 1.0
        current_step: String,
        total_steps: u32,
        current_step_progress: f32, // 0.0 to 1.0 for current step
        message: Option<String>,
        error: Option<String>,
        estimated_remaining_ms: Option<u64>,
        timestamp: DateTime<Utc>,
    },
    /// Job started event
    JobStarted {
        server_id: Option<String>,
        job_id: String,
        job_type: String,
        total_steps: u32,
        timestamp: DateTime<Utc>,
    },
    /// Job progress event
    JobProgress {
        server_id: Option<String>,
        job_id: String,
        job_type: String,
        step: u32,
        total_steps: u32,
        progress: f32,
        message: String,
        timestamp: DateTime<Utc>,
    },
    /// Job completed event
    JobCompleted {
        server_id: Option<String>,
        job_id: String,
        job_type: String,
        result: Option<String>,
        timestamp: DateTime<Utc>,
    },
    /// Job failed event
    JobFailed {
        server_id: Option<String>,
        job_id: String,
        job_type: String,
        error: String,
        timestamp: DateTime<Utc>,
    },
}

/// WebSocket connection information
#[derive(Debug, Clone)]
pub struct WebSocketConnection {
    pub id: String,
    pub server_id: Option<String>,
    pub subscribed_events: Vec<String>,
    pub last_ping: DateTime<Utc>,
    pub last_pong: DateTime<Utc>,
    pub connection_time: Instant,
    pub is_healthy: bool,
    pub reconnect_count: u32,
    pub last_reconnect: Option<DateTime<Utc>>,
}

/// WebSocket manager for handling real-time connections
#[derive(Debug)]
pub struct WebSocketManager {
    /// Active connections
    pub connections: Arc<RwLock<HashMap<String, WebSocketConnection>>>,
    /// Broadcast channel for sending messages to all connections
    pub broadcast_tx: broadcast::Sender<WebSocketMessage>,
    /// Global sender for broadcasting messages
    pub global_sender: broadcast::Sender<WebSocketMessage>,
    /// Server-specific broadcast channels
    pub server_channels: Arc<RwLock<HashMap<String, broadcast::Sender<WebSocketMessage>>>>,
    /// Heartbeat configuration
    pub heartbeat_interval: Duration,
    pub heartbeat_timeout: Duration,
    pub max_reconnect_attempts: u32,
    pub reconnect_backoff: Duration,
}

impl WebSocketManager {
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);
        
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx: broadcast_tx.clone(),
            global_sender: broadcast_tx,
            server_channels: Arc::new(RwLock::new(HashMap::new())),
            heartbeat_interval: Duration::from_secs(30), // Send ping every 30 seconds
            heartbeat_timeout: Duration::from_secs(60),  // Consider dead if no pong for 60 seconds
            max_reconnect_attempts: 5,
            reconnect_backoff: Duration::from_secs(5),
        }
    }

    /// Handle WebSocket upgrade
    pub async fn handle_websocket(
        ws: WebSocketUpgrade,
        State(manager): State<Arc<WebSocketManager>>,
    ) -> Response {
        ws.on_upgrade(|socket| manager.handle_socket(socket))
    }

    /// Handle individual WebSocket connection
    pub async fn handle_socket(self: Arc<Self>, socket: WebSocket) {
        let connection_id = Uuid::new_v4().to_string();
        let mut rx = self.broadcast_tx.subscribe();
        
        // Register connection
        {
            let mut connections = self.connections.write().await;
            connections.insert(connection_id.clone(), WebSocketConnection {
                id: connection_id.clone(),
                server_id: None,
                subscribed_events: vec!["all".to_string()],
                last_ping: Utc::now(),
                last_pong: Utc::now(),
                connection_time: Instant::now(),
                is_healthy: true,
                reconnect_count: 0,
                last_reconnect: None,
            });
        }

        let (sender, mut receiver) = socket.split();
        let sender = Arc::new(Mutex::new(sender));

        // Spawn task to handle incoming messages
        let manager_clone = self.clone();
        let connection_id_clone = connection_id.clone();
        let sender_clone = sender.clone();
        tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Err(e) = manager_clone.handle_message(&connection_id_clone, &text).await {
                            tracing::error!("Error handling WebSocket message: {}", e);
                        }
                    }
                    Ok(Message::Ping(payload)) => {
                        let mut sender_guard = sender_clone.lock().await;
                        if let Err(e) = sender_guard.send(Message::Pong(payload)).await {
                            tracing::error!("Error sending pong: {}", e);
                            break;
                        }
                    }
                    Ok(Message::Close(_)) => {
                        break;
                    }
                    Err(e) => {
                        tracing::error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        // Spawn task to send messages to this connection
        let manager_clone = self.clone();
        let connection_id_clone = connection_id.clone();
        tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if let Some(connection) = manager_clone.connections.read().await.get(&connection_id_clone) {
                    // Check if this connection should receive this message
                    if manager_clone.should_send_message(connection, &msg).await {
                        let json = match serde_json::to_string(&msg) {
                            Ok(json) => json,
                            Err(e) => {
                                tracing::error!("Error serializing WebSocket message: {}", e);
                                continue;
                            }
                        };

                        let mut sender_guard = sender.lock().await;
                        if let Err(e) = sender_guard.send(Message::Text(json)).await {
                            tracing::error!("Error sending WebSocket message: {}", e);
                            break;
                        }
                    }
                }
            }
        });

        // Cleanup on disconnect
        {
            let mut connections = self.connections.write().await;
            connections.remove(&connection_id);
        }
    }

    /// Handle incoming WebSocket message
    async fn handle_message(&self, connection_id: &str, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        let message: serde_json::Value = serde_json::from_str(text)?;
        
        match message.get("type").and_then(|t| t.as_str()) {
            Some("ping") => {
                self.send_to_connection(connection_id, WebSocketMessage::Pong {
                    timestamp: Utc::now(),
                }).await?;
            }
            Some("pong") => {
                self.handle_pong(connection_id).await;
            }
            Some("subscribe") => {
                if let Some(server_id) = message.get("server_id").and_then(|s| s.as_str()) {
                    self.subscribe_to_server(connection_id, server_id).await?;
                }
            }
            Some("unsubscribe") => {
                if let Some(server_id) = message.get("server_id").and_then(|s| s.as_str()) {
                    self.unsubscribe_from_server(connection_id, server_id).await?;
                }
            }
            Some("set_events") => {
                if let Some(events) = message.get("events").and_then(|e| e.as_array()) {
                    let event_list: Vec<String> = events.iter()
                        .filter_map(|e| e.as_str().map(|s| s.to_string()))
                        .collect();
                    self.set_subscribed_events(connection_id, event_list).await?;
                }
            }
            _ => {
                self.send_to_connection(connection_id, WebSocketMessage::Error {
                    message: "Unknown message type".to_string(),
                    timestamp: Utc::now(),
                }).await?;
            }
        }
        
        Ok(())
    }

    /// Check if a connection should receive a specific message
    async fn should_send_message(&self, connection: &WebSocketConnection, msg: &WebSocketMessage) -> bool {
        // Check if connection is subscribed to all events
        if connection.subscribed_events.contains(&"all".to_string()) {
            return true;
        }

        // Check server-specific subscriptions
        let server_id = match msg {
            WebSocketMessage::ConsoleMessage { server_id, .. } => Some(server_id.to_string()),
            WebSocketMessage::MetricsUpdate { server_id, .. } => Some(server_id.to_string()),
            WebSocketMessage::PlayerEvent { server_id, .. } => Some(server_id.to_string()),
            WebSocketMessage::ServerStatusChange { server_id, .. } => Some(server_id.to_string()),
            WebSocketMessage::WorldFreeze { server_id, .. } => Some(server_id.to_string()),
            WebSocketMessage::PregenProgress { server_id, .. } => Some(server_id.to_string()),
            WebSocketMessage::ProgressEvent { server_id, .. } => server_id.clone(),
            _ => None,
        };

        if let Some(server_id) = server_id {
            if let Some(conn_server_id) = &connection.server_id {
                if conn_server_id == &server_id {
                    return true;
                }
            }
        }

        // Check event type subscriptions
        let event_type = match msg {
            WebSocketMessage::ConsoleMessage { .. } => "console",
            WebSocketMessage::MetricsUpdate { .. } => "metrics",
            WebSocketMessage::PlayerEvent { .. } => "players",
            WebSocketMessage::ServerStatusChange { .. } => "status",
            WebSocketMessage::WorldFreeze { .. } => "freezes",
            WebSocketMessage::PregenProgress { .. } => "pregen",
            WebSocketMessage::ProgressEvent { .. } => "progress",
            WebSocketMessage::Ping { .. } | WebSocketMessage::Pong { .. } => "ping",
            WebSocketMessage::Error { .. } => "error",
            WebSocketMessage::JobStarted { .. } => "jobs",
            WebSocketMessage::JobProgress { .. } => "jobs",
            WebSocketMessage::JobCompleted { .. } => "jobs",
            WebSocketMessage::JobFailed { .. } => "jobs",
        };

        connection.subscribed_events.contains(&event_type.to_string())
    }

    /// Send message to a specific connection
    pub async fn send_to_connection(&self, connection_id: &str, message: WebSocketMessage) -> Result<(), Box<dyn std::error::Error>> {
        // This would be implemented with a proper message queue
        // For now, we'll use the broadcast channel
        self.broadcast_tx.send(message)?;
        Ok(())
    }

    /// Get the number of active connections
    pub async fn get_connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }

    /// Subscribe connection to a specific server
    async fn subscribe_to_server(&self, connection_id: &str, server_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(connection_id) {
            connection.server_id = Some(server_id.to_string());
        }
        Ok(())
    }

    /// Unsubscribe connection from a specific server
    async fn unsubscribe_from_server(&self, connection_id: &str, server_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(connection_id) {
            if connection.server_id.as_ref() == Some(&server_id.to_string()) {
                connection.server_id = None;
            }
        }
        Ok(())
    }

    /// Set subscribed events for a connection
    async fn set_subscribed_events(&self, connection_id: &str, events: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(connection_id) {
            connection.subscribed_events = events;
        }
        Ok(())
    }

    /// Broadcast message to all connections
    pub async fn broadcast(&self, message: WebSocketMessage) -> Result<(), Box<dyn std::error::Error>> {
        self.broadcast_tx.send(message)?;
        Ok(())
    }

    /// Broadcast message to connections subscribed to a specific server
    pub async fn broadcast_to_server(&self, server_id: &str, message: WebSocketMessage) -> Result<(), Box<dyn std::error::Error>> {
        // Get or create server-specific channel
        let server_tx = {
            let mut channels = self.server_channels.write().await;
            channels.entry(server_id.to_string())
                .or_insert_with(|| {
                    let (tx, _) = broadcast::channel(100);
                    tx
                })
                .clone()
        };
        
        server_tx.send(message)?;
        Ok(())
    }

    /// Get connection count
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Get server-specific connection count
    pub async fn server_connection_count(&self, server_id: &str) -> usize {
        let connections = self.connections.read().await;
        connections.values()
            .filter(|conn| conn.server_id.as_ref() == Some(&server_id.to_string()))
            .count()
    }

    /// Send server status update
    pub async fn send_server_status(&self, server_id: Uuid, status: String) -> Result<(), Box<dyn std::error::Error>> {
        let message = WebSocketMessage::ServerStatusChange {
            server_id: server_id.to_string(),
            timestamp: Utc::now(),
            old_status: "unknown".to_string(),
            new_status: status.clone(),
        };
        self.broadcast_to_server(&server_id.to_string(), message).await
    }

    /// Send server metrics
    pub async fn send_metrics(&self, server_id: Uuid, metrics: serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        // Extract metrics from the JSON value
        let tps = metrics.get("tps").and_then(|v| v.as_f64()).unwrap_or(20.0);
        let tick_p95_ms = metrics.get("tick_p95_ms").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let heap_mb = metrics.get("heap_mb").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let players_online = metrics.get("players_online").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let memory_usage_mb = metrics.get("memory_usage_mb").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let cpu_usage_percent = metrics.get("cpu_usage_percent").and_then(|v| v.as_f64()).unwrap_or(0.0);

        let message = WebSocketMessage::MetricsUpdate {
            server_id: server_id.to_string(),
            timestamp: Utc::now(),
            tps,
            tick_p95_ms,
            heap_mb,
            players_online,
            memory_usage_mb,
            cpu_usage_percent,
        };
        self.broadcast_to_server(&server_id.to_string(), message).await
    }

    /// Send server status update (alias for compatibility)
    pub async fn send_server_status_update(&self, server_id: Uuid, status: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.send_server_status(server_id, status.to_string()).await
    }


    /// Start heartbeat monitoring for all connections
    pub async fn start_heartbeat_monitoring(&self) {
        let connections = self.connections.clone();
        let heartbeat_interval = self.heartbeat_interval;
        let heartbeat_timeout = self.heartbeat_timeout;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(heartbeat_interval);
            
            loop {
                interval.tick().await;
                
                // Check health of all connections and remove dead ones
                let now = Utc::now();
                let mut to_remove = Vec::new();
                
                {
                    let mut connections = connections.write().await;
                    
                    for (connection_id, connection) in connections.iter_mut() {
                        let time_since_pong = now.signed_duration_since(connection.last_pong);
                        
                        if time_since_pong.num_seconds() > heartbeat_timeout.as_secs() as i64 {
                            tracing::warn!("Connection {} is unhealthy, marking for removal", connection_id);
                            connection.is_healthy = false;
                            to_remove.push(connection_id.clone());
                        }
                    }
                    
                    // Remove unhealthy connections
                    for connection_id in to_remove {
                        connections.remove(&connection_id);
                        tracing::info!("Removed unhealthy connection: {}", connection_id);
                    }
                }
            }
        });
    }

    /// Check health of all connections and remove dead ones
    async fn check_connection_health(&self) {
        let now = Utc::now();
        let mut to_remove = Vec::new();
        
        {
            let mut connections = self.connections.write().await;
            
            for (connection_id, connection) in connections.iter_mut() {
                let time_since_pong = now.signed_duration_since(connection.last_pong);
                
                if time_since_pong.num_seconds() > self.heartbeat_timeout.as_secs() as i64 {
                    tracing::warn!("Connection {} is unhealthy, marking for removal", connection_id);
                    connection.is_healthy = false;
                    to_remove.push(connection_id.clone());
                }
            }
            
            // Remove unhealthy connections
            for connection_id in to_remove {
                connections.remove(&connection_id);
                tracing::info!("Removed unhealthy connection: {}", connection_id);
            }
        }
    }

    /// Send ping to a specific connection
    pub async fn send_ping(&self, connection_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let ping_message = WebSocketMessage::Ping {
            timestamp: Utc::now(),
        };
        
        // Update last ping time
        {
            let mut connections = self.connections.write().await;
            if let Some(connection) = connections.get_mut(connection_id) {
                connection.last_ping = Utc::now();
            }
        }
        
        self.broadcast(ping_message).await
    }

    /// Handle pong response from client
    pub async fn handle_pong(&self, connection_id: &str) {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(connection_id) {
            connection.last_pong = Utc::now();
            connection.is_healthy = true;
        }
    }

    /// Get connection health status
    pub async fn get_connection_health(&self, connection_id: &str) -> Option<bool> {
        let connections = self.connections.read().await;
        connections.get(connection_id).map(|c| c.is_healthy)
    }

    /// Get all connection statistics
    pub async fn get_connection_stats(&self) -> HashMap<String, serde_json::Value> {
        let connections = self.connections.read().await;
        let mut stats = HashMap::new();
        
        for (connection_id, connection) in connections.iter() {
            let connection_stats = serde_json::json!({
                "id": connection_id,
                "server_id": connection.server_id,
                "is_healthy": connection.is_healthy,
                "reconnect_count": connection.reconnect_count,
                "connection_duration_seconds": connection.connection_time.elapsed().as_secs(),
                "last_ping": connection.last_ping,
                "last_pong": connection.last_pong,
                "last_reconnect": connection.last_reconnect,
            });
            
            stats.insert(connection_id.clone(), connection_stats);
        }
        
        stats
    }

    /// Clean up expired connections
    pub async fn cleanup_expired_connections(&self) {
        let now = Utc::now();
        let mut to_remove = Vec::new();
        
        {
            let connections = self.connections.read().await;
            
            for (connection_id, connection) in connections.iter() {
                let time_since_pong = now.signed_duration_since(connection.last_pong);
                
                if time_since_pong.num_seconds() > self.heartbeat_timeout.as_secs() as i64 {
                    to_remove.push(connection_id.clone());
                }
            }
        }
        
        if !to_remove.is_empty() {
            let mut connections = self.connections.write().await;
            for connection_id in to_remove {
                connections.remove(&connection_id);
                tracing::info!("Cleaned up expired connection: {}", connection_id);
            }
        }
    }

    /// Send progress event for long-running operations
    pub async fn send_progress_event(&self, server_id: Option<&str>, job_id: &str, job_type: &str, 
                                   status: &str, progress: f32, current_step: &str, total_steps: u32,
                                   current_step_progress: f32, message: Option<&str>, error: Option<&str>,
                                   estimated_remaining_ms: Option<u64>) -> Result<(), Box<dyn std::error::Error>> {
        let progress_message = WebSocketMessage::ProgressEvent {
            server_id: server_id.map(|s| s.to_string()),
            job_id: job_id.to_string(),
            job_type: job_type.to_string(),
            status: status.to_string(),
            progress,
            current_step: current_step.to_string(),
            total_steps,
            current_step_progress,
            message: message.map(|s| s.to_string()),
            error: error.map(|s| s.to_string()),
            estimated_remaining_ms,
            timestamp: Utc::now(),
        };

        self.broadcast(progress_message).await
    }

    /// Send job started event
    pub async fn send_job_started(&self, server_id: Option<&str>, job_id: &str, job_type: &str, total_steps: u32) -> Result<(), Box<dyn std::error::Error>> {
        self.send_progress_event(
            server_id,
            job_id,
            job_type,
            "started",
            0.0,
            "Initializing",
            total_steps,
            0.0,
            Some(&format!("Starting {} job", job_type)),
            None,
            None,
        ).await
    }

    /// Send job progress update
    pub async fn send_job_progress(&self, server_id: Option<&str>, job_id: &str, job_type: &str, 
                                 current_step: &str, step_progress: f32, total_steps: u32, 
                                 message: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        let overall_progress = (total_steps as f32 - 1.0 + step_progress) / total_steps as f32;
        
        self.send_progress_event(
            server_id,
            job_id,
            job_type,
            "in_progress",
            overall_progress.min(1.0),
            current_step,
            total_steps,
            step_progress,
            message,
            None,
            None,
        ).await
    }

    /// Send job completed event
    pub async fn send_job_completed(&self, server_id: Option<&str>, job_id: &str, job_type: &str, 
                                  message: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        self.send_progress_event(
            server_id,
            job_id,
            job_type,
            "completed",
            1.0,
            "Completed",
            1,
            1.0,
            message,
            None,
            Some(0),
        ).await
    }

    /// Send job failed event
    pub async fn send_job_failed(&self, server_id: Option<&str>, job_id: &str, job_type: &str, 
                                error: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.send_progress_event(
            server_id,
            job_id,
            job_type,
            "failed",
            0.0,
            "Failed",
            1,
            0.0,
            Some("Job failed"),
            Some(error),
            None,
        ).await
    }
}

impl Default for WebSocketManager {
    fn default() -> Self {
        Self::new()
    }
}
