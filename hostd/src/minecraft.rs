use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    fs,
    process::Command as TokioCommand,
    sync::{Mutex, RwLock},
    time::{interval, sleep},
};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use crate::database::{DatabaseManager, ServerConfig, EventLog};
use crate::rcon::RconClient;

/// Minecraft server process manager
#[derive(Debug, Clone)]
pub struct MinecraftServer {
    pub id: String,
    pub config: ServerConfig,
    pub status: ServerStatus,
    pub process: Option<Arc<Mutex<Option<Child>>>>,
    pub last_start: Option<Instant>,
    pub restart_count: u32,
    pub last_restart: Option<Instant>,
}

/// Server status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Crashed,
    Unknown,
}

/// Server metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetrics {
    pub tps: f64,
    pub tick_p95: f64,
    pub heap_mb: u64,
    pub players_online: u32,
    pub gpu_queue_ms: f64,
    pub uptime_seconds: u64,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

/// Player information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub uuid: String,
    pub name: String,
    pub online: bool,
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
    pub playtime: Option<u64>,
    pub ping: Option<u32>,
    pub dimension: Option<String>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub z: Option<f64>,
}

/// Console message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleMessage {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: String,
    pub message: String,
    pub raw: String,
}

// RCON client is now imported from the rcon module

impl MinecraftServer {
    /// Create a new Minecraft server instance
    pub fn new(config: ServerConfig) -> Self {
        Self {
            id: config.id.clone(),
            config,
            status: ServerStatus::Stopped,
            process: None,
            last_start: None,
            restart_count: 0,
            last_restart: None,
        }
    }

    /// Start the Minecraft server
    pub async fn start(&mut self, db: &DatabaseManager) -> Result<()> {
        if self.status == ServerStatus::Running || self.status == ServerStatus::Starting {
            return Err(anyhow!("Server is already running or starting"));
        }

        info!("Starting Minecraft server: {}", self.id);
        
        self.status = ServerStatus::Starting;
        self.last_start = Some(Instant::now());

        // Log the start event
        let event = EventLog {
            id: Uuid::new_v4().to_string(),
            server_id: Some(self.id.clone()),
            event_type: "server_start".to_string(),
            message: "Server starting".to_string(),
            level: "info".to_string(),
            metadata: None,
            created_at: chrono::Utc::now(),
        };
        db.log_event(&event).await?;

        // Build the command
        let mut cmd = Command::new(&self.config.java_path);
        
        // Add JVM arguments
        for arg in self.config.jvm_args.split_whitespace() {
            cmd.arg(arg);
        }
        
        // Add server JAR
        cmd.arg("-jar").arg(&self.config.server_jar);
        
        // Add server arguments
        for arg in self.config.server_args.split_whitespace() {
            cmd.arg(arg);
        }

        // Set working directory
        let server_dir = PathBuf::from(&self.config.host);
        cmd.current_dir(&server_dir);

        // Set up process
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Start the process
        let child = cmd.spawn()?;
        let process = Arc::new(Mutex::new(Some(child)));
        self.process = Some(process.clone());

        // Start monitoring task
        let server_id = self.id.clone();
        let db_clone = db.clone();
        tokio::spawn(async move {
            Self::monitor_process(process, server_id, db_clone).await;
        });

        // Wait a moment for the server to start
        sleep(Duration::from_secs(2)).await;

        // Check if the process is still running
        if let Some(process) = &self.process {
            let mut process_guard = process.lock().await;
            if let Some(child) = process_guard.as_mut() {
                match child.try_wait() {
                    Ok(Some(_)) => {
                        self.status = ServerStatus::Crashed;
                        return Err(anyhow!("Server process exited immediately"));
                    }
                    Ok(None) => {
                        self.status = ServerStatus::Running;
                        info!("Minecraft server started successfully: {}", self.id);
                        
                        // Log successful start
                        let event = EventLog {
                            id: Uuid::new_v4().to_string(),
                            server_id: Some(self.id.clone()),
                            event_type: "server_started".to_string(),
                            message: "Server started successfully".to_string(),
                            level: "info".to_string(),
                            metadata: None,
                            created_at: chrono::Utc::now(),
                        };
                        db.log_event(&event).await?;
                    }
                    Err(e) => {
                        self.status = ServerStatus::Crashed;
                        return Err(anyhow!("Failed to check server process: {}", e));
                    }
                }
            }
        }

        Ok(())
    }

