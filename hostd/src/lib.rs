use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};
use anyhow::Result;
use uuid::Uuid;

pub mod config;
pub mod process;
pub mod snapshot;
pub mod metrics;
pub mod web;

use config::Config;
use process::ProcessManager;
use snapshot::SnapshotManager;
use metrics::MetricsCollector;
use web::WebServer;

/// Main host daemon that orchestrates all Guardian services
pub struct HostDaemon {
    config: Config,
    process_manager: Arc<ProcessManager>,
    snapshot_manager: Arc<SnapshotManager>,
    metrics_collector: Arc<MetricsCollector>,
    web_server: Arc<WebServer>,
    state: Arc<RwLock<DaemonState>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonState {
    pub start_time: Instant,
    pub uptime: Duration,
    pub server_status: ServerStatus,
    pub gpu_worker_status: WorkerStatus,
    pub last_health_check: Option<Instant>,
    pub restart_count: u32,
    pub last_restart: Option<Instant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Crashed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkerStatus {
    Starting,
    Running,
    Stopped,
    Crashed,
    Unknown,
}

impl HostDaemon {
    /// Create a new host daemon
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing Guardian Host Daemon...");
        
        // Create process manager
        let process_manager = Arc::new(ProcessManager::new(config.clone()).await?);
        
        // Create snapshot manager
        let snapshot_manager = Arc::new(SnapshotManager::new(config.clone()).await?);
        
        // Create metrics collector
        let metrics_collector = Arc::new(MetricsCollector::new(config.clone()).await?);
        
        // Create web server
        let web_server = Arc::new(WebServer::new(config.clone(), metrics_collector.clone()).await?);
        
        // Create initial state
        let state = Arc::new(RwLock::new(DaemonState {
            start_time: Instant::now(),
            uptime: Duration::ZERO,
            server_status: ServerStatus::Starting,
            gpu_worker_status: WorkerStatus::Starting,
            last_health_check: None,
            restart_count: 0,
            last_restart: None,
        }));
        
        info!("Guardian Host Daemon initialized successfully");
        
        Ok(Self {
            config,
            process_manager,
            snapshot_manager,
            metrics_collector,
            web_server,
            state,
        })
    }
    
    /// Run the daemon in foreground mode
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting Guardian Host Daemon services...");
        
        // Start all services
        self.start_services().await?;
        
        // Wait for shutdown signal
        tokio::signal::ctrl_c().await?;
        
        info!("Shutdown signal received, stopping services...");
        self.stop_services().await?;
        
        Ok(())
    }
    
    /// Run the daemon in daemon mode
    pub async fn run_daemon(&mut self) -> Result<()> {
        info!("Starting Guardian Host Daemon in daemon mode...");
        
        // Start all services
        self.start_services().await?;
        
        // Keep running until shutdown
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            
            // Check if we should shutdown
            if self.should_shutdown().await {
                break;
            }
        }
        
        info!("Daemon shutdown requested, stopping services...");
        self.stop_services().await?;
        
        Ok(())
    }
    
    /// Start all daemon services
    async fn start_services(&mut self) -> Result<()> {
        // Start process manager
        self.process_manager.start().await?;
        
        // Start snapshot manager
        self.snapshot_manager.start().await?;
        
        // Start metrics collector
        self.metrics_collector.start().await?;
        
        // Start web server
        self.web_server.start().await?;
        
        // Start main daemon loop
        self.start_daemon_loop().await;
        
        info!("All Guardian Host Daemon services started");
        Ok(())
    }
    
    /// Stop all daemon services
    async fn stop_services(&mut self) -> Result<()> {
        info!("Stopping Guardian Host Daemon services...");
        
        // Stop web server
        self.web_server.stop().await?;
        
        // Stop metrics collector
        self.metrics_collector.stop().await?;
        
        // Stop snapshot manager
        self.snapshot_manager.stop().await?;
        
        // Stop process manager
        self.process_manager.stop().await?;
        
        info!("All Guardian Host Daemon services stopped");
        Ok(())
    }
    
    /// Start the main daemon loop
    async fn start_daemon_loop(&self) {
        let state = self.state.clone();
        let process_manager = self.process_manager.clone();
        let snapshot_manager = self.snapshot_manager.clone();
        let metrics_collector = self.metrics_collector.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                // Update uptime
                {
                    let mut state_guard = state.write().await;
                    state_guard.uptime = state_guard.start_time.elapsed();
                }
                
                // Perform health checks
                if let Err(e) = Self::perform_health_checks(
                    &state,
                    &process_manager,
                    &snapshot_manager,
                    &metrics_collector,
                ).await {
                    error!("Health check failed: {}", e);
                }
                
                // Update metrics
                if let Err(e) = metrics_collector.update_metrics(&state).await {
                    error!("Failed to update metrics: {}", e);
                }
            }
        });
    }
    
    /// Perform health checks on all services
    async fn perform_health_checks(
        state: &Arc<RwLock<DaemonState>>,
        process_manager: &Arc<ProcessManager>,
        snapshot_manager: &Arc<SnapshotManager>,
        metrics_collector: &Arc<MetricsCollector>,
    ) -> Result<()> {
        debug!("Performing health checks...");
        
        // Check server process
        let server_healthy = process_manager.is_server_healthy().await;
        
        // Check GPU worker
        let gpu_worker_healthy = process_manager.is_gpu_worker_healthy().await;
        
        // Update state
        {
            let mut state_guard = state.write().await;
            state_guard.last_health_check = Some(Instant::now());
            
            state_guard.server_status = if server_healthy {
                ServerStatus::Running
            } else {
                ServerStatus::Crashed
            };
            
            state_guard.gpu_worker_status = if gpu_worker_healthy {
                WorkerStatus::Running
            } else {
                WorkerStatus::Crashed
            };
        }
        
        // Take snapshots if needed
        if server_healthy {
            if let Err(e) = snapshot_manager.take_snapshot_if_needed().await {
                warn!("Failed to take snapshot: {}", e);
            }
        }
        
        debug!("Health checks completed");
        Ok(())
    }
    
    /// Check if the daemon should shutdown
    async fn should_shutdown(&self) -> bool {
        // Check for shutdown file
        let shutdown_file = PathBuf::from("/tmp/guardian-shutdown");
        if shutdown_file.exists() {
            info!("Shutdown file detected, initiating shutdown...");
            return true;
        }
        
        // Check for excessive restarts
        let state = self.state.read().await;
        if state.restart_count > 10 {
            error!("Too many restarts, shutting down daemon");
            return true;
        }
        
        false
    }
    
    /// Get current daemon state
    pub async fn get_state(&self) -> DaemonState {
        self.state.read().await.clone()
    }
    
    /// Get daemon statistics
    pub async fn get_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        
        let state = self.state.read().await;
        stats.insert("uptime".to_string(), serde_json::Value::String(format!("{:?}", state.uptime)));
        stats.insert("server_status".to_string(), serde_json::Value::String(format!("{:?}", state.server_status)));
        stats.insert("gpu_worker_status".to_string(), serde_json::Value::String(format!("{:?}", state.gpu_worker_status)));
        stats.insert("restart_count".to_string(), serde_json::Value::Number(state.restart_count.into()));
        
        // Add process manager stats
        let process_stats = self.process_manager.get_stats().await;
        stats.insert("process_stats".to_string(), serde_json::Value::Object(process_stats));
        
        // Add snapshot manager stats
        let snapshot_stats = self.snapshot_manager.get_stats().await;
        stats.insert("snapshot_stats".to_string(), serde_json::Value::Object(snapshot_stats));
        
        stats
    }
}
