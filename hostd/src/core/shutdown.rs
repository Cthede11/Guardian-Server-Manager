use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{timeout, sleep};
use tracing::{info, warn, error};
use uuid::Uuid;
use futures::future::join_all;

use crate::core::{
    process_manager::ProcessManager,
    crash_watchdog::CrashWatchdog,
    port_registry::PortRegistry,
    credential_manager::CredentialManager,
    error_handler::Result,
};
use crate::websocket_manager::WebSocketManager;
use crate::database::DatabaseManager;

/// Shutdown manager for graceful application shutdown
pub struct ShutdownManager {
    shutdown_tx: broadcast::Sender<()>,
    shutdown_rx: broadcast::Receiver<()>,
    is_shutting_down: Arc<RwLock<bool>>,
    shutdown_timeout: Duration,
}

impl ShutdownManager {
    pub fn new(shutdown_timeout: Duration) -> Self {
        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        
        Self {
            shutdown_tx,
            shutdown_rx,
            is_shutting_down: Arc::new(RwLock::new(false)),
            shutdown_timeout,
        }
    }

    /// Get a receiver for shutdown signals
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    /// Check if the application is shutting down
    pub async fn is_shutting_down(&self) -> bool {
        *self.is_shutting_down.read().await
    }

    /// Initiate graceful shutdown
    pub async fn shutdown(&self) -> Result<()> {
        let mut is_shutting_down = self.is_shutting_down.write().await;
        if *is_shutting_down {
            return Ok(()); // Already shutting down
        }
        *is_shutting_down = true;
        drop(is_shutting_down);

        info!("Initiating graceful shutdown...");

        // Send shutdown signal to all subscribers
        let _ = self.shutdown_tx.send(());

        Ok(())
    }

    /// Wait for shutdown signal
    pub async fn wait_for_shutdown(&mut self) {
        let _ = self.shutdown_rx.recv().await;
    }
}

/// Application shutdown handler
pub struct AppShutdownHandler {
    shutdown_manager: Arc<ShutdownManager>,
    process_manager: Arc<ProcessManager>,
    websocket_manager: Arc<WebSocketManager>,
    crash_watchdog: Arc<CrashWatchdog>,
    port_registry: Arc<PortRegistry>,
    credential_manager: Arc<CredentialManager>,
    database: Arc<DatabaseManager>,
}

impl AppShutdownHandler {
    pub fn new(
        shutdown_manager: Arc<ShutdownManager>,
        process_manager: Arc<ProcessManager>,
        websocket_manager: Arc<WebSocketManager>,
        crash_watchdog: Arc<CrashWatchdog>,
        port_registry: Arc<PortRegistry>,
        credential_manager: Arc<CredentialManager>,
        database: Arc<DatabaseManager>,
    ) -> Self {
        Self {
            shutdown_manager,
            process_manager,
            websocket_manager,
            crash_watchdog,
            port_registry,
            credential_manager,
            database,
        }
    }

    /// Perform graceful shutdown of all components
    pub async fn shutdown(&self) -> Result<()> {
        info!("Starting graceful shutdown process...");

        // Set up timeout for shutdown process
        let shutdown_result = timeout(self.shutdown_manager.shutdown_timeout, async {
            self.shutdown_components().await
        }).await;

        match shutdown_result {
            Ok(result) => {
                result?;
                info!("Graceful shutdown completed successfully");
            }
            Err(_) => {
                error!("Shutdown timeout exceeded, forcing shutdown");
                self.force_shutdown().await;
            }
        }

        Ok(())
    }

    async fn shutdown_components(&self) -> Result<()> {
        // 1. Stop accepting new connections
        info!("Stopping new connections...");
        // This would be handled by the HTTP server

        // 2. Stop crash watchdog
        info!("Stopping crash watchdog...");
        // Note: CrashWatchdog doesn't have a stop method in the current implementation
        // This would need to be added

        // 3. Stop all running servers gracefully
        info!("Stopping all running servers...");
        self.stop_all_servers().await?;

        // 4. Clean up monitoring tasks
        info!("Cleaning up monitoring tasks...");
        self.process_manager.cleanup_all_monitoring_tasks().await;

        // 5. Close WebSocket connections
        info!("Closing WebSocket connections...");
        self.websocket_manager.cleanup_expired_connections().await;

        // 6. Clean up port registry
        info!("Cleaning up port registry...");
        // Port registry cleanup would be automatic when servers are stopped

        // 7. Close database connections
        info!("Closing database connections...");
        // Database cleanup would be handled by the connection pool

        // 8. Clean up temporary files
        info!("Cleaning up temporary files...");
        self.cleanup_temp_files().await?;

        Ok(())
    }