    /// Stop the Minecraft server
    pub async fn stop(&mut self, db: &DatabaseManager) -> Result<()> {
        if self.status == ServerStatus::Stopped {
            return Ok(());
        }

        info!("Stopping Minecraft server: {}", self.id);
        
        self.status = ServerStatus::Stopping;

        // Log the stop event
        let event = EventLog {
            id: Uuid::new_v4().to_string(),
            server_id: Some(self.id.clone()),
            event_type: "server_stop".to_string(),
            message: "Server stopping".to_string(),
            level: "info".to_string(),
            metadata: None,
            created_at: chrono::Utc::now(),
        };
        db.log_event(&event).await?;

        let mut should_clear_process = false;
        if let Some(process) = &self.process {
            let mut process_guard = process.lock().await;
            if let Some(child) = process_guard.as_mut() {
                // Try graceful shutdown first
                if let Some(stdin) = child.stdin.as_mut() {
                    use std::io::Write;
                    let _ = stdin.write_all(b"stop\n");
                    let _ = stdin.flush();
                }

                // Wait for graceful shutdown
                for _ in 0..30 {
                    match child.try_wait() {
                        Ok(Some(_)) => {
                            self.status = ServerStatus::Stopped;
                            should_clear_process = true;
                            info!("Minecraft server stopped gracefully: {}", self.id);
                            
                            // Log successful stop
                            let event = EventLog {
                                id: Uuid::new_v4().to_string(),
                                server_id: Some(self.id.clone()),
                                event_type: "server_stopped".to_string(),
                                message: "Server stopped successfully".to_string(),
                                level: "info".to_string(),
                                metadata: None,
                                created_at: chrono::Utc::now(),
                            };
                            db.log_event(&event).await?;
                            break;
                        }
                        Ok(None) => {
                            sleep(Duration::from_secs(1)).await;
                        }
                        Err(e) => {
                            error!("Error checking server process: {}", e);
                            break;
                        }
                    }
                }

                // Force kill if graceful shutdown failed
                if !should_clear_process {
                    let _ = child.kill();
                    self.status = ServerStatus::Stopped;
                    should_clear_process = true;
                    warn!("Minecraft server force killed: {}", self.id);
                }
            }
        }
        
        if should_clear_process {
            self.process = None;
        }

        Ok(())
    }

    /// Restart the Minecraft server
    pub async fn restart(&mut self, db: &DatabaseManager) -> Result<()> {
        info!("Restarting Minecraft server: {}", self.id);
        
        self.restart_count += 1;
        self.last_restart = Some(Instant::now());

        // Log the restart event
        let event = EventLog {
            id: Uuid::new_v4().to_string(),
            server_id: Some(self.id.clone()),
            event_type: "server_restart".to_string(),
            message: format!("Server restarting (count: {})", self.restart_count),
            level: "info".to_string(),
            metadata: Some(serde_json::json!({"restart_count": self.restart_count})),
            created_at: chrono::Utc::now(),
        };
        db.log_event(&event).await?;

        self.stop(db).await?;
        sleep(Duration::from_secs(2)).await;
        self.start(db).await?;

        Ok(())
    }

    /// Send a command to the server
    pub async fn send_command(&self, command: &str) -> Result<String> {
        if self.status != ServerStatus::Running {
            return Err(anyhow!("Server is not running"));
        }

        let rcon = RconClient::new(
            self.config.host.clone(),
            self.config.rcon_port,
            self.config.rcon_password.clone(),
        );

        rcon.send_command(command)
    }

    /// Get server metrics
    pub async fn get_metrics(&self) -> Result<ServerMetrics> {
        if self.status != ServerStatus::Running {
            return Err(anyhow!("Server is not running"));
        }

        // Get TPS
        let tps_response = self.send_command("tps").await?;
        let tps = self.parse_tps(&tps_response)?;

        // Get player count
        let list_response = self.send_command("list").await?;
        let players_online = self.parse_player_count(&list_response)?;

        // Get heap usage (simulated for now)
        let heap_mb = 2048; // TODO: Get actual heap usage

        // Get tick time (simulated for now)
        let tick_p95 = 45.2; // TODO: Get actual tick time

        // Get GPU queue time (simulated for now)
        let gpu_queue_ms = 5.2; // TODO: Get actual GPU queue time

        // Calculate uptime
        let uptime_seconds = if let Some(start_time) = self.last_start {
            start_time.elapsed().as_secs()
        } else {
            0
        };

        Ok(ServerMetrics {
            tps,
            tick_p95,
            heap_mb,
            players_online,
            gpu_queue_ms,
            uptime_seconds,
            last_update: chrono::Utc::now(),
        })
    }

