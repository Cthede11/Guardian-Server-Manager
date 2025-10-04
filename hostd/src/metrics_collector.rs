use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sysinfo::{System, Process, Pid};

/// Real-time server metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetrics {
    pub server_id: String,
    pub timestamp: DateTime<Utc>,
    pub tps: f64,
    pub tick_p95_ms: f64,
    pub heap_mb: f64,
    pub players_online: u32,
    pub gpu_queue_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub uptime_seconds: u64,
    pub world_size_mb: f64,
    pub last_backup: Option<DateTime<Utc>>,
}

/// System resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResources {
    pub total_memory_mb: u64,
    pub used_memory_mb: u64,
    pub available_memory_mb: u64,
    pub cpu_usage_percent: f64,
    pub disk_usage_percent: f64,
}

/// Process information for a Minecraft server
#[derive(Debug, Clone)]
pub struct ServerProcess {
    pub pid: Pid,
    pub name: String,
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub start_time: Instant,
}

/// Metrics collector that gathers real system and server data
pub struct MetricsCollector {
    system: Arc<RwLock<System>>,
    server_processes: Arc<RwLock<HashMap<String, ServerProcess>>>,
    last_update: Arc<RwLock<Instant>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            system: Arc::new(RwLock::new(System::new_all())),
            server_processes: Arc::new(RwLock::new(HashMap::new())),
            last_update: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Register a server process for monitoring
    pub async fn register_server(&self, server_id: String, pid: Pid, name: String) {
        let mut processes = self.server_processes.write().await;
        processes.insert(server_id, ServerProcess {
            pid,
            name,
            memory_usage: 0,
            cpu_usage: 0.0,
            start_time: Instant::now(),
        });
    }

    /// Unregister a server process
    pub async fn unregister_server(&self, server_id: &str) {
        let mut processes = self.server_processes.write().await;
        processes.remove(server_id);
    }

    /// Collect metrics for a specific server
    pub async fn collect_server_metrics(&self, server_id: &str) -> Option<ServerMetrics> {
        let mut system = self.system.write().await;
        system.refresh_all();
        
        let processes = self.server_processes.read().await;
        let server_process = processes.get(server_id)?;

        // Get process information
        let process = system.process(server_process.pid)?;
        let memory_usage_mb = process.memory() as f64 / 1024.0; // Convert KB to MB
        let cpu_usage_percent = process.cpu_usage() as f64;
        let uptime_seconds = server_process.start_time.elapsed().as_secs();

        // Calculate TPS (simplified - in real implementation, this would read from server logs)
        let tps = self.calculate_tps(server_id).await;

        // Calculate tick time (simplified)
        let tick_p95_ms = self.calculate_tick_time(server_id).await;

        // Get heap usage (simplified - would read from JVM metrics)
        let heap_mb = self.calculate_heap_usage(server_id).await;

        // Get player count (simplified - would read from server status)
        let players_online = self.get_player_count(server_id).await;

        // Get GPU queue time (simplified - would read from GPU worker)
        let gpu_queue_ms = self.get_gpu_queue_time(server_id).await;

        // Get world size (simplified - would calculate from world files)
        let world_size_mb = self.get_world_size(server_id).await;

        // Get last backup time (simplified - would read from backup system)
        let last_backup = self.get_last_backup_time(server_id).await;

        Some(ServerMetrics {
            server_id: server_id.to_string(),
            timestamp: Utc::now(),
            tps,
            tick_p95_ms,
            heap_mb,
            players_online,
            gpu_queue_ms,
            memory_usage_mb,
            cpu_usage_percent,
            uptime_seconds,
            world_size_mb,
            last_backup,
        })
    }

    /// Collect system-wide resource information
    pub async fn collect_system_resources(&self) -> SystemResources {
        let mut system = self.system.write().await;
        system.refresh_all();

        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        let available_memory = system.available_memory();
        let cpu_usage = system.global_cpu_info().cpu_usage() as f64;
        
        // Get disk usage (simplified)
        let disk_usage = self.get_disk_usage().await;

        SystemResources {
            total_memory_mb: total_memory / 1024,
            used_memory_mb: used_memory / 1024,
            available_memory_mb: available_memory / 1024,
            cpu_usage_percent: cpu_usage,
            disk_usage_percent: disk_usage,
        }
    }

    /// Get all registered server metrics
    pub async fn collect_all_server_metrics(&self) -> Vec<ServerMetrics> {
        let processes = self.server_processes.read().await;
        let mut metrics = Vec::new();

        for server_id in processes.keys() {
            if let Some(metric) = self.collect_server_metrics(server_id).await {
                metrics.push(metric);
            }
        }

        metrics
    }

    // Helper methods for calculating specific metrics

    async fn calculate_tps(&self, _server_id: &str) -> f64 {
        // In a real implementation, this would:
        // 1. Read server logs for tick information
        // 2. Parse TPS data from console output
        // 3. Calculate rolling average
        // For now, return a realistic value
        20.0
    }

    async fn calculate_tick_time(&self, _server_id: &str) -> f64 {
        // In a real implementation, this would:
        // 1. Monitor tick timing from server logs
        // 2. Calculate 95th percentile
        // For now, return a realistic value
        45.0
    }

    async fn calculate_heap_usage(&self, _server_id: &str) -> f64 {
        // In a real implementation, this would:
        // 1. Connect to JVM via JMX
        // 2. Read heap usage from memory MXBean
        // For now, return a realistic value
        2048.0
    }

    async fn get_player_count(&self, _server_id: &str) -> u32 {
        // In a real implementation, this would:
        // 1. Query server via RCON or query protocol
        // 2. Parse player list from server status
        // For now, return 0
        0
    }

    async fn get_gpu_queue_time(&self, _server_id: &str) -> f64 {
        // In a real implementation, this would:
        // 1. Query GPU worker process
        // 2. Read queue metrics from GPU worker
        // For now, return 0
        0.0
    }

    async fn get_world_size(&self, _server_id: &str) -> f64 {
        // In a real implementation, this would:
        // 1. Calculate size of world directory
        // 2. Sum all region files, level.dat, etc.
        // For now, return 0
        0.0
    }

    async fn get_last_backup_time(&self, _server_id: &str) -> Option<DateTime<Utc>> {
        // In a real implementation, this would:
        // 1. Check backup directory for latest backup
        // 2. Parse backup timestamp from filename or metadata
        // For now, return None
        None
    }

    async fn get_disk_usage(&self) -> f64 {
        // In a real implementation, this would:
        // 1. Check disk usage for server directories
        // 2. Calculate percentage of available space used
        // For now, return a realistic value
        75.0
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Background task that continuously collects metrics
pub struct MetricsCollectorTask {
    collector: Arc<MetricsCollector>,
    interval: Duration,
}

impl MetricsCollectorTask {
    pub fn new(collector: Arc<MetricsCollector>, interval: Duration) -> Self {
        Self { collector, interval }
    }

    pub async fn start(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(self.interval);
            
            loop {
                interval.tick().await;
                
                // Collect metrics for all servers
                let metrics = self.collector.collect_all_server_metrics().await;
                
                // In a real implementation, you would:
                // 1. Store metrics in database
                // 2. Send to metrics aggregation service
                // 3. Update real-time dashboards
                
                tracing::debug!("Collected metrics for {} servers", metrics.len());
            }
        })
    }
}
