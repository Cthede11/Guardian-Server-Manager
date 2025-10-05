use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::core::{
    app_state::AppState,
    error_handler::{AppError, Result},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub id: String,
    pub server_id: Option<String>,
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct WebSocketConnection {
    pub id: String,
    pub user_id: Option<String>,
    pub server_id: Option<String>,
    pub sender: broadcast::Sender<WebSocketMessage>,
}

#[derive(Debug)]
pub struct WebSocketManager {
    connections: Arc<RwLock<HashMap<String, WebSocketConnection>>>,
    global_sender: broadcast::Sender<WebSocketMessage>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        let (global_sender, _) = broadcast::channel(1000);
        
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            global_sender,
        }
    }
    
    pub async fn handle_websocket(
        ws: WebSocketUpgrade,
        State(state): State<Arc<AppState>>,
    ) -> Response {
        ws.on_upgrade(|socket| Self::handle_socket(socket, state))
    }
    
    async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
        let connection_id = Uuid::new_v4().to_string();
        let (mut sender, mut receiver) = socket.split();
        
        // Create a broadcast channel for this connection
        let (tx, mut rx) = broadcast::channel(1000);
        
        // Register the connection
        let connection = WebSocketConnection {
            id: connection_id.clone(),
            user_id: None,
            server_id: None,
            sender: tx.clone(),
        };
        
        {
            let mut connections = state.websocket.connections.write().await;
            // Convert to websocket_manager::WebSocketConnection
            let ws_connection = crate::websocket_manager::WebSocketConnection {
                id: connection_id.clone(),
                server_id: connection.server_id.clone(),
                subscribed_events: vec![],
                last_ping: chrono::Utc::now(),
            };
            connections.insert(connection_id.clone(), ws_connection);
        }
        
        // Subscribe to global messages
        let mut global_rx = state.websocket.global_sender.subscribe();
        
        // Create a channel for sending messages to the WebSocket
        let (ws_tx, mut ws_rx) = tokio::sync::mpsc::unbounded_channel();
        
        // Spawn task to send messages to the client
        tokio::spawn(async move {
            while let Some(msg) = ws_rx.recv().await {
                if let Err(e) = sender.send(msg).await {
                    tracing::error!("Failed to send message to client: {}", e);
                    break;
                }
            }
        });
        
        // Spawn task to handle incoming messages from channels
        let websocket_manager = state.websocket.clone();
        let ws_tx_clone = ws_tx.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Send messages from this connection's channel
                    msg = rx.recv() => {
                        match msg {
                            Ok(msg) => {
                                let ws_msg = Message::Text(serde_json::to_string(&msg).unwrap_or_default());
                                if let Err(_) = ws_tx_clone.send(ws_msg) {
                                    break;
                                }
                            }
                            Err(broadcast::error::RecvError::Closed) => break,
                            Err(broadcast::error::RecvError::Lagged(_)) => continue,
                        }
                    }
                    // Send global messages
                    msg = global_rx.recv() => {
                        match msg {
                            Ok(msg) => {
                                let ws_msg = Message::Text(serde_json::to_string(&msg).unwrap_or_default());
                                if let Err(_) = ws_tx_clone.send(ws_msg) {
                                    break;
                                }
                            }
                            Err(broadcast::error::RecvError::Closed) => break,
                            Err(broadcast::error::RecvError::Lagged(_)) => continue,
                        }
                    }
                }
            }
        });
        
        // Handle incoming messages
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Err(e) = Self::handle_message(&state, &connection_id, &text).await {
                        tracing::error!("Failed to handle message: {}", e);
                    }
                }
                Ok(Message::Close(_)) => break,
                Ok(Message::Ping(data)) => {
                    let pong_msg = Message::Pong(data);
                    if let Err(_) = ws_tx.send(pong_msg) {
                        break;
                    }
                }
                Ok(Message::Pong(_)) => {}
                Ok(Message::Binary(_)) => {}
                Err(e) => {
                    tracing::error!("WebSocket error: {}", e);
                    break;
                }
            }
        }
        
        // Clean up connection
        {
            let mut connections = state.websocket.connections.write().await;
            connections.remove(&connection_id);
        }
        
        tracing::info!("WebSocket connection {} closed", connection_id);
    }
    
    async fn handle_message(state: &Arc<AppState>, connection_id: &str, message: &str) -> Result<()> {
        let message: WebSocketMessage = serde_json::from_str(message)
            .map_err(|e| AppError::ValidationError {
                message: format!("Invalid WebSocket message: {}", e),
                field: "message".to_string(),
                value: "invalid".to_string(),
                constraint: "valid JSON format".to_string(),
            })?;
        
        match message.event_type.as_str() {
            "subscribe_server" => {
                if let Some(server_id) = message.data.get("server_id") {
                    if let Some(server_id) = server_id.as_str() {
                        Self::subscribe_to_server(state, connection_id, server_id).await?;
                    }
                }
            }
            "unsubscribe_server" => {
                if let Some(server_id) = message.data.get("server_id") {
                    if let Some(server_id) = server_id.as_str() {
                        Self::unsubscribe_from_server(state, connection_id, server_id).await?;
                    }
                }
            }
            "ping" => {
                Self::send_pong(state, connection_id).await?;
            }
            _ => {
                tracing::warn!("Unknown WebSocket message type: {}", message.event_type);
            }
        }
        
        Ok(())
    }
    
    async fn subscribe_to_server(state: &Arc<AppState>, connection_id: &str, server_id: &str) -> Result<()> {
        let mut connections = state.websocket.connections.write().await;
        
        if let Some(connection) = connections.get_mut(connection_id) {
            connection.server_id = Some(server_id.to_string());
        }
        
        Ok(())
    }
    
    async fn unsubscribe_from_server(state: &Arc<AppState>, connection_id: &str, server_id: &str) -> Result<()> {
        let mut connections = state.websocket.connections.write().await;
        
        if let Some(connection) = connections.get_mut(connection_id) {
            if connection.server_id == Some(server_id.to_string()) {
                connection.server_id = None;
            }
        }
        
        Ok(())
    }
    
    async fn send_pong(state: &Arc<AppState>, connection_id: &str) -> Result<()> {
        let pong_message = WebSocketMessage {
            id: Uuid::new_v4().to_string(),
            server_id: None,
            event_type: "pong".to_string(),
            data: serde_json::Value::Null,
            timestamp: chrono::Utc::now(),
        };
        
        // Convert to websocket_manager::WebSocketMessage
        let ws_message = crate::websocket_manager::WebSocketMessage::ConsoleMessage {
            server_id: pong_message.server_id.unwrap_or_default(),
            timestamp: pong_message.timestamp,
            level: "info".to_string(),
            message: "pong".to_string(),
        };
        state.websocket.send_to_connection(connection_id, ws_message).await
            .map_err(|e| AppError::WebSocketError {
                message: format!("Failed to send pong: {}", e),
                connection_id: Some(connection_id.to_string()),
                event_type: Some("pong".to_string()),
            })
    }
    
    pub async fn send_to_connection(&self, connection_id: &str, message: WebSocketMessage) -> Result<()> {
        let connections = self.connections.read().await;
        
        if let Some(connection) = connections.get(connection_id) {
            let _ = connection.sender.send(message);
        }
        
        Ok(())
    }
    
    pub async fn send_to_server(&self, server_id: &str, message: WebSocketMessage) -> Result<()> {
        let connections = self.connections.read().await;
        
        for connection in connections.values() {
            if connection.server_id == Some(server_id.to_string()) {
                let _ = connection.sender.send(message.clone());
            }
        }
        
        Ok(())
    }
    
    pub async fn broadcast(&self, message: WebSocketMessage) -> Result<()> {
        let _ = self.global_sender.send(message);
        Ok(())
    }
    
    pub async fn send_server_status_update(&self, server_id: &str, status: &str) -> Result<()> {
        let message = WebSocketMessage {
            id: Uuid::new_v4().to_string(),
            server_id: Some(server_id.to_string()),
            event_type: "server_status".to_string(),
            data: serde_json::json!({
                "status": status,
                "timestamp": chrono::Utc::now()
            }),
            timestamp: chrono::Utc::now(),
        };
        
        self.send_to_server(server_id, message).await
    }
    
    pub async fn send_server_metrics(&self, server_id: &str, metrics: serde_json::Value) -> Result<()> {
        let message = WebSocketMessage {
            id: Uuid::new_v4().to_string(),
            server_id: Some(server_id.to_string()),
            event_type: "server_metrics".to_string(),
            data: metrics,
            timestamp: chrono::Utc::now(),
        };
        
        self.send_to_server(server_id, message).await
    }
    
    pub async fn send_console_message(&self, server_id: &str, level: &str, message: &str) -> Result<()> {
        let message = WebSocketMessage {
            id: Uuid::new_v4().to_string(),
            server_id: Some(server_id.to_string()),
            event_type: "console_message".to_string(),
            data: serde_json::json!({
                "level": level,
                "message": message,
                "timestamp": chrono::Utc::now()
            }),
            timestamp: chrono::Utc::now(),
        };
        
        self.send_to_server(server_id, message).await
    }
    
    pub async fn send_alert(&self, server_id: Option<&str>, level: &str, title: &str, message: &str) -> Result<()> {
        let message = WebSocketMessage {
            id: Uuid::new_v4().to_string(),
            server_id: server_id.map(|s| s.to_string()),
            event_type: "alert".to_string(),
            data: serde_json::json!({
                "level": level,
                "title": title,
                "message": message,
                "timestamp": chrono::Utc::now()
            }),
            timestamp: chrono::Utc::now(),
        };
        
        if let Some(server_id) = server_id {
            self.send_to_server(server_id, message).await
        } else {
            self.broadcast(message).await
        }
    }
}
