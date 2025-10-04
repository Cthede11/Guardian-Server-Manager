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
}

/// WebSocket connection information
#[derive(Debug, Clone)]
pub struct WebSocketConnection {
    pub id: String,
    pub server_id: Option<String>,
    pub subscribed_events: Vec<String>,
    pub last_ping: DateTime<Utc>,
}

/// WebSocket manager for handling real-time connections
pub struct WebSocketManager {
    /// Active connections
    connections: Arc<RwLock<HashMap<String, WebSocketConnection>>>,
    /// Broadcast channel for sending messages to all connections
    broadcast_tx: broadcast::Sender<WebSocketMessage>,
    /// Server-specific broadcast channels
    server_channels: Arc<RwLock<HashMap<String, broadcast::Sender<WebSocketMessage>>>>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);
        
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
            server_channels: Arc::new(RwLock::new(HashMap::new())),
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
    async fn handle_socket(self: Arc<Self>, socket: WebSocket) {
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
            WebSocketMessage::ConsoleMessage { server_id, .. } => Some(server_id),
            WebSocketMessage::MetricsUpdate { server_id, .. } => Some(server_id),
            WebSocketMessage::PlayerEvent { server_id, .. } => Some(server_id),
            WebSocketMessage::ServerStatusChange { server_id, .. } => Some(server_id),
            WebSocketMessage::WorldFreeze { server_id, .. } => Some(server_id),
            WebSocketMessage::PregenProgress { server_id, .. } => Some(server_id),
            _ => None,
        };

        if let Some(server_id) = server_id {
            if let Some(conn_server_id) = &connection.server_id {
                if conn_server_id == server_id {
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
            WebSocketMessage::Ping { .. } | WebSocketMessage::Pong { .. } => "ping",
            WebSocketMessage::Error { .. } => "error",
        };

        connection.subscribed_events.contains(&event_type.to_string())
    }

    /// Send message to a specific connection
    async fn send_to_connection(&self, connection_id: &str, message: WebSocketMessage) -> Result<(), Box<dyn std::error::Error>> {
        // This would be implemented with a proper message queue
        // For now, we'll use the broadcast channel
        self.broadcast_tx.send(message)?;
        Ok(())
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
}

impl Default for WebSocketManager {
    fn default() -> Self {
        Self::new()
    }
}
