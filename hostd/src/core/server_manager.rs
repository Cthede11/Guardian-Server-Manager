use std::process::Stdio;
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;
use tokio::process::Command as TokioCommand;
use tokio::time::sleep;
use serde_json;

use crate::database::{DatabaseManager, ServerConfig};
use crate::core::{
    file_manager::FileManager,
    process_manager::ProcessManager,
    error_handler::{AppError, Result},
};

#[derive(Debug, Clone)]
pub struct ServerManager {
    database: std::sync::Arc<DatabaseManager>,
    file_manager: std::sync::Arc<FileManager>,
    process_manager: std::sync::Arc<ProcessManager>,
}

impl ServerManager {
    pub fn new(
        database: std::sync::Arc<DatabaseManager>,
        file_manager: std::sync::Arc<FileManager>,
        process_manager: std::sync::Arc<ProcessManager>,
    ) -> Self {
        Self {
            database,
            file_manager,
            process_manager,
        }
    }
    
    pub async fn create_server(&self, config: ServerConfig) -> Result<Uuid> {
        // Validate configuration
        self.validate_server_config(&config).await?;
        
        // Create server directory structure
        self.file_manager.create_server_directory(Uuid::parse_str(&config.id)?).await?;
        
        // Download Minecraft server JAR
        self.download_server_jar(&config).await?;
        
        // Create server.properties file
        self.create_server_properties(&config).await?;
        
        // Create eula.txt file
        self.create_eula_file(&config).await?;
        
        // Create startup script
        self.create_startup_script(&config).await?;
        
        // Save to database
        self.database.create_server(&config).await?;
        
        // Log server creation
        self.database.log_server_message(
            &config.id,
            "INFO",
            &format!("Server '{}' created successfully", config.name),
            Some("ServerManager"),
        ).await?;
        
        Ok(Uuid::parse_str(&config.id)?)
    }
    
    pub async fn start_server(&self, server_id: Uuid) -> Result<()> {
        // Get server configuration
        let config = self.database.get_server(&server_id.to_string()).await?
            .ok_or_else(|| AppError::ServerError {
                message: "Server not found".to_string(),
                server_id: server_id.to_string(),
                operation: "get".to_string(),
            })?;
        
        // Check if server is already running
        if self.process_manager.is_server_running(server_id).await {
            return Err(AppError::ServerError {
                message: "Server is already running".to_string(),
                server_id: server_id.to_string(),
                operation: "start".to_string(),
            });
        }
        
        // Start the server process using ProcessManager
        self.process_manager.start_server_process(config).await?;
        
        // Log server start
        self.database.log_server_message(
            &server_id.to_string(),
            "INFO",
            &format!("Server '{}' started", server_id),
            Some("ServerManager"),
        ).await?;
        
        Ok(())
    }
    
    pub async fn stop_server(&self, server_id: Uuid) -> Result<()> {
        // Check if server is running
        if !self.process_manager.is_server_running(server_id).await {
            return Err(AppError::ServerError {
                message: "Server is not running".to_string(),
                server_id: server_id.to_string(),
                operation: "stop".to_string(),
            });
        }
        
        // Stop the server process using ProcessManager
        self.process_manager.stop_server_process(server_id).await?;
        
        // Log server stop
        self.database.log_server_message(
            &server_id.to_string(),
            "INFO",
            &format!("Server '{}' stopped", server_id),
            Some("ServerManager"),
        ).await?;
        
        Ok(())
    }
    
    pub async fn restart_server(&self, server_id: Uuid) -> Result<()> {
        // Stop server first
        if let Err(e) = self.stop_server(server_id).await {
            // Log error but continue with restart
            self.database.log_server_message(
                &server_id.to_string(),
                "WARN",
                &format!("Error stopping server during restart: {}", e),
                Some("ServerManager"),
            ).await?;
        }
        
        // Wait a moment for cleanup
        sleep(Duration::from_secs(2)).await;
        
        // Start server again
        self.start_server(server_id).await?;
        
        Ok(())
    }
    