    /// Get player list
    pub async fn get_players(&self) -> Result<Vec<Player>> {
        if self.status != ServerStatus::Running {
            return Err(anyhow!("Server is not running"));
        }

        let rcon = RconClient::new(
            self.config.host.clone(),
            self.config.rcon_port,
            self.config.rcon_password.clone(),
        );

        let rcon_players = rcon.get_players()?;
        let players = rcon_players.into_iter().map(|p| Player {
            uuid: p.uuid,
            name: p.name,
            dimension: p.dimension,
            last_seen: p.last_seen,
            online: p.online,
            playtime: p.playtime,
            ping: p.ping,
            x: p.x,
            y: p.y,
            z: p.z,
        }).collect();
        Ok(players)
    }

    /// Parse TPS from server response
    fn parse_tps(&self, response: &str) -> Result<f64> {
        // Parse TPS from response like "TPS: 20.0 (1m, 5m, 15m)"
        if let Some(tps_part) = response.split_whitespace().nth(1) {
            if let Some(tps_str) = tps_part.strip_suffix(',') {
                return Ok(tps_str.parse()?);
            }
            return Ok(tps_part.parse()?);
        }
        Ok(20.0) // Default TPS
    }

    /// Parse player count from server response
    fn parse_player_count(&self, response: &str) -> Result<u32> {
        // Parse player count from response like "There are 5 of a max of 20 players online: ..."
        if let Some(count_part) = response.split_whitespace().nth(2) {
            return Ok(count_part.parse()?);
        }
        Ok(0)
    }

    /// Parse player list from server response
    fn parse_players(&self, response: &str) -> Result<Vec<Player>> {
        let mut players = Vec::new();
        
        // Parse players from response like "There are 5 of a max of 20 players online: TestPlayer, Player2, Player3, Player4, Player5"
        if let Some(players_part) = response.split(": ").nth(1) {
            for player_name in players_part.split(", ") {
                let name = player_name.trim();
                if !name.is_empty() {
                    players.push(Player {
                        uuid: Uuid::new_v4().to_string(), // TODO: Get actual UUID
                        name: name.to_string(),
                        online: true,
                        last_seen: Some(chrono::Utc::now()),
                        playtime: None,
                        ping: None,
                        dimension: None,
                        x: None,
                        y: None,
                        z: None,
                    });
                }
            }
        }
        
        Ok(players)
    }

    /// Monitor server process
    async fn monitor_process(
        process: Arc<Mutex<Option<Child>>>,
        server_id: String,
        db: DatabaseManager,
    ) {
        let mut interval = interval(Duration::from_secs(5));
        
        loop {
            interval.tick().await;
            
            let mut process_guard = process.lock().await;
            if let Some(child) = process_guard.as_mut() {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        error!("Server process exited unexpectedly: {} (status: {:?})", server_id, status);
                        
                        // Log the crash
                        let event = EventLog {
                            id: Uuid::new_v4().to_string(),
                            server_id: Some(server_id.clone()),
                            event_type: "server_crash".to_string(),
                            message: format!("Server process crashed with status: {:?}", status),
                            level: "error".to_string(),
                            metadata: Some(serde_json::json!({"exit_status": format!("{:?}", status)})),
                            created_at: chrono::Utc::now(),
                        };
                        
                        if let Err(e) = db.log_event(&event).await {
                            error!("Failed to log server crash event: {}", e);
                        }
                        
                        *process_guard = None;
                        break;
                    }
                    Ok(None) => {
                        // Process is still running
                        debug!("Server process is running: {}", server_id);
                    }
                    Err(e) => {
                        error!("Error checking server process: {} - {}", server_id, e);
                        break;
                    }
                }
            } else {
                // Process is no longer available
                break;
            }
        }
    }
}

/// Minecraft server manager
#[derive(Debug, Clone)]
pub struct MinecraftManager {
    servers: Arc<RwLock<HashMap<String, MinecraftServer>>>,
    db: DatabaseManager,
}

