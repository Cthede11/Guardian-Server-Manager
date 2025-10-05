use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use uuid::Uuid;

use crate::core::{
    config::Config,
    error_handler::Result,
};
use crate::database::DatabaseManager;
use crate::websocket_manager::WebSocketManager;
use crate::core::auth::AuthManager;

/// Global application state
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub database: Arc<DatabaseManager>,
    pub websocket: Arc<WebSocketManager>,
    pub auth: Arc<AuthManager>,
    pub active_servers: Arc<RwLock<HashMap<Uuid, ActiveServer>>>,
}

/// Information about an active server process
#[derive(Debug, Clone)]
pub struct ActiveServer {
    pub id: Uuid,
    pub name: String,
    pub status: ServerStatus,
    pub process_id: Option<u32>,
    pub port: u16,
    pub rcon_port: u16,
    pub query_port: u16,
    pub memory_usage: u64,
    pub cpu_usage: f32,
    pub uptime: std::time::Duration,
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Crashed,
    Unknown,
}

impl AppState {
    pub async fn new(config: Config, auth: Arc<AuthManager>) -> Result<Self> {
        // Initialize database
        let database = Arc::new(DatabaseManager::new("guardian.db").await?);
        
        // Initialize WebSocket manager
        let websocket = Arc::new(WebSocketManager::new());
        
        Ok(Self {
            config,
            database,
            websocket,
            auth,
            active_servers: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting Guardian Server Manager...");
        // Start any background tasks here
        Ok(())
    }
    
    pub async fn stop(&self) -> Result<()> {
        tracing::info!("Stopping Guardian Server Manager...");
        // Stop any background tasks here
        Ok(())
    }
    
    pub async fn add_active_server(&self, server: ActiveServer) {
        let mut servers = self.active_servers.write().await;
        servers.insert(server.id, server);
    }
    
    pub async fn remove_active_server(&self, server_id: Uuid) {
        let mut servers = self.active_servers.write().await;
        servers.remove(&server_id);
    }
    
    pub async fn get_active_server(&self, server_id: Uuid) -> Option<ActiveServer> {
        let servers = self.active_servers.read().await;
        servers.get(&server_id).cloned()
    }
    
    pub async fn list_active_servers(&self) -> Vec<ActiveServer> {
        let servers = self.active_servers.read().await;
        servers.values().cloned().collect()
    }
    
    pub async fn update_server_status(&self, server_id: Uuid, status: ServerStatus) -> Result<()> {
        let mut servers = self.active_servers.write().await;
        if let Some(server) = servers.get_mut(&server_id) {
            server.status = status;
            server.last_heartbeat = chrono::Utc::now();
        }
        Ok(())
    }
    
    pub async fn update_server_metrics(&self, server_id: Uuid, memory_usage: u64, cpu_usage: f32) -> Result<()> {
        let mut servers = self.active_servers.write().await;
        if let Some(server) = servers.get_mut(&server_id) {
            server.memory_usage = memory_usage;
            server.cpu_usage = cpu_usage;
            server.last_heartbeat = chrono::Utc::now();
        }
        Ok(())
    }
}