    pub async fn delete_server(&self, server_id: Uuid) -> Result<()> {
        // Stop server if running
        if self.process_manager.is_server_running(server_id).await {
            let _ = self.stop_server(server_id).await;
        }
        
        // Remove from process manager
        // ProcessManager no longer has unregister_server method
        
        // Delete server files
        self.file_manager.delete_server_directory(server_id).await?;
        
        // Delete from database
        self.database.delete_server(&server_id.to_string()).await?;
        
        Ok(())
    }
    
    pub async fn send_command(&self, server_id: Uuid, command: &str) -> Result<()> {
        // Check if server is running
        if !self.process_manager.is_server_running(server_id).await {
            return Err(AppError::ServerError {
                message: "Server is not running".to_string(),
                server_id: server_id.to_string(),
                operation: "stop".to_string(),
            });
        }
        
        // Send command via RCON
        self.process_manager.send_rcon_command(server_id, command).await?;
        
        // Log command
        self.database.log_server_message(
            &server_id.to_string(),
            "INFO",
            &format!("Command sent: {}", command),
            Some("RCON"),
        ).await?;
        
        Ok(())
    }
    
    pub async fn get_server_status(&self, server_id: Uuid) -> Result<ServerStatus> {
        let is_running = self.process_manager.is_server_running(server_id).await;
        
        if is_running {
            // Get process information
            let process_info = self.process_manager.get_process_info(server_id).await?;
            
            Ok(ServerStatus {
                id: server_id,
                name: process_info.name,
                status: "running".to_string(),
                tps: process_info.tps,
                tick_p95: process_info.tick_p95,
                heap_mb: process_info.heap_mb,
                players_online: process_info.players_online,
                gpu_queue_ms: process_info.gpu_queue_ms,
                cpu_usage: process_info.cpu_usage,
                memory_usage: process_info.memory_usage,
                uptime: process_info.uptime,
                last_heartbeat: process_info.last_heartbeat,
            })
        } else {
            Ok(ServerStatus {
                id: server_id,
                name: "Unknown".to_string(),
                status: "stopped".to_string(),
                tps: 0.0,
                tick_p95: 0.0,
                heap_mb: 0,
                players_online: 0,
                gpu_queue_ms: 0.0,
                cpu_usage: 0.0,
                memory_usage: 0.0,
                uptime: Duration::ZERO,
                last_heartbeat: chrono::Utc::now(),
            })
        }
    }
    
    async fn validate_server_config(&self, config: &ServerConfig) -> Result<()> {
        if config.name.is_empty() {
            return Err(AppError::ValidationError {
                message: "Server name cannot be empty".to_string(),
                field: "name".to_string(),
                value: "".to_string(),
                constraint: "must not be empty".to_string(),
            });
        }
        
        if config.minecraft_version.is_empty() {
            return Err(AppError::ValidationError {
                message: "Minecraft version cannot be empty".to_string(),
                field: "minecraft_version".to_string(),
                value: "".to_string(),
                constraint: "must not be empty".to_string(),
            });
        }
        
        if config.port == 0 {
            return Err(AppError::ValidationError {
                message: "Port cannot be 0".to_string(),
                field: "port".to_string(),
                value: config.port.to_string(),
                constraint: "must be greater than 0".to_string(),
            });
        }
        
        if config.max_players == 0 {
            return Err(AppError::ValidationError {
                message: "Max players cannot be 0".to_string(),
                field: "max_players".to_string(),
                value: config.max_players.to_string(),
                constraint: "must be greater than 0".to_string(),
            });
        }
        
        if config.memory < 512 {
            return Err(AppError::ValidationError {
                message: "Memory must be at least 512MB".to_string(),
                field: "memory".to_string(),
                value: config.memory.to_string(),
                constraint: "must be at least 512".to_string(),
            });
        }
        
        Ok(())
    }
    
