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
pub mod websocket;
pub mod api;
pub mod database;
pub mod minecraft;
pub mod rcon;
pub mod version_manager;
pub mod mod_classification;
pub mod compatibility_engine;
pub mod compatibility;
pub mod pregeneration;
pub mod hot_import;
pub mod lighting;
pub mod mod_management;
pub mod external_apis;
pub mod mod_manager;

pub mod auth;
pub mod tenant;
pub mod plugin;
pub mod webhook;
pub mod compliance;
pub mod community;
pub mod ai;
pub mod error;
pub mod config_validation;
pub mod health;
pub mod backup;
pub mod performance;
pub mod deployment;

use config::Config;
use process::ProcessManager;
use snapshot::SnapshotManager;
use metrics::MetricsCollector;
use auth::AuthManager;
use tenant::TenantManager;
use plugin::PluginManager;
use webhook::WebhookManager;
use compliance::ComplianceManager;
use community::CommunityManager;
use ai::AIManager;

use error::{GuardianError, ErrorHandler, CircuitBreaker};
use config_validation::{ConfigValidator, Environment};
use health::{HealthMonitor, HealthCheckRegistry};
use backup::{BackupManager, BackupConfig};
use performance::{PerformanceProfiler, PerformanceMetrics};
use deployment::{DeploymentManager, DeploymentConfig};

/// Main host daemon that orchestrates all Guardian services
pub struct HostDaemon {
    config: Config,
    process_manager: Arc<ProcessManager>,
    snapshot_manager: Arc<SnapshotManager>,
    metrics_collector: Arc<MetricsCollector>,

    auth_manager: Arc<AuthManager>,
    tenant_manager: Arc<TenantManager>,
    plugin_manager: Arc<PluginManager>,
    webhook_manager: Arc<WebhookManager>,
    compliance_manager: Arc<ComplianceManager>,
    community_manager: Arc<CommunityManager>,
    ai_manager: Arc<AIManager>,
    error_handler: Arc<ErrorHandler>,
    health_monitor: Arc<HealthMonitor>,
    backup_manager: Arc<BackupManager>,
    performance_profiler: Arc<PerformanceProfiler>,
    deployment_manager: Arc<DeploymentManager>,
    state: Arc<RwLock<DaemonState>>,
}

#[derive(Debug, Clone)]
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
        
        // Validate configuration
        let environment = Environment::Production; // TODO: Make this configurable
        let config_validator = ConfigValidator::new(environment);
        config_validator.validate_config(&config).await?;
        
        // Create error handler
        let error_handler = Arc::new(ErrorHandler::default());
        
        // Create health monitor
        let health_monitor = Arc::new(HealthMonitor::new(Duration::from_secs(30)));
        
        // Create backup manager
        let backup_config = BackupConfig {
            strategy: backup::BackupStrategy::Full,
            retention: backup::RetentionPolicy::default(),
            storage: backup::StorageConfig {
                local_path: std::path::PathBuf::from("./backups"),
                remote: None,
                compression_level: 6,
                encryption_enabled: false,
                encryption_key: None,
            },
            schedule: "0 2 * * *".to_string(), // Daily at 2 AM
            enabled: true,
            include_paths: vec![
                std::path::PathBuf::from("./world"),
                std::path::PathBuf::from("./config"),
                std::path::PathBuf::from("./mods"),
            ],
            exclude_paths: vec![
                std::path::PathBuf::from("./logs"),
                std::path::PathBuf::from("./temp"),
            ],
            max_size_bytes: 0,
            compression_threads: 4,
        };
        let backup_manager = Arc::new(BackupManager::new(backup_config));
        
        // Create performance profiler
        let performance_profiler = Arc::new(PerformanceProfiler::new(
            Duration::from_secs(30),
            Duration::from_secs(3600), // 1 hour retention
        ));
        
        // Create deployment manager
        let deployment_manager = Arc::new(DeploymentManager::new(
            Duration::from_secs(1800), // 30 minutes timeout
            Duration::from_secs(30),   // 30 seconds health check interval
        ));
        
        // Create process manager
        let process_manager = Arc::new(ProcessManager::new(config.clone()).await?);
        
        // Create snapshot manager
        let snapshot_manager = Arc::new(SnapshotManager::new(config.clone()).await?);
        
        // Create metrics collector
        let metrics_collector = Arc::new(MetricsCollector::new(config.clone()).await?);
        
        // Create auth manager
        let auth_manager = Arc::new(AuthManager::new(config.clone()));
        auth_manager.initialize().await?;
        
        // Create tenant manager
        let tenant_manager = Arc::new(TenantManager::new());
        
        // Create plugin manager
        let plugin_dir = std::path::PathBuf::from("./plugins");
        let plugin_manager = Arc::new(PluginManager::new(plugin_dir));
        plugin_manager.initialize().await?;
        
        // Create webhook manager
        let webhook_manager = Arc::new(WebhookManager::new());
        webhook_manager.initialize().await?;
        
        // Create compliance manager
        let compliance_manager = Arc::new(ComplianceManager::new());
        compliance_manager.initialize().await?;
        
        // Create community manager
        let community_manager = Arc::new(CommunityManager::new());
        community_manager.initialize().await?;
        
        // Create AI manager
        let ai_manager = Arc::new(AIManager::new());
        ai_manager.initialize().await?;
        

        
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

            auth_manager,
            tenant_manager,
            plugin_manager,
            webhook_manager,
            compliance_manager,
            community_manager,
            ai_manager,
            error_handler,
            health_monitor,
            backup_manager,
            performance_profiler,
            deployment_manager,
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
        // Start health monitor
        self.health_monitor.start().await?;
        
        // Register built-in health checks
        let health_registry = HealthCheckRegistry::new(self.health_monitor.clone());
        health_registry.register_builtin_checks().await?;
        
        // Start backup manager
        self.backup_manager.start().await?;
        
        // Start performance profiler
        self.performance_profiler.start().await?;
        
        // Start deployment manager
        self.deployment_manager.start().await?;
        
        // Start process manager
        self.process_manager.start().await?;
        
        // Start snapshot manager
        self.snapshot_manager.start().await?;
        
        // Start metrics collector
        self.metrics_collector.start().await?;
        

        
        // Start main daemon loop
        self.start_daemon_loop().await;
        
        info!("All Guardian Host Daemon services started");
        Ok(())
    }
    
    /// Stop all daemon services
    async fn stop_services(&mut self) -> Result<()> {
        info!("Stopping Guardian Host Daemon services...");
        

        
        // Stop metrics collector
        self.metrics_collector.stop().await?;
        
        // Stop snapshot manager
        self.snapshot_manager.stop().await?;
        
        // Stop process manager
        self.process_manager.stop().await?;
        
        // Stop backup manager
        self.backup_manager.stop().await;
        
        // Stop performance profiler
        self.performance_profiler.stop().await;
        
        // Stop deployment manager
        self.deployment_manager.stop().await;
        
        // Stop health monitor
        self.health_monitor.stop().await;
        
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
        // Performing health checks
        
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
        
        // Health checks completed
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
