use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::{interval, sleep};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{info, warn, error, debug};
use cron::Schedule;
use std::str::FromStr;

use crate::core::{
    error_handler::{AppError, Result},
    server_manager::ServerManager,
};

/// Scheduled task types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    Backup,
    Restart,
    Maintenance,
    Custom(String),
}

/// Task status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Scheduled,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Scheduled task definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub task_type: TaskType,
    pub server_id: Option<Uuid>, // None for global tasks
    pub cron_expression: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub status: TaskStatus,
    pub config: serde_json::Value, // Task-specific configuration
}

/// Task execution result
#[derive(Debug, Clone)]
pub struct TaskResult {
    pub task_id: Uuid,
    pub status: TaskStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration: Option<Duration>,
    pub message: String,
    pub error: Option<String>,
}

/// Scheduler configuration
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    pub check_interval: Duration,
    pub max_concurrent_tasks: usize,
    pub task_timeout: Duration,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(60), // Check every minute
            max_concurrent_tasks: 5,
            task_timeout: Duration::from_secs(300), // 5 minutes timeout
        }
    }
}

/// Task scheduler
pub struct TaskScheduler {
    config: SchedulerConfig,
    tasks: Arc<RwLock<HashMap<Uuid, ScheduledTask>>>,
    running_tasks: Arc<RwLock<HashMap<Uuid, tokio::task::JoinHandle<()>>>>,
    server_manager: Arc<ServerManager>,
}

impl TaskScheduler {
    pub fn new(
        config: SchedulerConfig,
        server_manager: Arc<ServerManager>,
    ) -> Self {
        Self {
            config,
            tasks: Arc::new(RwLock::new(HashMap::new())),
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
            server_manager,
        }
    }

