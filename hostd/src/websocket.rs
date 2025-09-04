use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::{broadcast, RwLock};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// WebSocket connection manager
#[derive(Debug, Clone)]
pub struct WebSocketManager {
    /// Broadcast channel for sending messages to all connected clients
    tx: broadcast::Sender<WebSocketMessage>,
    /// Active connections
    connections: Arc<RwLock<HashMap<Uuid, WebSocketConnection>>>,
}

/// Individual WebSocket connection
#[derive(Debug, Clone)]
pub struct WebSocketConnection {
    pub id: Uuid,
    pub server_id: Option<String>,
    pub connected_at: chrono::DateTime<chrono::Utc>,
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketMessage {
    /// Server status update
    ServerStatus {
        server_id: String,
        status: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Console message
    ConsoleMessage {
        server_id: String,
        message: String,
        level: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Player data update
    PlayerUpdate {
        server_id: String,
        players: Vec<PlayerData>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Metrics update
    MetricsUpdate {
        server_id: String,
        metrics: MetricsData,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// World data update
    WorldUpdate {
        server_id: String,
        world_data: WorldData,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Pregen job update
    PregenUpdate {
        server_id: String,
        jobs: Vec<PregenJobData>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Health check response
    HealthCheck {
        status: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Error message
    Error {
        message: String,
        code: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Player data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    pub uuid: String,
    pub name: String,
    pub online: bool,
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
    pub playtime: Option<u64>,
}

/// Metrics data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsData {
    pub tps: f64,
    pub tick_p95: f64,
    pub heap_mb: u64,
    pub players_online: u32,
    pub gpu_queue_ms: f64,
}

/// World data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldData {
    pub freezes: Vec<FreezeData>,
    pub chunks_loaded: u32,
    pub world_size_mb: u64,
}

/// Freeze data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreezeData {
    pub x: i32,
    pub z: i32,
    pub duration_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Pregen job data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PregenJobData {
    pub id: String,
    pub region: RegionData,
    pub dimension: String,
    pub priority: String,
    pub status: String,
    pub progress: f64,
    pub eta: Option<String>,
    pub gpu_assist: bool,
}

/// Region data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionData {
    pub x: i32,
    pub z: i32,
    pub radius: u32,
}

impl WebSocketManager {
    /// Create a new WebSocket manager
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(1000);
        
        Self {
            tx,
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the broadcast sender
    pub fn sender(&self) -> broadcast::Sender<WebSocketMessage> {
        self.tx.clone()
    }

    /// Add a new connection
    pub async fn add_connection(&self, id: Uuid, server_id: Option<String>) {
        let connection = WebSocketConnection {
            id,
            server_id,
            connected_at: chrono::Utc::now(),
        };

        let mut connections = self.connections.write().await;
        connections.insert(id, connection);
        
        info!("WebSocket connection added: {} (server: {:?})", id, server_id);
    }

    /// Remove a connection
    pub async fn remove_connection(&self, id: Uuid) {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.remove(&id) {
            info!("WebSocket connection removed: {} (server: {:?})", id, connection.server_id);
        }
    }

    /// Get all connections for a specific server
    pub async fn get_server_connections(&self, server_id: &str) -> Vec<Uuid> {
        let connections = self.connections.read().await;
        connections
            .iter()
            .filter(|(_, conn)| conn.server_id.as_ref() == Some(server_id))
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get connection count
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Broadcast message to all connections
    pub async fn broadcast(&self, message: WebSocketMessage) -> Result<usize, broadcast::error::SendError<WebSocketMessage>> {
        self.tx.send(message)
    }

    /// Broadcast message to specific server connections
    pub async fn broadcast_to_server(&self, server_id: &str, message: WebSocketMessage) {
        let server_connections = self.get_server_connections(server_id).await;
        
        for connection_id in server_connections {
            if let Err(e) = self.tx.send(message.clone()) {
                warn!("Failed to send message to connection {}: {}", connection_id, e);
            }
        }
    }
}

/// WebSocket handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(manager): State<Arc<WebSocketManager>>,
) -> Response {
    ws.on_upgrade(|socket| websocket_connection(socket, manager))
}

/// Handle individual WebSocket connection
async fn websocket_connection(socket: WebSocket, manager: Arc<WebSocketManager>) {
    let connection_id = Uuid::new_v4();
    let mut rx = manager.sender().subscribe();
    
    // Add connection to manager
    manager.add_connection(connection_id, None).await;
    
    // Send welcome message
    let welcome_msg = WebSocketMessage::HealthCheck {
        status: "connected".to_string(),
        timestamp: chrono::Utc::now(),
    };
    
    if let Ok(msg) = serde_json::to_string(&welcome_msg) {
        if let Err(e) = socket.send(Message::Text(msg)).await {
            error!("Failed to send welcome message: {}", e);
        }
    }

    let (mut sender, mut receiver) = socket.split();

    // Spawn task to handle incoming messages
    let manager_clone = manager.clone();
    let connection_id_clone = connection_id;
    tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    debug!("Received message from {}: {}", connection_id_clone, text);
                    
                    // Parse and handle incoming messages
                    if let Ok(message) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let Some(msg_type) = message.get("type").and_then(|v| v.as_str()) {
                            match msg_type {
                                "subscribe" => {
                                    if let Some(server_id) = message.get("server_id").and_then(|v| v.as_str()) {
                                        // Update connection with server ID
                                        let mut connections = manager_clone.connections.write().await;
                                        if let Some(connection) = connections.get_mut(&connection_id_clone) {
                                            connection.server_id = Some(server_id.to_string());
                                            info!("Connection {} subscribed to server {}", connection_id_clone, server_id);
                                        }
                                    }
                                }
                                "ping" => {
                                    // Respond to ping with pong
                                    let pong_msg = WebSocketMessage::HealthCheck {
                                        status: "pong".to_string(),
                                        timestamp: chrono::Utc::now(),
                                    };
                                    
                                    if let Ok(msg) = serde_json::to_string(&pong_msg) {
                                        if let Err(e) = sender.send(Message::Text(msg)).await {
                                            error!("Failed to send pong: {}", e);
                                        }
                                    }
                                }
                                _ => {
                                    debug!("Unknown message type: {}", msg_type);
                                }
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection {} closed", connection_id_clone);
                    break;
                }
                Ok(Message::Ping(data)) => {
                    if let Err(e) = sender.send(Message::Pong(data)).await {
                        error!("Failed to send pong: {}", e);
                    }
                }
                Ok(Message::Pong(_)) => {
                    // Handle pong if needed
                }
                Ok(Message::Binary(_)) => {
                    debug!("Received binary message from {}", connection_id_clone);
                }
                Err(e) => {
                    error!("WebSocket error for connection {}: {}", connection_id_clone, e);
                    break;
                }
            }
        }
        
        // Remove connection when done
        manager_clone.remove_connection(connection_id_clone).await;
    });

    // Spawn task to handle outgoing messages
    let manager_clone = manager.clone();
    let connection_id_clone = connection_id;
    tokio::spawn(async move {
        while let Ok(message) = rx.recv().await {
            if let Ok(msg) = serde_json::to_string(&message) {
                if let Err(e) = sender.send(Message::Text(msg)).await {
                    error!("Failed to send message to connection {}: {}", connection_id_clone, e);
                    break;
                }
            }
        }
    });
}

/// Create WebSocket router
pub fn create_websocket_router(manager: Arc<WebSocketManager>) -> Router {
    Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(manager)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_websocket_manager() {
        let manager = WebSocketManager::new();
        
        // Test adding connection
        let id = Uuid::new_v4();
        manager.add_connection(id, Some("test-server".to_string())).await;
        
        assert_eq!(manager.connection_count().await, 1);
        
        // Test getting server connections
        let connections = manager.get_server_connections("test-server").await;
        assert_eq!(connections.len(), 1);
        assert_eq!(connections[0], id);
        
        // Test removing connection
        manager.remove_connection(id).await;
        assert_eq!(manager.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_broadcast() {
        let manager = WebSocketManager::new();
        
        let message = WebSocketMessage::HealthCheck {
            status: "test".to_string(),
            timestamp: chrono::Utc::now(),
        };
        
        let result = manager.broadcast(message).await;
        assert!(result.is_ok());
    }
}
