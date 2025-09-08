use std::collections::HashMap;
use std::sync::Arc;
use axum::{
    extract::{
        ws::{WebSocket, Message},
        Path, State,
    },
    response::Response,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock, mpsc};
use uuid::Uuid;
use tracing::{info, error, debug, warn};
use futures_util::{SinkExt, StreamExt};

/// Console message for WebSocket streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleMessage {
    pub ts: String,
    pub level: String,
    pub msg: String,
}

/// Client message types (incoming from frontend)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "command")]
    Command {
        payload: CommandPayload,
    },
    #[serde(rename = "subscribe")]
    Subscribe {
        server_id: String,
    },
    #[serde(rename = "ping")]
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPayload {
    pub command: String,
}

/// Server message types (outgoing to frontend)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "console")]
    Console {
        serverId: String,
        messages: Vec<ConsoleMessage>,
    },
    #[serde(rename = "connected")]
    Connected,
    #[serde(rename = "disconnected")]
    Disconnected,
    #[serde(rename = "error")]
    Error {
        message: String,
    },
    #[serde(rename = "pong")]
    Pong,
}

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
    Console {
        server_id: String,
        messages: Vec<ConsoleMessage>,
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
    pub sender: mpsc::UnboundedSender<ServerMessage>,
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
    pub async fn add_connection(&self, id: Uuid, server_id: Option<String>, sender: mpsc::UnboundedSender<ServerMessage>) {
        let connection = WebSocketConnection {
            id,
            server_id: server_id.clone(),
            connected_at: chrono::Utc::now(),
            sender,
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
        let connections = self.connections.read().await;
        for (_, connection) in connections.iter() {
            if connection.server_id.as_ref() == Some(&server_id.to_string()) {
                match &message {
                    WebSocketMessage::Console { server_id: msg_server_id, messages } => {
                        let server_msg = ServerMessage::Console {
                            serverId: msg_server_id.clone(),
                            messages: messages.clone(),
                        };
                        let _ = connection.sender.send(server_msg);
                    },
                    _ => {
                        // Handle other message types as needed
                    }
                }
            }
        }
    }

    /// Send console messages to server connections
    pub async fn send_console_messages(&self, server_id: &str, messages: Vec<ConsoleMessage>) {
        let message = WebSocketMessage::Console {
            server_id: server_id.to_string(),
            messages,
        };
        self.broadcast_to_server(server_id, message).await;
    }
}

/// Create WebSocket router
pub fn create_websocket_router() -> Router {
    let manager = Arc::new(WebSocketManager::new());
    
    Router::new()
        .route("/ws", get(websocket_handler))
        .route("/ws/servers/:server_id", get(websocket_server_handler))
        .with_state(manager)
}

/// WebSocket handler for general connections
async fn websocket_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    State(manager): State<Arc<WebSocketManager>>,
) -> Response {
    ws.on_upgrade(|socket| websocket_connection(socket, manager, None))
}

/// WebSocket handler for server-specific connections
async fn websocket_server_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    Path(server_id): Path<String>,
    State(manager): State<Arc<WebSocketManager>>,
) -> Response {
    ws.on_upgrade(|socket| websocket_connection(socket, manager, Some(server_id)))
}

