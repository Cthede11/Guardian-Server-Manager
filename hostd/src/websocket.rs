use std::collections::HashMap;
use std::sync::Arc;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State,
    },
    response::Response,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;
use tracing::{info, error, debug};

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    HealthCheck {
        status: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    ServerEvent {
        server_id: String,
        event: String,
        data: serde_json::Value,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    ServerStatus {
        server_id: String,
        status: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    Error {
        message: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// WebSocket connection info
#[derive(Debug, Clone)]
pub struct WebSocketConnection {
    pub id: Uuid,
    pub server_id: Option<String>,
    pub connected_at: chrono::DateTime<chrono::Utc>,
}

/// WebSocket manager
#[derive(Debug, Clone)]
pub struct WebSocketManager {
    connections: Arc<RwLock<HashMap<Uuid, WebSocketConnection>>>,
    sender: broadcast::Sender<WebSocketMessage>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            sender,
        }
    }

    pub fn sender(&self) -> broadcast::Sender<WebSocketMessage> {
        self.sender.clone()
    }

    /// Add a new connection
    pub async fn add_connection(&self, id: Uuid, server_id: Option<String>) {
        let connection = WebSocketConnection {
            id,
            server_id: server_id.clone(),
            connected_at: chrono::Utc::now(),
        };

        let mut connections = self.connections.write().await;
        connections.insert(id, connection);

        info!("WebSocket connection added: {} (server: {:?})", id, server_id);
    }

    /// Remove a connection
    pub async fn remove_connection(&self, id: Uuid) {
        let mut connections = self.connections.write().await;
        connections.remove(&id);
        info!("WebSocket connection removed: {}", id);
    }

    /// Get all connections for a specific server
    pub async fn get_server_connections(&self, server_id: &str) -> Vec<Uuid> {
        let connections = self.connections.read().await;
        connections
            .iter()
            .filter(|(_, conn)| conn.server_id.as_ref() == Some(&server_id.to_string()))
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get connection count
    pub async fn get_connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }

    /// Broadcast message to all connections
    pub async fn broadcast(&self, message: WebSocketMessage) {
        let _ = self.sender.send(message);
    }

    /// Broadcast message to specific server connections
    pub async fn broadcast_to_server(&self, server_id: &str, message: WebSocketMessage) {
        let connections = self.get_server_connections(server_id).await;
        for _ in connections {
            let _ = self.sender.send(message.clone());
        }
    }
}

/// Create WebSocket router
pub fn create_websocket_router() -> Router {
    let manager = Arc::new(WebSocketManager::new());
    
    Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(manager)
}

/// WebSocket handler
async fn websocket_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    State(manager): State<Arc<WebSocketManager>>,
) -> Response {
    ws.on_upgrade(|socket| websocket_connection(socket, manager))
}

/// Handle individual WebSocket connection - simplified for now
async fn websocket_connection(_socket: WebSocket, _manager: Arc<WebSocketManager>) {
    // Simplified websocket handling - just return for now
    return;
}
