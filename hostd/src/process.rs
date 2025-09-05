use crate::config::Config;
use anyhow::Result;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration, Instant};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// Process manager for handling Minecraft server and GPU worker processes
pub struct ProcessManager {
    config: Config,
    minecraft_process: Arc<RwLock<Option<Child>>>,
    gpu_worker_process: Arc<RwLock<Option<Child>>>,
    restart_count: Arc<RwLock<u32>>,
    last_restart: Arc<RwLock<Option<Instant>>>,
}

impl ProcessManager {
    /// Create a new process manager
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing Process Manager...");
        
        Ok(Self {
            config,
            minecraft_process: Arc::new(RwLock::new(None)),
            gpu_worker_process: Arc::new(RwLock::new(None)),
            restart_count: Arc::new(RwLock::new(0)),
            last_restart: Arc::new(RwLock::new(None)),
        })
    }
    
    /// Start the process manager
    pub async fn start(&self) -> Result<()> {
        info!("Starting Process Manager...");
        
        // Start GPU worker first
        if self.config.gpu.enabled {
            self.start_gpu_worker().await?;
        }
        
        // Note: Minecraft servers are started on-demand via the API
        // No automatic server startup
        
        info!("Process Manager started successfully");
        Ok(())
    }
    
    /// Stop the process manager
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Process Manager...");
        
        // Stop Minecraft server
        self.stop_minecraft_server().await?;
        
        // Stop GPU worker
        self.stop_gpu_worker().await?;
        
        info!("Process Manager stopped");
        Ok(())
    }
    
    /// Start the Minecraft server process
    async fn start_minecraft_server(&self) -> Result<()> {
        info!("Starting Minecraft server...");
        
        let mut cmd = Command::new("java");
        
        // Add JVM flags
        let heap_size = format!("-Xmx{}G", self.config.minecraft.java.heap_gb);
        cmd.arg(&heap_size);
        
        // Add performance flags based on profile
        match self.config.minecraft.java.flags.as_str() {
            "g1gc-balanced" => {
                cmd.args(&[
                    "-XX:+UseG1GC",
                    "-XX:MaxGCPauseMillis=100",
                    "-XX:+ParallelRefProcEnabled",
                    "-XX:+UnlockExperimentalVMOptions",
                    "-XX:+AlwaysPreTouch",
                    "-XX:G1NewSizePercent=20",
                    "-XX:G1MaxNewSizePercent=35",
                    "-XX:G1HeapRegionSize=16M",
                    "-XX:G1ReservePercent=20",
                    "-XX:InitiatingHeapOccupancyPercent=15",
                ]);
            }
            "g1gc-performance" => {
                cmd.args(&[
                    "-XX:+UseG1GC",
                    "-XX:MaxGCPauseMillis=50",
                    "-XX:+ParallelRefProcEnabled",
                    "-XX:+UnlockExperimentalVMOptions",
                    "-XX:+AlwaysPreTouch",
                    "-XX:G1NewSizePercent=30",
                    "-XX:G1MaxNewSizePercent=40",
                    "-XX:G1HeapRegionSize=32M",
                    "-XX:G1ReservePercent=15",
                    "-XX:InitiatingHeapOccupancyPercent=10",
                ]);
            }
            _ => {
                warn!("Unknown JVM flags profile: {}", self.config.minecraft.java.flags);
            }
        }
        
        // Add extra flags
        for flag in &self.config.minecraft.java.extra_flags {
            cmd.arg(flag);
        }
        
        // Add Guardian Agent JVM arguments
        cmd.args(&[
            "-javaagent:guardian-agent.jar",
            "-Dguardian.config.file=configs/server.yaml",
            "-Dguardian.rules.file=configs/rules.yaml",
        ]);
        
        // Add server jar and arguments
        cmd.args(&[
            "-jar",
            "server.jar",
            "nogui",
        ]);
        
        // Set working directory
        cmd.current_dir(&self.config.paths.world_dir);
        
        // Set up stdio
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        // Start the process
        let child = cmd.spawn()?;
        
        {
            let mut process_guard = self.minecraft_process.write().await;
            *process_guard = Some(child);
        }
        
        info!("Minecraft server started with PID: {:?}", self.get_minecraft_pid().await);
        Ok(())
    }
    
    /// Stop the Minecraft server process
    async fn stop_minecraft_server(&self) -> Result<()> {
        info!("Stopping Minecraft server...");
        
        let mut process_guard = self.minecraft_process.write().await;
        if let Some(mut child) = process_guard.take() {
            // Try graceful shutdown first
            if let Some(stdin) = child.stdin.take() {
                use tokio::io::AsyncWriteExt;
                let mut stdin = tokio::process::ChildStdin::from_std(stdin)?;
                stdin.write_all(b"stop\n").await?;
                stdin.flush().await?;
            }
            
            // Wait for graceful shutdown
            tokio::time::timeout(Duration::from_secs(30), async {
                child.wait()
            }).await??;
            
            info!("Minecraft server stopped gracefully");
        }
        
        Ok(())
    }
    
    /// Start the GPU worker process
    async fn start_gpu_worker(&self) -> Result<()> {
        info!("Starting GPU worker...");
        
        let mut cmd = Command::new("./gpu-worker");
        
        // Add environment variables
        cmd.env("RUST_LOG", "info");
        cmd.env("GPU_WORKER_IPC", &self.config.gpu.worker_ipc);
        
        // Set up stdio
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        // Start the process
        let child = cmd.spawn()?;
        
        {
            let mut process_guard = self.gpu_worker_process.write().await;
            *process_guard = Some(child);
        }
        
        info!("GPU worker started with PID: {:?}", self.get_gpu_worker_pid().await);
        Ok(())
    }
    
    /// Stop the GPU worker process
    async fn stop_gpu_worker(&self) -> Result<()> {
        info!("Stopping GPU worker...");
        
        let mut process_guard = self.gpu_worker_process.write().await;
        if let Some(mut child) = process_guard.take() {
            // Send SIGTERM
            child.kill()?;
            
            // Wait for process to exit
            child.wait()?;
            
            info!("GPU worker stopped");
        }
        
        Ok(())
    }
    
    /// Restart the Minecraft server
    pub async fn restart_minecraft_server(&self) -> Result<()> {
        info!("Restarting Minecraft server...");
        
        // Update restart count
        {
            let mut count_guard = self.restart_count.write().await;
            *count_guard += 1;
        }
        
        {
            let mut last_restart_guard = self.last_restart.write().await;
            *last_restart_guard = Some(Instant::now());
        }
        
        // Stop current server
        self.stop_minecraft_server().await?;
        
        // Wait before restarting
        sleep(Duration::from_secs(self.config.ha.restart_delay_seconds as u64)).await;
        
        // Start new server
        self.start_minecraft_server().await?;
        
        info!("Minecraft server restarted");
        Ok(())
    }
    
    /// Check if the Minecraft server is healthy
    pub async fn is_server_healthy(&self) -> bool {
        let process_guard = self.minecraft_process.read().await;
        if let Some(child) = process_guard.as_ref() {
            // For now, just check if the process exists
            // In a real implementation, we'd check if it's actually responding
            true
        } else {
            false // No process
        }
    }
    
    /// Check if the GPU worker is healthy
    pub async fn is_gpu_worker_healthy(&self) -> bool {
        let process_guard = self.gpu_worker_process.read().await;
        if let Some(child) = process_guard.as_ref() {
            // For now, just check if the process exists
            // In a real implementation, we'd check if it's actually responding
            true
        } else {
            false // No process
        }
    }
    
    /// Get Minecraft server PID
    pub async fn get_minecraft_pid(&self) -> Option<u32> {
        let process_guard = self.minecraft_process.read().await;
        process_guard.as_ref().map(|child| child.id())
    }
    
    /// Get GPU worker PID
    pub async fn get_gpu_worker_pid(&self) -> Option<u32> {
        let process_guard = self.gpu_worker_process.read().await;
        process_guard.as_ref().map(|child| child.id())
    }
    
    /// Get restart count
    pub async fn get_restart_count(&self) -> u32 {
        let count_guard = self.restart_count.read().await;
        *count_guard
    }
    
    /// Get last restart time
    pub async fn get_last_restart(&self) -> Option<Instant> {
        let last_restart_guard = self.last_restart.read().await;
        *last_restart_guard
    }
    
    /// Get process manager statistics
    pub async fn get_stats(&self) -> serde_json::Map<String, serde_json::Value> {
        let mut stats = serde_json::Map::new();
        
        stats.insert("minecraft_pid".to_string(), 
                    serde_json::Value::Number(self.get_minecraft_pid().await.unwrap_or(0).into()));
        stats.insert("gpu_worker_pid".to_string(), 
                    serde_json::Value::Number(self.get_gpu_worker_pid().await.unwrap_or(0).into()));
        stats.insert("restart_count".to_string(), 
                    serde_json::Value::Number(self.get_restart_count().await.into()));
        stats.insert("server_healthy".to_string(), 
                    serde_json::Value::Bool(self.is_server_healthy().await));
        stats.insert("gpu_worker_healthy".to_string(), 
                    serde_json::Value::Bool(self.is_gpu_worker_healthy().await));
        
        if let Some(last_restart) = self.get_last_restart().await {
            stats.insert("last_restart".to_string(), 
                        serde_json::Value::String(format!("{:?}", last_restart)));
        }
        
        stats
    }
}