/// Handle individual WebSocket connection
async fn websocket_connection(socket: WebSocket, manager: Arc<WebSocketManager>, server_id: Option<String>) {
    let connection_id = Uuid::new_v4();
    info!("New WebSocket connection: {} for server: {:?}", connection_id, server_id);

    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();

    // Add connection to manager
    manager.add_connection(connection_id, server_id.clone(), tx).await;

    // Send initial connection message
    let connected_msg = ServerMessage::Connected;
    if let Ok(msg_text) = serde_json::to_string(&connected_msg) {
        if let Err(e) = ws_sender.send(Message::Text(msg_text)).await {
            error!("Failed to send connected message: {}", e);
            return;
        }
    }

    // If connected to a specific server, send some initial console messages
    if let Some(ref srv_id) = server_id {
        let sample_messages = vec![
            ConsoleMessage {
                ts: chrono::Utc::now().to_rfc3339(),
                level: "info".to_string(),
                msg: format!("WebSocket connected to server {}", srv_id),
            },
            ConsoleMessage {
                ts: chrono::Utc::now().to_rfc3339(),
                level: "info".to_string(),
                msg: "Console streaming is now active".to_string(),
            },
        ];

        let console_msg = ServerMessage::Console {
            serverId: srv_id.clone(),
            messages: sample_messages,
        };

        if let Ok(msg_text) = serde_json::to_string(&console_msg) {
            if let Err(e) = ws_sender.send(Message::Text(msg_text)).await {
                error!("Failed to send initial console messages: {}", e);
            }
        }
    }

    // Handle both incoming and outgoing messages in a single task
    let manager_clone = manager.clone();
    let server_id_clone = server_id.clone();
    let connection_task = tokio::spawn(async move {
        let mut rx_clone = rx;
        
        loop {
            tokio::select! {
                // Handle outgoing messages
                message = rx_clone.recv() => {
                    match message {
                        Some(msg) => {
                            match serde_json::to_string(&msg) {
                                Ok(msg_text) => {
                                    if let Err(e) = ws_sender.send(Message::Text(msg_text)).await {
                                        error!("Failed to send message: {}", e);
                                        break;
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to serialize message: {}", e);
                                }
                            }
                        }
                        None => {
                            info!("Message channel closed");
                            break;
                        }
                    }
                }
                
                // Handle incoming messages
                msg = ws_receiver.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            debug!("Received WebSocket message: {}", text);
                            
                            // Try to parse as client message
                            match serde_json::from_str::<ClientMessage>(&text) {
                                Ok(client_msg) => {
                                    match client_msg {
                                        ClientMessage::Command { payload } => {
                                            if let Some(ref srv_id) = server_id_clone {
                                                info!("Received command for server {}: {}", srv_id, payload.command);
                                                
                                                // Echo the command back as a console message
                                                let command_echo = vec![ConsoleMessage {
                                                    ts: chrono::Utc::now().to_rfc3339(),
                                                    level: "info".to_string(),
                                                    msg: format!("> {}", payload.command),
                                                }];
                                                
                                                manager_clone.send_console_messages(srv_id, command_echo).await;
                                                
                                                // TODO: Actually execute the command via RCON
                                                // For now, just send a mock response
                                                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                                                
                                                let response = vec![ConsoleMessage {
                                                    ts: chrono::Utc::now().to_rfc3339(),
                                                    level: "info".to_string(),
                                                    msg: format!("Command '{}' executed successfully", payload.command),
                                                }];
                                                
                                                manager_clone.send_console_messages(srv_id, response).await;
                                            }
                                        }
                                        ClientMessage::Subscribe { server_id: sub_server_id } => {
                                            info!("Client subscribed to server: {}", sub_server_id);
                                            // TODO: Update connection's server_id if needed
                                        }
                                        ClientMessage::Ping => {
                                            debug!("Received ping, sending pong");
                                            // Pong will be handled by the outgoing message handler
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to parse client message: {} - {}", e, text);
                                }
                            }
                        }
                        Some(Ok(Message::Binary(_))) => {
                            debug!("Received binary message (ignoring)");
                        }
                        Some(Ok(Message::Ping(ping_data))) => {
                            debug!("Received ping, sending pong");
                            if let Err(e) = ws_sender.send(Message::Pong(ping_data)).await {
                                error!("Failed to send pong: {}", e);
                                break;
                            }
                        }
                        Some(Ok(Message::Pong(_))) => {
                            debug!("Received pong");
                        }
                        Some(Ok(Message::Close(_))) => {
                            info!("WebSocket connection closed by client");
                            break;
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                        None => {
                            info!("WebSocket connection closed");
                            break;
                        }
                    }
                }
            }
        }
    });

    // Spawn periodic task to send sample console messages for testing
    let manager_periodic = manager.clone();
    let server_id_periodic = server_id.clone();
    let periodic_task = tokio::spawn(async move {
        if let Some(srv_id) = server_id_periodic {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
            let mut counter = 0;
            
            loop {
                interval.tick().await;
                counter += 1;
                
                let sample_messages = vec![
                    ConsoleMessage {
                        ts: chrono::Utc::now().to_rfc3339(),
                        level: "info".to_string(),
                        msg: format!("Server update #{} - Server is running normally", counter),
                    },
                ];
                
                manager_periodic.send_console_messages(&srv_id, sample_messages).await;
                
                if counter >= 20 { // Stop after 20 messages
                    break;
                }
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = connection_task => {
            info!("Connection handler finished");
        }
        _ = periodic_task => {
            info!("Periodic task finished");
        }
    }

    // Clean up connection
    manager.remove_connection(connection_id).await;
    info!("WebSocket connection {} closed", connection_id);
}
