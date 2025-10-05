use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;
use tokio::process::Command as TokioCommand;
use tokio::io::{AsyncBufReadExt, BufReader, AsyncWriteExt};
use tokio::process::Child as TokioChild;
use std::path::PathBuf;
use std::fs;
use serde_json;
use std::process::Stdio;

use crate::database::ServerConfig;
use crate::core::{
    error_handler::{AppError, Result},
    websocket::WebSocketManager,
};

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub id: Uuid,
    pub name: String,
    pub pid: u32,
    pub tps: f32,
    pub tick_p95: f32,
    pub heap_mb: u32,
    pub players_online: u32,
    pub gpu_queue_ms: f32,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub uptime: Duration,
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
struct ServerProcess {
    child: TokioChild,
    start_time: Instant,
    last_heartbeat: chrono::DateTime<chrono::Utc>,
    rcon_port: u16,
    rcon_password: String,
}

#[derive(Debug)]
pub struct ProcessManager {
    processes: Arc<RwLock<HashMap<Uuid, ServerProcess>>>,
    process_info: Arc<RwLock<HashMap<Uuid, ProcessInfo>>>,
    websocket: Arc<WebSocketManager>,
}

impl ProcessManager {
    pub fn new(websocket: Arc<WebSocketManager>) -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
            process_info: Arc::new(RwLock::new(HashMap::new())),
            websocket,
        }
    }
    
    pub async fn start_server_process(&self, config: ServerConfig) -> Result<()> {
        let server_id = config.id.clone();
        let server_name = config.name.clone();
        
        tracing::info!("Starting server process for: {}", server_name);
        
        // Check if server is already running
        if self.is_server_running(Uuid::parse_str(&server_id)?).await {
            return Err(AppError::ValidationError {
                message: format!("Server {} is already running", server_name),
                field: "server_name".to_string(),
                value: server_name.to_string(),
                constraint: "must not be already running".to_string(),
            });
        }
        
        // Create server directory if it doesn't exist
        let server_dir = self.get_server_directory(Uuid::parse_str(&server_id)?);
        if !server_dir.exists() {
            fs::create_dir_all(&server_dir)
                .map_err(|e| AppError::FileSystemError {
                    message: format!("Failed to create server directory: {}", e),
                    path: server_dir.to_string_lossy().to_string(),
                    operation: "create".to_string(),
                })?;
        }
        
        // Download server JAR if needed
        let jar_path = self.get_server_jar_path(&config)?;
        if !jar_path.exists() {
            self.download_server_jar(&config).await?;
        }
        
        // Create server.properties file
        self.create_server_properties(&config).await?;
        
        // Create eula.txt file
        self.create_eula_file(&server_dir).await?;
        
        // Start the actual Minecraft server process
        let mut cmd = TokioCommand::new("java");
        cmd.current_dir(&server_dir);
        
        // Add JVM arguments
        let java_args: Vec<String> = serde_json::from_str(&config.java_args).unwrap_or_default();
        for arg in &java_args {
            cmd.arg(arg);
        }
        
        // Add memory settings
        cmd.arg(&format!("-Xmx{}M", config.memory));
        cmd.arg(&format!("-Xms{}M", config.memory / 2));
        
        // Add server JAR
        cmd.arg("-jar");
        cmd.arg(&jar_path);
        
        // Add server arguments
        let server_args: Vec<String> = serde_json::from_str(&config.server_args).unwrap_or_default();
        for arg in &server_args {
            cmd.arg(arg);
        }
        
        // Set up process
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        let mut child = cmd.spawn()
            .map_err(|e| AppError::ProcessError {
                message: format!("Failed to start server process: {}", e),
                process_id: None,
                operation: "start".to_string(),
            })?;
        
        let pid = child.id().unwrap_or(0) as u32;
        
        // Create process info
        let process_info = ProcessInfo {
            id: Uuid::parse_str(&server_id)?,
            name: server_name,
            pid,
            tps: 20.0,
            tick_p95: 45.0,
            heap_mb: config.memory,
            players_online: 0,
            gpu_queue_ms: 0.0,
            cpu_usage: 0.0,
            memory_usage: 0.0,
            uptime: Duration::ZERO,
            last_heartbeat: chrono::Utc::now(),
        };
        
        // Store process and info
        let server_process = ServerProcess {
            child,
            start_time: Instant::now(),
            last_heartbeat: chrono::Utc::now(),
            rcon_port: config.rcon_port,
            rcon_password: "N/A".to_string(), // Should be loaded securely
        };
        
        let mut processes = self.processes.write().await;
        let mut process_info_guard = self.process_info.write().await;
        
        processes.insert(Uuid::parse_str(&server_id)?, server_process);
        process_info_guard.insert(Uuid::parse_str(&server_id)?, process_info);
        
        // Start monitoring task
        self.start_monitoring_task(Uuid::parse_str(&server_id)?).await;
        
        // Send status update via WebSocket
        let _ = self.websocket.send_server_status_update(&server_id, "starting").await;
        
        tracing::info!("Server {} started with PID {}", config.name, pid);
        Ok(())
    }
    
    pub async fn stop_server_process(&self, server_id: Uuid) -> Result<()> {
        tracing::info!("Stopping server process: {}", server_id);
        
        let mut processes = self.processes.write().await;
        let mut process_info = self.process_info.write().await;
        
        if let Some(mut process) = processes.remove(&server_id) {
            // Send stop command to the server
            if let Some(mut stdin) = process.child.stdin.take() {
                let _ = stdin.write_all(b"stop\n").await;
            }
            
            // Wait for the process to exit gracefully
            let _ = tokio::time::timeout(Duration::from_secs(30), process.child.wait()).await;
            
            // Force kill if still running
            let _ = process.child.kill().await;
        }
        
        process_info.remove(&server_id);
        
        // Send status update via WebSocket
        let _ = self.websocket.send_server_status_update(&server_id.to_string(), "stopped").await;
        
        tracing::info!("Server {} stopped", server_id);
        Ok(())
    }
    
    pub async fn restart_server_process(&self, server_id: Uuid) -> Result<()> {
        tracing::info!("Restarting server process: {}", server_id);
        
        // Get the server config before stopping
        let config = self.get_server_config(server_id).await?;
        
        // Stop the server
        self.stop_server_process(server_id).await?;
        
        // Wait a moment
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Start the server again
        self.start_server_process(config).await?;
        
        tracing::info!("Server {} restarted", server_id);
        Ok(())
    }
    
    pub async fn is_server_running(&self, server_id: Uuid) -> bool {
        let processes = self.processes.read().await;
        processes.contains_key(&server_id)
    }
    
    pub async fn get_process_info(&self, server_id: Uuid) -> Result<ProcessInfo> {
        let process_info = self.process_info.read().await;
        process_info.get(&server_id)
            .cloned()
            .ok_or_else(|| AppError::ServerError {
                message: "Server not found".to_string(),
                server_id: server_id.to_string(),
                operation: "get".to_string(),
            })
    }
    
    pub async fn send_rcon_command(&self, server_id: Uuid, command: &str) -> Result<()> {
        let processes = self.processes.read().await;
        
        if let Some(process) = processes.get(&server_id) {
            // In a real implementation, this would connect to the RCON port
            // For now, we'll send the command to stdin
            tracing::info!("RCON command for server {}: {}", server_id, command);
            
            // TODO: Implement actual RCON communication
            // This would require connecting to the RCON port and sending the command
        } else {
            return Err(AppError::ServerError {
                message: "Server not found".to_string(),
                server_id: server_id.to_string(),
                operation: "rcon".to_string(),
            });
        }
        
        Ok(())
    }
    
    pub async fn get_all_processes(&self) -> Result<Vec<ProcessInfo>> {
        let process_info = self.process_info.read().await;
        Ok(process_info.values().cloned().collect())
    }
    
    pub async fn update_server_metrics(&self, server_id: Uuid, metrics: ProcessMetrics) -> Result<()> {
        let mut process_info = self.process_info.write().await;
        
        if let Some(info) = process_info.get_mut(&server_id) {
            info.tps = metrics.tps;
            info.tick_p95 = metrics.tick_p95;
            info.heap_mb = metrics.heap_mb;
            info.players_online = metrics.players_online;
            info.gpu_queue_ms = metrics.gpu_queue_ms;
            info.cpu_usage = metrics.cpu_usage;
            info.memory_usage = metrics.memory_usage;
            info.last_heartbeat = chrono::Utc::now();
        }
        
        Ok(())
    }
    
    fn get_server_directory(&self, server_id: Uuid) -> PathBuf {
        PathBuf::from("servers").join(server_id.to_string())
    }
    
    fn get_server_jar_path(&self, config: &ServerConfig) -> Result<PathBuf> {
        let server_dir = self.get_server_directory(Uuid::parse_str(&config.id)?);
        Ok(server_dir.join(format!("server-{}.jar", config.minecraft_version)))
    }
    
    async fn download_server_jar(&self, config: &ServerConfig) -> Result<()> {
        let jar_path = self.get_server_jar_path(config)?;
        
        tracing::info!("Downloading server JAR for {} {}", config.loader, config.minecraft_version);
        
        // For now, create a placeholder JAR file
        // In a real implementation, this would download from Mojang/Modrinth/CurseForge
        let placeholder_content = b"# Placeholder Minecraft Server JAR\n# In production, this would be the actual server JAR";
        
        fs::write(&jar_path, placeholder_content)
            .map_err(|e| AppError::FileSystemError {
                message: format!("Failed to create placeholder JAR: {}", e),
                path: jar_path.to_string_lossy().to_string(),
                operation: "create".to_string(),
            })?;
        
        tracing::info!("Created placeholder JAR at {:?}", jar_path);
        Ok(())
    }
    
    async fn create_server_properties(&self, config: &ServerConfig) -> Result<()> {
        let server_dir = self.get_server_directory(Uuid::parse_str(&config.id)?);
        let properties_path = server_dir.join("server.properties");
        
        let properties = format!(
            r#"#Minecraft server properties
#Generated by Guardian Server Manager
server-port={}
rcon.port={}
rcon.password={}
query.port={}
max-players={}
motd={}
difficulty={}
gamemode={}
pvp={}
online-mode={}
white-list={}
enable-command-block={}
view-distance={}
simulation-distance={}
"#,
            config.port,
            config.rcon_port,
            "guardian123", // TODO: Generate secure password
            config.query_port,
            config.max_players,
            config.motd,
            config.difficulty,
            config.gamemode,
            config.pvp,
            config.online_mode,
            config.whitelist,
            config.enable_command_block,
            config.view_distance,
            config.simulation_distance,
        );
        
        fs::write(&properties_path, properties)
            .map_err(|e| AppError::FileSystemError {
                message: format!("Failed to create server.properties: {}", e),
                path: properties_path.to_string_lossy().to_string(),
                operation: "create".to_string(),
            })?;
        
        Ok(())
    }
    
    async fn create_eula_file(&self, server_dir: &PathBuf) -> Result<()> {
        let eula_path = server_dir.join("eula.txt");
        let eula_content = "eula=true\n";
        
        fs::write(&eula_path, eula_content)
            .map_err(|e| AppError::FileSystemError {
                message: format!("Failed to create eula.txt: {}", e),
                path: eula_path.to_string_lossy().to_string(),
                operation: "create".to_string(),
            })?;
        
        Ok(())
    }
    
    async fn get_server_config(&self, server_id: Uuid) -> Result<ServerConfig> {
        // In a real implementation, this would load from the database
        // For now, return a default config
        Ok(ServerConfig {
            id: server_id.to_string(),
            name: "Minecraft Server".to_string(),
            minecraft_version: "1.21.1".to_string(),
            loader: "vanilla".to_string(),
            loader_version: "1.21.1".to_string(),
            port: 25565,
            rcon_port: 25575,
            query_port: 25566,
            max_players: 20,
            memory: 2048,
            java_args: serde_json::to_string(&vec!["-XX:+UseG1GC".to_string()]).unwrap_or_default(),
            server_args: serde_json::to_string(&vec!["nogui".to_string()]).unwrap_or_default(),
            auto_start: false,
            auto_restart: false,
            world_name: "world".to_string(),
            difficulty: "normal".to_string(),
            gamemode: "survival".to_string(),
            pvp: true,
            online_mode: true,
            whitelist: false,
            enable_command_block: false,
            view_distance: 10,
            simulation_distance: 10,
            motd: "A Minecraft Server".to_string(),
            host: "0.0.0.0".to_string(),
            java_path: "java".to_string(),
            jvm_args: serde_json::to_string(&vec!["-Xmx2G".to_string()]).unwrap_or_default(),
            server_jar: "server.jar".to_string(),
            rcon_password: "password".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }
    
    async fn start_monitoring_task(&self, server_id: Uuid) {
        let processes = self.processes.clone();
        let process_info = self.process_info.clone();
        let websocket = self.websocket.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                
                // Check if server is still running
                let mut processes_guard = processes.write().await;
                let mut info_guard = process_info.write().await;
                
                if let Some(process) = processes_guard.get_mut(&server_id) {
                    // Check if process is still alive
                    if process.child.try_wait().unwrap_or(None).is_some() {
                        // Process has exited
                        processes_guard.remove(&server_id);
                        info_guard.remove(&server_id);
                        
                        // Send status update
                        let _ = websocket.send_server_status_update(&server_id.to_string(), "stopped").await;
                        
                        tracing::info!("Server {} process exited", server_id);
                        break;
                    }
                    
                    // Update uptime and send metrics
                    if let Some(info) = info_guard.get_mut(&server_id) {
                        info.uptime = process.start_time.elapsed();
                        info.last_heartbeat = chrono::Utc::now();
                        
                        // Send real-time metrics via WebSocket
                        let metrics = serde_json::json!({
                            "tps": info.tps,
                            "tick_p95": info.tick_p95,
                            "heap_mb": info.heap_mb,
                            "players_online": info.players_online,
                            "gpu_queue_ms": info.gpu_queue_ms,
                            "cpu_usage": info.cpu_usage,
                            "memory_usage": info.memory_usage,
                            "uptime_seconds": info.uptime.as_secs(),
                            "timestamp": chrono::Utc::now()
                        });
                        
                        let _ = websocket.send_server_metrics(&server_id.to_string(), metrics).await;
                    }
                } else {
                    // Server no longer exists
                    break;
                }
            }
        });
    }
    
    async fn start_console_streaming(&self, server_id: Uuid, mut child: TokioChild) {
        let websocket = self.websocket.clone();
        
        // Stream stdout
        if let Some(stdout) = child.stdout.take() {
            let websocket_clone = websocket.clone();
            tokio::spawn(async move {
                let mut reader = BufReader::new(stdout);
                let mut line = String::new();
                
                while let Ok(n) = reader.read_line(&mut line).await {
                    if n == 0 {
                        break; // EOF
                    }
                    
                    // Send console message via WebSocket
                    let _ = websocket_clone.send_console_message(&server_id.to_string(), "info", line.trim()).await;
                    
                    // Log to database
                    // TODO: Add database logging here
                    
                    line.clear();
                }
            });
        }
        
        // Stream stderr
        if let Some(stderr) = child.stderr.take() {
            let websocket_clone = websocket.clone();
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr);
                let mut line = String::new();
                
                while let Ok(n) = reader.read_line(&mut line).await {
                    if n == 0 {
                        break; // EOF
                    }
                    
                    // Send console message via WebSocket
                    let _ = websocket_clone.send_console_message(&server_id.to_string(), "error", line.trim()).await;
                    
                    // Log to database
                    // TODO: Add database logging here
                    
                    line.clear();
                }
            });
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessMetrics {
    pub tps: f32,
    pub tick_p95: f32,
    pub heap_mb: u32,
    pub players_online: u32,
    pub gpu_queue_ms: f32,
    pub cpu_usage: f32,
    pub memory_usage: f32,
}