    /// Start the scheduler
    pub async fn start(&self) -> Result<()> {
        info!("Starting task scheduler with config: {:?}", self.config);
        
        let mut interval = interval(self.config.check_interval);
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.check_and_execute_tasks().await {
                error!("Error in scheduler: {}", e);
            }
        }
    }

    /// Add a scheduled task
    pub async fn add_task(&self, task: ScheduledTask) -> Result<()> {
        // Validate cron expression
        // Simple validation - just check if it's not empty
        if task.cron_expression.is_empty() {
            return Err(AppError::ConfigurationError {
                message: "Empty cron expression".to_string(),
                config_key: "cron_expression".to_string(),
                expected_type: "String".to_string(),
            });
        }

        // Calculate next run time
        let next_run = self.calculate_next_run(&task.cron_expression)?;
        
        let task_name = task.name.clone();
        let mut tasks = self.tasks.write().await;
        tasks.insert(task.id, task);
        
        info!("Added scheduled task: {}", task_name);
        Ok(())
    }

    /// Remove a scheduled task
    pub async fn remove_task(&self, task_id: Uuid) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        tasks.remove(&task_id);
        
        // Cancel if running
        let mut running = self.running_tasks.write().await;
        if let Some(handle) = running.remove(&task_id) {
            handle.abort();
        }
        
        info!("Removed scheduled task: {}", task_id);
        Ok(())
    }

    /// Update a scheduled task
    pub async fn update_task(&self, task: ScheduledTask) -> Result<()> {
        // Validate cron expression
        // Simple validation - just check if it's not empty
        if task.cron_expression.is_empty() {
            return Err(AppError::ConfigurationError {
                message: "Empty cron expression".to_string(),
                config_key: "cron_expression".to_string(),
                expected_type: "String".to_string(),
            });
        }

        let task_id = task.id;
        let mut tasks = self.tasks.write().await;
        tasks.insert(task_id, task);
        
        info!("Updated scheduled task: {}", task_id);
        Ok(())
    }

    /// Get all tasks
    pub async fn get_tasks(&self) -> Vec<ScheduledTask> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    /// Get tasks for a specific server
    pub async fn get_server_tasks(&self, server_id: Uuid) -> Vec<ScheduledTask> {
        let tasks = self.tasks.read().await;
        tasks.values()
            .filter(|task| task.server_id == Some(server_id))
            .cloned()
            .collect()
    }

    /// Check and execute due tasks
    async fn check_and_execute_tasks(&self) -> Result<()> {
        let now = Utc::now();
        let mut tasks_to_run = Vec::new();
        
        {
            let tasks = self.tasks.read().await;
            for task in tasks.values() {
                if !task.enabled || task.status == TaskStatus::Running {
                    continue;
                }

                if let Some(next_run) = task.next_run {
                    if now >= next_run {
                        tasks_to_run.push(task.clone());
                    }
                }
            }
        }

        // Check if we can run more tasks
        let running_count = {
            let running = self.running_tasks.read().await;
            running.len()
        };

        if running_count >= self.config.max_concurrent_tasks {
            debug!("Maximum concurrent tasks reached, skipping execution");
            return Ok(());
        }

        // Execute tasks
        for task in tasks_to_run {
            if running_count >= self.config.max_concurrent_tasks {
                break;
            }

            self.execute_task(task).await?;
        }

        Ok(())
    }

    /// Execute a task
    async fn execute_task(&self, mut task: ScheduledTask) -> Result<()> {
        info!("Executing scheduled task: {} ({})", task.name, task.id);
        
        // Update task status
        {
            let mut tasks = self.tasks.write().await;
            if let Some(t) = tasks.get_mut(&task.id) {
                t.status = TaskStatus::Running;
                t.last_run = Some(Utc::now());
            }
        }

        // Create execution handle
        let task_id = task.id;
        let task_type = task.task_type.clone();
        let server_id = task.server_id;
        let config = task.config.clone();
        let server_manager = self.server_manager.clone();
        let tasks = self.tasks.clone();

        let handle = tokio::spawn(async move {
            let result = match task_type {
                TaskType::Backup => {
                    Self::execute_backup_task(server_id, config).await
                }
                TaskType::Restart => {
                    Self::execute_restart_task(server_id, server_manager).await
                }
                TaskType::Maintenance => {
                    Self::execute_maintenance_task(server_id, config, server_manager).await
                }
                TaskType::Custom(name) => {
                    Self::execute_custom_task(name, server_id, config).await
                }
            };

            // Update task status
            let mut tasks = tasks.write().await;
            if let Some(t) = tasks.get_mut(&task_id) {
                match result {
                    Ok(_) => {
                        t.status = TaskStatus::Completed;
                        info!("Task {} completed successfully", task_id);
                    }
                    Err(e) => {
                        t.status = TaskStatus::Failed;
                        error!("Task {} failed: {}", task_id, e);
                    }
                }
                
                // Calculate next run time
                if let Ok(next_run) = Self::calculate_next_run_static(&t.cron_expression) {
                    t.next_run = Some(next_run);
                }
            }
        });

        // Store running task handle
        {
            let mut running = self.running_tasks.write().await;
            running.insert(task_id, handle);
        }

        Ok(())
    }

    /// Execute backup task
    async fn execute_backup_task(
        server_id: Option<Uuid>,
        config: serde_json::Value,
    ) -> Result<()> {
        let server_id = server_id.ok_or_else(|| AppError::ServerError {
            message: "Backup task requires server_id".to_string(),
            server_id: "unknown".to_string(),
            operation: "backup".to_string(),
        })?;

        let backup_name = config.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Scheduled Backup");

        let description = config.get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Create backup using BackupManager
        let backup_manager = crate::backup_manager::BackupManager::new(
            std::path::PathBuf::from("data/backups"),
            std::path::PathBuf::from("data/servers")
        );

        let request = crate::backup_manager::CreateBackupRequest {
            name: backup_name.to_string(),
            description,
            backup_type: crate::backup_manager::BackupType::Scheduled,
            compression: crate::backup_manager::CompressionType::Zip,
            includes: crate::backup_manager::BackupIncludes {
                world: true,
                mods: true,
                config: true,
                logs: false,
                server_properties: true,
                whitelist: true,
                ops: true,
                banned_players: true,
                banned_ips: true,
            },
            metadata: Some(serde_json::Value::Object(serde_json::Map::new())),
        };

        backup_manager.create_backup(&server_id.to_string(), request).await
            .map_err(|e| AppError::ServerError {
                message: format!("Failed to create scheduled backup: {}", e),
                server_id: server_id.to_string(),
                operation: "backup".to_string(),
            })?;

        info!("Scheduled backup completed for server {}", server_id);
        Ok(())
    }

    /// Execute restart task
    async fn execute_restart_task(
        server_id: Option<Uuid>,
        server_manager: Arc<ServerManager>,
    ) -> Result<()> {
        let server_id = server_id.ok_or_else(|| AppError::ServerError {
            message: "Restart task requires server_id".to_string(),
            server_id: "unknown".to_string(),
            operation: "restart".to_string(),
        })?;

        server_manager.restart_server(server_id).await?;
        info!("Scheduled restart completed for server {}", server_id);
        Ok(())
    }

    /// Execute maintenance task
    async fn execute_maintenance_task(
        server_id: Option<Uuid>,
        config: serde_json::Value,
        server_manager: Arc<ServerManager>,
    ) -> Result<()> {
        // Maintenance tasks can include cleanup, optimization, etc.
        let maintenance_type = config.get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        match maintenance_type {
            "cleanup_logs" => {
                // Clean up old log files
                info!("Performing log cleanup maintenance");
                // Implementation would go here
            }
            "optimize_world" => {
                // Optimize world files
                if let Some(server_id) = server_id {
                    info!("Performing world optimization for server {}", server_id);
                    // Implementation would go here
                }
            }
            _ => {
                info!("Performing general maintenance");
                // General maintenance tasks
            }
        }

        Ok(())
    }

    /// Execute custom task
    async fn execute_custom_task(
        name: String,
        server_id: Option<Uuid>,
        config: serde_json::Value,
    ) -> Result<()> {
        info!("Executing custom task: {} for server: {:?}", name, server_id);
        
        // Custom task execution would be implemented based on requirements
        // This could include calling external scripts, APIs, etc.
        
        Ok(())
    }

    /// Calculate next run time for a cron expression
    fn calculate_next_run(&self, cron_expr: &str) -> Result<DateTime<Utc>> {
        Self::calculate_next_run_static(cron_expr)
    }

    fn calculate_next_run_static(cron_expr: &str) -> Result<DateTime<Utc>> {
        if cron_expr.is_empty() {
            return Err(AppError::ConfigurationError {
                message: "Empty cron expression".to_string(),
                config_key: "cron_expression".to_string(),
                expected_type: "String".to_string(),
            });
        }
        
        // Parse cron expression and calculate next run time
        let schedule = Schedule::from_str(cron_expr).map_err(|e| AppError::ConfigurationError {
            message: format!("Invalid cron expression: {}", e),
            config_key: "cron_expression".to_string(),
            expected_type: "Valid cron expression".to_string(),
        })?;
        
        // Get the next occurrence after now
        let now = Utc::now();
        schedule.upcoming(Utc).next()
            .ok_or_else(|| AppError::ConfigurationError {
                message: "No valid next run time found".to_string(),
                config_key: "cron_expression".to_string(),
                expected_type: "Valid cron expression".to_string(),
            })
    }

    /// Get task execution history (would need to be stored in database)
    pub async fn get_task_history(&self, task_id: Uuid) -> Vec<TaskResult> {
        // This would typically query a database for task execution history
        // For now, return empty vector
        Vec::new()
    }
}