impl MinecraftManager {
    /// Create a new Minecraft manager
    pub fn new(db: DatabaseManager) -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
            db,
        }
    }

    /// Load servers from database
    pub async fn load_servers(&self) -> Result<()> {
        let server_configs = self.db.get_all_servers().await?;
        
        let mut servers = self.servers.write().await;
        for config in server_configs {
            let server = MinecraftServer::new(config);
            servers.insert(server.id.clone(), server);
        }
        
        info!("Loaded {} servers from database", servers.len());
        Ok(())
    }

    /// Add a new server
    pub async fn add_server(&self, config: ServerConfig) -> Result<()> {
        self.db.create_server(&config).await?;
        
        let server = MinecraftServer::new(config.clone());
        let mut servers = self.servers.write().await;
        servers.insert(server.id.clone(), server);
        
        info!("Added server: {}", config.id);
        Ok(())
    }

    /// Remove a server from memory
    pub async fn remove_server(&self, id: &str) -> Result<()> {
        let mut servers = self.servers.write().await;
        if servers.remove(id).is_some() {
            info!("Removed server from memory: {}", id);
            Ok(())
        } else {
            Err(anyhow!("Server not found in memory: {}", id))
        }
    }

    /// Get a server by ID
    pub async fn get_server(&self, id: &str) -> Option<MinecraftServer> {
        let servers = self.servers.read().await;
        servers.get(id).cloned()
    }

    /// Get all servers
    pub async fn get_all_servers(&self) -> Vec<MinecraftServer> {
        let servers = self.servers.read().await;
        servers.values().cloned().collect()
    }

    /// Start a server
    pub async fn start_server(&self, id: &str) -> Result<()> {
        let mut servers = self.servers.write().await;
        if let Some(server) = servers.get_mut(id) {
            server.start(&self.db).await?;
        } else {
            return Err(anyhow!("Server not found: {}", id));
        }
        Ok(())
    }

    /// Stop a server
    pub async fn stop_server(&self, id: &str) -> Result<()> {
        let mut servers = self.servers.write().await;
        if let Some(server) = servers.get_mut(id) {
            server.stop(&self.db).await?;
        } else {
            return Err(anyhow!("Server not found: {}", id));
        }
        Ok(())
    }

    /// Restart a server
    pub async fn restart_server(&self, id: &str) -> Result<()> {
        let mut servers = self.servers.write().await;
        if let Some(server) = servers.get_mut(id) {
            server.restart(&self.db).await?;
        } else {
            return Err(anyhow!("Server not found: {}", id));
        }
        Ok(())
    }

    /// Send command to a server
    pub async fn send_command(&self, id: &str, command: &str) -> Result<String> {
        let servers = self.servers.read().await;
        if let Some(server) = servers.get(id) {
            server.send_command(command).await
        } else {
            Err(anyhow!("Server not found: {}", id))
        }
    }

    /// Get server metrics
    pub async fn get_server_metrics(&self, id: &str) -> Result<ServerMetrics> {
        let servers = self.servers.read().await;
        if let Some(server) = servers.get(id) {
            server.get_metrics().await
        } else {
            Err(anyhow!("Server not found: {}", id))
        }
    }

    /// Get server players
    pub async fn get_server_players(&self, id: &str) -> Result<Vec<Player>> {
        let servers = self.servers.read().await;
        if let Some(server) = servers.get(id) {
            server.get_players().await
        } else {
            Err(anyhow!("Server not found: {}", id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_minecraft_server_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let database_url = format!("sqlite:{}", db_path.display());
        
        let db = DatabaseManager::new(&database_url).await.unwrap();
        
        let config = ServerConfig {
            id: "test-server".to_string(),
            name: "Test Server".to_string(),
            host: "/tmp/test-server".to_string(),
            port: 25565,
            rcon_port: 25575,
            rcon_password: "password".to_string(),
            java_path: "/usr/bin/java".to_string(),
            server_jar: "server.jar".to_string(),
            jvm_args: "-Xmx4G".to_string(),
            server_args: "nogui".to_string(),
            auto_start: true,
            auto_restart: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        let server = MinecraftServer::new(config);
        assert_eq!(server.id, "test-server");
        assert_eq!(server.status, ServerStatus::Stopped);
    }

    #[tokio::test]
    async fn test_rcon_client() {
        let rcon = RconClient::new("localhost".to_string(), 25575, "password".to_string());
        
        let response = rcon.send_command("list").await.unwrap();
        assert!(!response.is_empty());
        
        let available = rcon.is_available().await;
        assert!(available);
    }

    #[tokio::test]
    async fn test_minecraft_manager() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let database_url = format!("sqlite:{}", db_path.display());
        
        let db = DatabaseManager::new(&database_url).await.unwrap();
        let manager = MinecraftManager::new(db);
        
        let servers = manager.get_all_servers().await;
        assert_eq!(servers.len(), 0);
    }
}
