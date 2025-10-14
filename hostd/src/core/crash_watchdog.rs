use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;
use uuid::Uuid;
use serde::Serialize;
use tracing::{info, warn, error};

use crate::core::{
    error_handler::{AppError, Result},
    process_manager::{ProcessManager, ServerState},
    monitoring::MonitoringManager,
    retry_backoff::{RetryManager, RetryConfig, with_crash_recovery_retry},
};
use crate::database::DatabaseManager;

/// Crash detection configuration
#[derive(Debug, Clone)]
pub struct WatchdogConfig {
    pub hang_threshold: Duration,
    pub check_interval: Duration,
    pub max_restart_attempts: u32,
    pub restart_cooldown: Duration,
}

impl Default for WatchdogConfig {
    fn default() -> Self {
        Self {
            hang_threshold: Duration::from_secs(5),
            check_interval: Duration::from_secs(1),
            max_restart_attempts: 3,
            restart_cooldown: Duration::from_secs(30),
        }
    }
}

/// Server health status
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ServerHealth {
    Healthy,
    Hanging,
    Crashed,
    Restarting,
}

/// Server watchdog state
#[derive(Debug)]
struct ServerWatchdogState {
    server_id: Uuid,
    last_heartbeat: Instant,
    health: ServerHealth,
    restart_attempts: u32,
    last_restart: Option<Instant>,
    hang_start: Option<Instant>,
}

/// Crash watchdog system
pub struct CrashWatchdog {
    config: WatchdogConfig,
    process_manager: Arc<ProcessManager>,
    monitoring: Arc<MonitoringManager>,
    database: Arc<DatabaseManager>,
    server_states: Arc<RwLock<std::collections::HashMap<Uuid, ServerWatchdogState>>>,
}