    async fn download_server_jar(&self, config: &ServerConfig) -> Result<()> {
        let server_dir = self.file_manager.get_server_directory(Uuid::parse_str(&config.id)?);
        let jar_path = server_dir.join("server.jar");
        
        // Check if JAR already exists
        if jar_path.exists() {
            return Ok(());
        }
        
        // Download server JAR based on loader type
        match config.loader.as_str() {
            "vanilla" => self.download_vanilla_jar(&config.minecraft_version, &jar_path).await?,
            "forge" => self.download_forge_jar(&config.minecraft_version, &config.loader_version, &jar_path).await?,
            "fabric" => self.download_fabric_jar(&config.minecraft_version, &config.loader_version, &jar_path).await?,
            "quilt" => self.download_quilt_jar(&config.minecraft_version, &config.loader_version, &jar_path).await?,
            "paper" => self.download_paper_jar(&config.minecraft_version, &jar_path).await?,
            _ => return Err(AppError::ValidationError {
                message: format!("Unsupported loader: {}", config.loader),
                field: "loader".to_string(),
                value: config.loader.clone(),
                constraint: "must be one of: vanilla, forge, fabric, quilt".to_string(),
            }),
        }
        
        Ok(())
    }
    
    async fn download_vanilla_jar(&self, version: &str, jar_path: &PathBuf) -> Result<()> {
        // Use Mojang's version manifest to get download URL
        let manifest_url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
        let response = reqwest::get(manifest_url).await
            .map_err(|e| AppError::NetworkError {
                message: format!("Failed to fetch version manifest: {}", e),
                endpoint: "https://launchermeta.mojang.com/mc/game/version_manifest.json".to_string(),
                status_code: None,
            })?;
        
        let manifest: serde_json::Value = response.json().await
            .map_err(|e| AppError::NetworkError {
                message: format!("Failed to parse version manifest: {}", e),
                endpoint: "https://launchermeta.mojang.com/mc/game/version_manifest.json".to_string(),
                status_code: None,
            })?;
        
        // Find the version
        let versions = manifest["versions"].as_array()
            .ok_or_else(|| AppError::ValidationError {
                message: "Invalid version manifest".to_string(),
                field: "versions".to_string(),
                value: "null".to_string(),
                constraint: "must be an array".to_string(),
            })?;
        
        let version_info = versions.iter()
            .find(|v| v["id"].as_str() == Some(version))
            .ok_or_else(|| AppError::ValidationError {
                message: format!("Version {} not found", version),
                field: "version".to_string(),
                value: version.to_string(),
                constraint: "must exist in version manifest".to_string(),
            })?;
        
        let version_url = version_info["url"].as_str()
            .ok_or_else(|| AppError::ValidationError {
                message: "Invalid version URL".to_string(),
                field: "url".to_string(),
                value: "null".to_string(),
                constraint: "must be a string".to_string(),
            })?;
        
        // Get version details
        let version_response = reqwest::get(version_url).await
            .map_err(|e| AppError::NetworkError {
                message: format!("Failed to fetch version details: {}", e),
                endpoint: version_url.to_string(),
                status_code: None,
            })?;
        
        let version_details: serde_json::Value = version_response.json().await
            .map_err(|e| AppError::NetworkError {
                message: format!("Failed to parse version details: {}", e),
                endpoint: version_url.to_string(),
                status_code: None,
            })?;
        
        let jar_url = version_details["downloads"]["server"]["url"].as_str()
            .ok_or_else(|| AppError::ValidationError {
                message: "Server JAR URL not found".to_string(),
                field: "downloads.server.url".to_string(),
                value: "null".to_string(),
                constraint: "must be a string".to_string(),
            })?;
        
        // Download the JAR
        let jar_response = reqwest::get(jar_url).await
            .map_err(|e| AppError::NetworkError {
                message: format!("Failed to download server JAR: {}", e),
                endpoint: jar_url.to_string(),
                status_code: None,
            })?;
        
        let jar_data = jar_response.bytes().await
            .map_err(|e| AppError::NetworkError {
                message: format!("Failed to read JAR data: {}", e),
                endpoint: jar_url.to_string(),
                status_code: None,
            })?;
        
        std::fs::write(jar_path, jar_data)
            .map_err(|e| AppError::FileSystemError {
                message: format!("Failed to write server JAR: {}", e),
                path: jar_path.to_string_lossy().to_string(),
                operation: "write".to_string(),
            })?;
        
        Ok(())
    }
    