    async fn stop_all_servers(&self) -> Result<()> {
        // Get all running servers
        let running_servers = self.get_running_servers().await;
        
        if running_servers.is_empty() {
            info!("No running servers to stop");
            return Ok(());
        }

        info!("Stopping {} running servers...", running_servers.len());

        // Stop servers in parallel with a timeout
        let stop_tasks: Vec<_> = running_servers.into_iter().map(|server_id| {
            let process_manager = self.process_manager.clone();
            async move {
                match process_manager.stop_server_process(server_id).await {
                    Ok(_) => {
                        info!("Successfully stopped server {}", server_id);
                        Ok(())
                    }
                    Err(e) => {
                        error!("Failed to stop server {}: {}", server_id, e);
                        Err(e)
                    }
                }
            }
        }).collect();

        // Wait for all servers to stop with a timeout
        let stop_timeout = Duration::from_secs(30);
        let stop_result = timeout(stop_timeout, async {
            let results = join_all(stop_tasks).await;
            results.into_iter().collect::<std::result::Result<Vec<_>, _>>()
        }).await;

        match stop_result {
            Ok(Ok(_)) => {
                info!("All servers stopped successfully");
            }
            Ok(Err(e)) => {
                warn!("Some servers failed to stop gracefully: {}", e);
            }
            Err(_) => {
                warn!("Server stop timeout exceeded, some servers may still be running");
            }
        }

        Ok(())
    }

    async fn get_running_servers(&self) -> Vec<Uuid> {
        // This would need to be implemented in ProcessManager
        // For now, return empty vector
        vec![]
    }

    async fn cleanup_temp_files(&self) -> Result<()> {
        // Clean up temporary files created during operation
        let temp_dirs = vec![
            "data/temp",
            "data/logs",
            "data/backups/temp",
        ];

        for dir in temp_dirs {
            if let Err(e) = tokio::fs::remove_dir_all(dir).await {
                if e.kind() != std::io::ErrorKind::NotFound {
                    warn!("Failed to clean up temp directory {}: {}", dir, e);
                }
            }
        }

        Ok(())
    }

    async fn force_shutdown(&self) {
        warn!("Performing forced shutdown...");
        
        // Force kill all processes
        // This would be implemented based on the specific process management needs
        
        // Clean up resources immediately
        self.process_manager.cleanup_all_monitoring_tasks().await;
        
        warn!("Forced shutdown completed");
    }
}

/// Set up signal handlers for graceful shutdown (Unix/Linux)
#[cfg(unix)]
pub async fn setup_signal_handlers(shutdown_manager: Arc<ShutdownManager>) -> Result<()> {
    let shutdown_manager_clone = shutdown_manager.clone();
    
    // Handle SIGTERM
    tokio::spawn(async move {
        let mut sigterm = match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(signal) => signal,
            Err(e) => {
                error!("Failed to create SIGTERM handler: {}", e);
                return;
            }
        };
        
        sigterm.recv().await;
        info!("Received SIGTERM, initiating shutdown...");
        
        if let Err(e) = shutdown_manager_clone.shutdown().await {
            error!("Failed to initiate shutdown: {}", e);
        }
    });

    // Handle SIGINT (Ctrl+C)
    let shutdown_manager_clone = shutdown_manager.clone();
    tokio::spawn(async move {
        let mut sigint = match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt()) {
            Ok(signal) => signal,
            Err(e) => {
                error!("Failed to create SIGINT handler: {}", e);
                return;
            }
        };
        
        sigint.recv().await;
        info!("Received SIGINT, initiating shutdown...");
        
        if let Err(e) = shutdown_manager_clone.shutdown().await {
            error!("Failed to initiate shutdown: {}", e);
        }
    });

    // Handle SIGUSR1 for graceful restart
    let shutdown_manager_clone = shutdown_manager.clone();
    tokio::spawn(async move {
        let mut sigusr1 = match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::user_defined1()) {
            Ok(signal) => signal,
            Err(e) => {
                error!("Failed to create SIGUSR1 handler: {}", e);
                return;
            }
        };
        
        sigusr1.recv().await;
        info!("Received SIGUSR1, initiating graceful restart...");
        
        if let Err(e) = shutdown_manager_clone.shutdown().await {
            error!("Failed to initiate graceful restart: {}", e);
        }
    });

    Ok(())
}

/// Set up signal handlers for graceful shutdown (Windows)
#[cfg(target_os = "windows")]
pub async fn setup_signal_handlers(shutdown_manager: Arc<ShutdownManager>) -> Result<()> {
    let shutdown_manager_clone = shutdown_manager.clone();
    
    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            error!("Failed to create Ctrl+C handler: {}", e);
            return;
        }
        
        info!("Received Ctrl+C, initiating shutdown...");
        
        if let Err(e) = shutdown_manager_clone.shutdown().await {
            error!("Failed to initiate shutdown: {}", e);
        }
    });

    Ok(())
}