impl CrashWatchdog {
    pub fn new(
        config: WatchdogConfig,
        process_manager: Arc<ProcessManager>,
        monitoring: Arc<MonitoringManager>,
        database: Arc<DatabaseManager>,
    ) -> Self {
        Self {
            config,
            process_manager,
            monitoring,
            database,
            server_states: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Start the watchdog monitoring loop
    pub async fn start(&self) -> Result<()> {
        info!("Starting crash watchdog with config: {:?}", self.config);
        
        let mut interval = interval(self.config.check_interval);
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.check_all_servers().await {
                error!("Error in watchdog check: {}", e);
            }
        }
    }

    /// Register a server for monitoring
    pub async fn register_server(&self, server_id: Uuid) -> Result<()> {
        let mut states = self.server_states.write().await;
        states.insert(server_id, ServerWatchdogState {
            server_id,
            last_heartbeat: Instant::now(),
            health: ServerHealth::Healthy,
            restart_attempts: 0,
            last_restart: None,
            hang_start: None,
        });
        
        info!("Registered server {} for crash monitoring", server_id);
        Ok(())
    }

    /// Unregister a server from monitoring
    pub async fn unregister_server(&self, server_id: Uuid) -> Result<()> {
        let mut states = self.server_states.write().await;
        states.remove(&server_id);
        
        info!("Unregistered server {} from crash monitoring", server_id);
        Ok(())
    }

    /// Update server heartbeat
    pub async fn update_heartbeat(&self, server_id: Uuid) -> Result<()> {
        let mut states = self.server_states.write().await;
        if let Some(state) = states.get_mut(&server_id) {
            state.last_heartbeat = Instant::now();
            state.hang_start = None;
            
            if state.health == ServerHealth::Hanging {
                state.health = ServerHealth::Healthy;
                info!("Server {} recovered from hang", server_id);
            }
        }
        Ok(())
    }

    /// Check all registered servers for crashes
    async fn check_all_servers(&self) -> Result<()> {
        let mut states = self.server_states.write().await;
        let mut to_remove = Vec::new();
        
        for (server_id, state) in states.iter_mut() {
            // Check server state first
            let server_state = self.process_manager.get_server_state(*server_id).await;
            
            // Check if server process has crashed
            if !self.process_manager.is_server_running(*server_id).await {
                if state.health != ServerHealth::Crashed {
                    state.health = ServerHealth::Crashed;
                    warn!("Server {} has crashed (process not running)", server_id);
                    
                    // Log crash immediately
                    self.log_crash_event(*server_id, "Process terminated unexpectedly").await?;
                    
                    // Try to restart if within limits
                    if self.should_attempt_restart(state) {
                        if let Err(e) = self.attempt_restart(*server_id).await {
                            error!("Failed to restart crashed server {}: {}", server_id, e);
                            self.log_crash_event(*server_id, &format!("Restart failed: {}", e)).await?;
                        }
                    }
                }
                continue;
            }
            
            // Check if server state indicates a crash
            if matches!(server_state, ServerState::Crashed) {
                if state.health != ServerHealth::Crashed {
                    state.health = ServerHealth::Crashed;
                    warn!("Server {} state indicates crash", server_id);
                    
                    // Log crash immediately
                    self.log_crash_event(*server_id, "Server state indicates crash").await?;
                    
                    // Try to restart if within limits
                    if self.should_attempt_restart(state) {
                        if let Err(e) = self.attempt_restart(*server_id).await {
                            error!("Failed to restart crashed server {}: {}", server_id, e);
                            self.log_crash_event(*server_id, &format!("Restart failed: {}", e)).await?;
                        }
                    }
                }
                continue;
            }

            // Check for hangs
            let time_since_heartbeat = state.last_heartbeat.elapsed();
            if time_since_heartbeat > self.config.hang_threshold {
                if state.health == ServerHealth::Healthy {
                    state.health = ServerHealth::Hanging;
                    state.hang_start = Some(Instant::now());
                    warn!("Server {} appears to be hanging (no heartbeat for {:?})", 
                          server_id, time_since_heartbeat);
                } else if state.health == ServerHealth::Hanging {
                    // Check if we should restart due to prolonged hang
                    if let Some(hang_start) = state.hang_start {
                        let hang_duration = hang_start.elapsed();
                        if hang_duration > Duration::from_secs(30) { // 30 seconds of hanging
                            warn!("Server {} has been hanging for {:?}, attempting restart", 
                                  server_id, hang_duration);
                            
                            if self.should_attempt_restart(state) {
                                if let Err(e) = self.attempt_restart(*server_id).await {
                                    error!("Failed to restart hanging server {}: {}", server_id, e);
                                }
                            }
                        }
                    }
                }
            } else if state.health == ServerHealth::Hanging {
                // Server recovered
                state.health = ServerHealth::Healthy;
                state.hang_start = None;
                info!("Server {} recovered from hang", server_id);
            }

            // Check if server should be removed (too many failed restarts)
            if state.restart_attempts >= self.config.max_restart_attempts {
                error!("Server {} has exceeded maximum restart attempts, removing from monitoring", server_id);
                to_remove.push(*server_id);
            }
        }

        // Remove servers that exceeded restart limits
        for server_id in to_remove {
            states.remove(&server_id);
        }

        Ok(())
    }

    /// Check if we should attempt a restart with exponential backoff
    fn should_attempt_restart(&self, state: &ServerWatchdogState) -> bool {
        if state.restart_attempts >= self.config.max_restart_attempts {
            return false;
        }

        if let Some(last_restart) = state.last_restart {
            // Calculate exponential backoff: base_cooldown * 2^attempts
            let backoff_multiplier = 2_u32.pow(state.restart_attempts.min(6)); // Cap at 2^6 = 64
            let backoff_duration = self.config.restart_cooldown * backoff_multiplier;
            
            if last_restart.elapsed() < backoff_duration {
                return false;
            }
        }

        true
    }

    /// Log a crash event to the database
    async fn log_crash_event(&self, server_id: Uuid, reason: &str) -> Result<()> {
        let timestamp = chrono::Utc::now();
        
        // Log to database
        if let Err(e) = self.database.log_server_message(
            &server_id.to_string(),
            "ERROR",
            &format!("Server crashed: {}", reason),
            Some("CrashWatchdog"),
        ).await {
            error!("Failed to log crash event for server {}: {}", server_id, e);
        }
        
        // Log to monitoring system
        // TODO: Implement record_event method in MonitoringManager
        // if let Err(e) = self.monitoring.record_event(
        //     server_id,
        //     "server_crash",
        //     &format!("Server crashed: {}", reason),
        //     Some(timestamp),
        // ).await {
        //     error!("Failed to record crash event in monitoring: {}", e);
        // }
        
        Ok(())
    }

    /// Attempt to restart a server
    async fn attempt_restart(&self, server_id: Uuid) -> Result<()> {
        let mut states = self.server_states.write().await;
        if let Some(state) = states.get_mut(&server_id) {
            state.restart_attempts += 1;
            state.last_restart = Some(Instant::now());
            state.health = ServerHealth::Restarting;
        }
        drop(states); // Release the lock before the async operation

        // Use retry backoff for the restart operation
        let restart_result = with_crash_recovery_retry(|| {
            let process_manager = self.process_manager.clone();
            let database = self.database.clone();
            let server_id = server_id;
            
            async move {
                // Stop the server gracefully first
                if let Err(e) = process_manager.stop_server_process(server_id).await {
                    warn!("Error stopping server {} during restart: {}", server_id, e);
                }

                // Wait a moment for cleanup
                tokio::time::sleep(Duration::from_secs(2)).await;

                // Get server config from database
                let server_config = database.get_server(&server_id.to_string()).await
                    .map_err(|e| AppError::DatabaseError {
                        message: format!("Failed to get server config: {}", e),
                        operation: "get_server".to_string(),
                        table: Some("servers".to_string()),
                    })?
                    .ok_or_else(|| AppError::DatabaseError {
                        message: format!("Server {} not found", server_id),
                        operation: "get_server".to_string(),
                        table: Some("servers".to_string()),
                    })?;
                
                // Start the server
                process_manager.start_server_process(server_config).await?;
                
                Ok::<(), AppError>(())
            }
        }).await;

        match restart_result {
            Ok(_) => {
                info!("Successfully restarted server {}", server_id);
                
                // Reset restart attempts on successful restart
                let mut states = self.server_states.write().await;
                if let Some(state) = states.get_mut(&server_id) {
                    state.restart_attempts = 0;
                    state.health = ServerHealth::Healthy;
                    state.last_heartbeat = Instant::now();
                }
            }
            Err(e) => {
                error!("Failed to restart server {} after retries: {}", server_id, e);
                
                // Update state to reflect restart failure
                let mut states = self.server_states.write().await;
                if let Some(state) = states.get_mut(&server_id) {
                    state.health = ServerHealth::Crashed;
                }
            }
        }

        Ok(())
    }

    /// Get server health status
    pub async fn get_server_health(&self, server_id: Uuid) -> Option<ServerHealth> {
        let states = self.server_states.read().await;
        states.get(&server_id).map(|state| state.health.clone())
    }

    /// Get all server health statuses
    pub async fn get_all_server_health(&self) -> std::collections::HashMap<Uuid, ServerHealth> {
        let states = self.server_states.read().await;
        states.iter()
            .map(|(id, state)| (*id, state.health.clone()))
            .collect()
    }

    /// Get crash statistics for all monitored servers
    pub async fn get_crash_stats(&self) -> std::collections::HashMap<Uuid, serde_json::Value> {
        let states = self.server_states.read().await;
        let mut stats = std::collections::HashMap::new();
        
        for (server_id, state) in states.iter() {
            let server_stats = serde_json::json!({
                "server_id": server_id,
                "health": state.health,
                "restart_attempts": state.restart_attempts,
                "last_heartbeat": state.last_heartbeat.elapsed().as_secs(),
                "last_restart": state.last_restart.map(|t| t.elapsed().as_secs()),
                "hang_start": state.hang_start.map(|t| t.elapsed().as_secs()),
                "is_healthy": state.health == ServerHealth::Healthy,
            });
            
            stats.insert(*server_id, server_stats);
        }
        
        stats
    }

    /// Get crash statistics for a specific server
    pub async fn get_server_crash_stats(&self, server_id: Uuid) -> Option<serde_json::Value> {
        let states = self.server_states.read().await;
        
        if let Some(state) = states.get(&server_id) {
            Some(serde_json::json!({
                "server_id": server_id,
                "health": state.health,
                "restart_attempts": state.restart_attempts,
                "last_heartbeat": state.last_heartbeat.elapsed().as_secs(),
                "last_restart": state.last_restart.map(|t| t.elapsed().as_secs()),
                "hang_start": state.hang_start.map(|t| t.elapsed().as_secs()),
                "is_healthy": state.health == ServerHealth::Healthy,
            }))
        } else {
            None
        }
    }

    /// Force restart a server (bypasses restart limits)
    pub async fn force_restart(&self, server_id: Uuid) -> Result<()> {
        info!("Force restarting server {}", server_id);
        
        // Reset restart attempts
        {
            let mut states = self.server_states.write().await;
            if let Some(state) = states.get_mut(&server_id) {
                state.restart_attempts = 0;
                state.health = ServerHealth::Restarting;
            }
        }

        self.attempt_restart(server_id).await
    }
}