    async fn download_forge_jar(&self, mc_version: &str, forge_version: &str, jar_path: &PathBuf) -> Result<()> {
        use crate::loaders::LoaderInstaller;
        
        // Detect Java installation
        let java_path = LoaderInstaller::detect_java().await?;
        let installer = LoaderInstaller::new(java_path);
        
        // Get server directory
        let server_dir = jar_path.parent().ok_or_else(|| AppError::ValidationError {
            message: "Invalid JAR path".to_string(),
            field: "jar_path".to_string(),
            value: jar_path.to_string_lossy().to_string(),
            constraint: "must have parent directory".to_string(),
        })?;
        
        // Install Forge server
        let _server_jar = installer.install_forge_server(mc_version, forge_version, server_dir).await?;
        
        Ok(())
    }
    
    async fn download_fabric_jar(&self, mc_version: &str, fabric_version: &str, jar_path: &PathBuf) -> Result<()> {
        use crate::loaders::LoaderInstaller;
        
        // Detect Java installation
        let java_path = LoaderInstaller::detect_java().await?;
        let installer = LoaderInstaller::new(java_path);
        
        // Get server directory
        let server_dir = jar_path.parent().ok_or_else(|| AppError::ValidationError {
            message: "Invalid JAR path".to_string(),
            field: "jar_path".to_string(),
            value: jar_path.to_string_lossy().to_string(),
            constraint: "must have parent directory".to_string(),
        })?;
        
        // Install Fabric server
        let _server_jar = installer.install_fabric_server(mc_version, fabric_version, server_dir).await?;
        
        Ok(())
    }
    
    async fn download_quilt_jar(&self, mc_version: &str, quilt_version: &str, jar_path: &PathBuf) -> Result<()> {
        use crate::loaders::LoaderInstaller;
        
        // Detect Java installation
        let java_path = LoaderInstaller::detect_java().await?;
        let installer = LoaderInstaller::new(java_path);
        
        // Get server directory
        let server_dir = jar_path.parent().ok_or_else(|| AppError::ValidationError {
            message: "Invalid JAR path".to_string(),
            field: "jar_path".to_string(),
            value: jar_path.to_string_lossy().to_string(),
            constraint: "must have parent directory".to_string(),
        })?;
        
        // Install Quilt server
        let _server_jar = installer.install_quilt_server(mc_version, quilt_version, server_dir).await?;
        
        Ok(())
    }
    
    async fn download_paper_jar(&self, mc_version: &str, jar_path: &PathBuf) -> Result<()> {
        // Paper uses a different download system
        // This is a simplified implementation
        Err(AppError::InternalError {
            message: "Paper download not implemented yet".to_string(),
            component: "server_manager".to_string(),
            details: Some("Feature not implemented".to_string()),
        })
    }
    
    async fn create_server_properties(&self, config: &ServerConfig) -> Result<()> {
        let server_dir = self.file_manager.get_server_directory(Uuid::parse_str(&config.id)?);
        let properties_path = server_dir.join("server.properties");
        
        let properties = format!(
            r#"#Minecraft server properties
#Generated by Guardian Server Manager
server-port={}
rcon.port={}
rcon.password=guardian123
enable-rcon=true
query.port={}
max-players={}
server-name={}
gamemode={}
difficulty={}
pvp={}
online-mode={}
white-list={}
enable-command-block={}
view-distance={}
simulation-distance={}
motd={}
"#,
            config.port,
            config.rcon_port,
            config.query_port,
            config.max_players,
            config.name,
            config.gamemode,
            config.difficulty,
            config.pvp,
            config.online_mode,
            config.whitelist,
            config.enable_command_block,
            config.view_distance,
            config.simulation_distance,
            config.motd
        );
        
        std::fs::write(&properties_path, properties)
            .map_err(|e| AppError::FileSystemError {
                message: format!("Failed to create server.properties: {}", e),
                path: properties_path.to_string_lossy().to_string(),
                operation: "write".to_string(),
            })?;
        
        Ok(())
    }
    
    async fn create_eula_file(&self, config: &ServerConfig) -> Result<()> {
        let server_dir = self.file_manager.get_server_directory(Uuid::parse_str(&config.id)?);
        let eula_path = server_dir.join("eula.txt");
        
        let eula = "eula=true\n";
        
        std::fs::write(&eula_path, eula)
            .map_err(|e| AppError::FileSystemError {
                message: format!("Failed to create eula.txt: {}", e),
                path: eula_path.to_string_lossy().to_string(),
                operation: "write".to_string(),
            })?;
        
        Ok(())
    }
    
    async fn create_startup_script(&self, config: &ServerConfig) -> Result<()> {
        let server_dir = self.file_manager.get_server_directory(Uuid::parse_str(&config.id)?);
        let script_path = server_dir.join("start.bat");
        
        let java_args = serde_json::from_str::<Vec<String>>(&config.java_args)
            .unwrap_or_else(|_| vec!["-Xmx2G".to_string(), "-Xms1G".to_string()]);
        
        let script = format!(
            r#"@echo off
cd /d "{}"
java {} -jar server.jar nogui
pause
"#,
            server_dir.display(),
            java_args.join(" ")
        );
        
        std::fs::write(script_path, script)
            .map_err(|e| AppError::FileSystemError {
                message: format!("Failed to create startup script: {}", e),
                path: "startup script".to_string(),
                operation: "create".to_string(),
            })?;
        
        Ok(())
    }
    
    async fn validate_server_files(&self, config: &ServerConfig) -> Result<()> {
        let server_dir = self.file_manager.get_server_directory(Uuid::parse_str(&config.id)?);
        let jar_path = server_dir.join("server.jar");
        let properties_path = server_dir.join("server.properties");
        let eula_path = server_dir.join("eula.txt");
        
        if !jar_path.exists() {
            return Err(AppError::FileSystemError {
                message: "Server JAR not found".to_string(),
                path: "server.jar".to_string(),
                operation: "validate".to_string(),
            });
        }
        
        if !properties_path.exists() {
            return Err(AppError::FileSystemError {
                message: "server.properties not found".to_string(),
                path: "server.properties".to_string(),
                operation: "validate".to_string(),
            });
        }
        
        if !eula_path.exists() {
            return Err(AppError::FileSystemError {
                message: "eula.txt not found".to_string(),
                path: "eula.txt".to_string(),
                operation: "validate".to_string(),
            });
        }
        
        Ok(())
    }
    
    async fn start_server_process(&self, config: &ServerConfig) -> Result<u32> {
        let server_dir = self.file_manager.get_server_directory(Uuid::parse_str(&config.id)?);
        let jar_path = server_dir.join("server.jar");
        
        let java_args = serde_json::from_str::<Vec<String>>(&config.java_args)
            .unwrap_or_else(|_| vec!["-Xmx2G".to_string(), "-Xms1G".to_string()]);
        
        let mut cmd = TokioCommand::new("java");
        cmd.args(&java_args);
        cmd.arg("-jar");
        cmd.arg(&jar_path);
        cmd.arg("nogui");
        cmd.current_dir(&server_dir);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        let child = cmd.spawn()
            .map_err(|e| AppError::ProcessError {
                message: format!("Failed to start server process: {}", e),
                process_id: None,
                operation: "start".to_string(),
            })?;
        
        let process_id = child.id()
            .ok_or_else(|| AppError::ProcessError {
                message: "Failed to get process ID".to_string(),
                process_id: None,
                operation: "get_id".to_string(),
            })?;
        
        Ok(process_id)
    }
    
    async fn stop_server_process(&self, server_id: Uuid) -> Result<()> {
        // Send stop command via RCON
        self.process_manager.send_rcon_command(server_id, "stop").await?;
        
        // Wait for process to stop gracefully
        for _ in 0..30 {
            if !self.process_manager.is_server_running(server_id).await {
                return Ok(());
            }
            sleep(Duration::from_secs(1)).await;
        }
        
        // Force kill if still running
        self.process_manager.stop_server_process(server_id).await?;
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ServerStatus {
    pub id: Uuid,
    pub name: String,
    pub status: String,
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
