use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use sysinfo::{System, Pid};
use tracing::{info, warn, error};

use crate::core::{
    error_handler::Result,
    guardian_config::GuardianConfig,
};

/// System resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub memory_total: u64,
    pub memory_available: u64,
    pub disk_usage: f32,
    pub disk_total: u64,
    pub disk_available: u64,
    pub network_in: u64,
    pub network_out: u64,
    pub gpu_usage: Option<f32>,
    pub gpu_memory_usage: Option<f32>,
    pub gpu_memory_total: Option<u64>,
    pub uptime: Duration,
}

/// Server-specific resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetrics {
    pub server_id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub tps: f32,
    pub tick_time: f32,
    pub players_online: u32,
    pub chunks_loaded: u32,
    pub entities: u32,
    pub uptime: Duration,
}

/// Resource monitoring configuration
#[derive(Debug, Clone)]
pub struct ResourceMonitorConfig {
    pub update_interval: Duration,
    pub history_retention: Duration,
    pub alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_usage: f32,
    pub low_tps: f32,
}

impl Default for ResourceMonitorConfig {
    fn default() -> Self {
        Self {
            update_interval: Duration::from_secs(5),
            history_retention: Duration::from_secs(3600), // 1 hour
            alert_thresholds: AlertThresholds {
                cpu_usage: 80.0,
                memory_usage: 85.0,
                disk_usage: 90.0,
                low_tps: 15.0,
            },
        }
    }
}

/// Resource monitor
pub struct ResourceMonitor {
    config: ResourceMonitorConfig,
    guardian_config: Arc<GuardianConfig>,
    system: Arc<RwLock<System>>,
    system_metrics: Arc<RwLock<Vec<SystemMetrics>>>,
    server_metrics: Arc<RwLock<std::collections::HashMap<Uuid, Vec<ServerMetrics>>>>,
    last_network_in: Arc<RwLock<u64>>,
    last_network_out: Arc<RwLock<u64>>,
}

impl ResourceMonitor {
    pub fn new(config: ResourceMonitorConfig, guardian_config: Arc<GuardianConfig>) -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            config,
            guardian_config,
            system: Arc::new(RwLock::new(system)),
            system_metrics: Arc::new(RwLock::new(Vec::new())),
            server_metrics: Arc::new(RwLock::new(std::collections::HashMap::new())),
            last_network_in: Arc::new(RwLock::new(0)),
            last_network_out: Arc::new(RwLock::new(0)),
        }
    }

    /// Start the resource monitoring loop
    pub async fn start(&self) -> Result<()> {
        info!("Starting resource monitor with config: {:?}", self.config);
        
        let mut interval = interval(self.config.update_interval);
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.collect_metrics().await {
                error!("Error collecting metrics: {}", e);
            }
        }
    }

    /// Collect current system and server metrics
    async fn collect_metrics(&self) -> Result<()> {
        let mut system = self.system.write().await;
        system.refresh_all();

        // Collect system metrics
        let system_metrics = self.collect_system_metrics(&system).await?;
        
        // Store system metrics
        {
            let mut metrics = self.system_metrics.write().await;
            metrics.push(system_metrics.clone());
            
            // Clean up old metrics
            let cutoff = chrono::Utc::now() - chrono::Duration::seconds(self.config.history_retention.as_secs() as i64);
            metrics.retain(|m| m.timestamp > cutoff);
        }

        // Check for alerts
        self.check_alerts(&system_metrics).await?;

        Ok(())
    }

    /// Collect system-wide metrics
    async fn collect_system_metrics(&self, system: &System) -> Result<SystemMetrics> {
        let now = chrono::Utc::now();
        
        // CPU usage
        let cpu_usage = system.global_cpu_info().cpu_usage();
        
        // Memory usage
        let memory_total = system.total_memory();
        let memory_available = system.available_memory();
        let memory_usage = if memory_total > 0 {
            ((memory_total - memory_available) as f32 / memory_total as f32) * 100.0
        } else {
            0.0
        };

        // Disk usage
        let disk_total = system.total_swap();
        let disk_available = system.free_swap();
        let disk_usage = if disk_total > 0 {
            ((disk_total - disk_available) as f32 / disk_total as f32) * 100.0
        } else {
            0.0
        };

        // Network usage - simplified for now
        let network_in = 0u64;
        let network_out = 0u64;

        // Calculate network deltas
        let (network_in_delta, network_out_delta) = {
            let mut last_in = self.last_network_in.write().await;
            let mut last_out = self.last_network_out.write().await;
            
            let in_delta = network_in.saturating_sub(*last_in);
            let out_delta = network_out.saturating_sub(*last_out);
            
            *last_in = network_in;
            *last_out = network_out;
            
            (in_delta, out_delta)
        };

        // GPU metrics (if available)
        let (gpu_usage, gpu_memory_usage, gpu_memory_total) = self.get_gpu_metrics().await;

        // System uptime
        let uptime = Duration::from_secs(System::uptime());

        Ok(SystemMetrics {
            timestamp: now,
            cpu_usage,
            memory_usage,
            memory_total,
            memory_available,
            disk_usage,
            disk_total,
            disk_available,
            network_in: network_in_delta,
            network_out: network_out_delta,
            gpu_usage,
            gpu_memory_usage,
            gpu_memory_total,
            uptime,
        })
    }

    /// Get GPU metrics (if GPU is available)
    async fn get_gpu_metrics(&self) -> (Option<f32>, Option<f32>, Option<u64>) {
        if !self.guardian_config.gpu_available() {
            return (None, None, None);
        }

        // This would integrate with the GPU worker to get actual GPU metrics
        // For now, return placeholder values
        (Some(0.0), Some(0.0), Some(0))
    }

    /// Collect metrics for a specific server
    pub async fn collect_server_metrics(&self, server_id: Uuid, pid: u32) -> Result<ServerMetrics> {
        let system = self.system.read().await;
        let now = chrono::Utc::now();

        // Get process information
        let process = system.process(Pid::from_u32(pid));
        let (cpu_usage, memory_usage) = if let Some(process) = process {
            (process.cpu_usage(), process.memory())
        } else {
            (0.0, 0)
        };

        // Server-specific metrics would be collected from the server process
        // This could include TPS, tick time, player count, etc.
        // For now, we'll use placeholder values
        let tps = 20.0; // Would be read from server logs or RCON
        let tick_time = 50.0; // Would be read from server logs
        let players_online = 0; // Would be read from server logs
        let chunks_loaded = 0; // Would be read from server logs
        let entities = 0; // Would be read from server logs

        let metrics = ServerMetrics {
            server_id,
            timestamp: now,
            cpu_usage,
            memory_usage,
            tps,
            tick_time,
            players_online,
            chunks_loaded,
            entities,
            uptime: Duration::from_secs(0), // Would be calculated from server start time
        };

        // Store server metrics
        {
            let mut server_metrics = self.server_metrics.write().await;
            let server_metrics_vec = server_metrics.entry(server_id).or_insert_with(Vec::new);
            server_metrics_vec.push(metrics.clone());
            
            // Clean up old metrics
            let cutoff = chrono::Utc::now() - chrono::Duration::seconds(self.config.history_retention.as_secs() as i64);
            server_metrics_vec.retain(|m| m.timestamp > cutoff);
        }

        Ok(metrics)
    }

    /// Check for resource alerts
    async fn check_alerts(&self, metrics: &SystemMetrics) -> Result<()> {
        let thresholds = &self.config.alert_thresholds;

        // CPU usage alert
        if metrics.cpu_usage > thresholds.cpu_usage {
            warn!("High CPU usage: {:.1}%", metrics.cpu_usage);
        }

        // Memory usage alert
        if metrics.memory_usage > thresholds.memory_usage {
            warn!("High memory usage: {:.1}%", metrics.memory_usage);
        }

        // Disk usage alert
        if metrics.disk_usage > thresholds.disk_usage {
            warn!("High disk usage: {:.1}%", metrics.disk_usage);
        }

        Ok(())
    }

    /// Get current system metrics
    pub async fn get_current_system_metrics(&self) -> Option<SystemMetrics> {
        let metrics = self.system_metrics.read().await;
        metrics.last().cloned()
    }

    /// Get system metrics history
    pub async fn get_system_metrics_history(&self, duration: Duration) -> Vec<SystemMetrics> {
        let metrics = self.system_metrics.read().await;
        let cutoff = chrono::Utc::now() - chrono::Duration::seconds(duration.as_secs() as i64);
        metrics.iter()
            .filter(|m| m.timestamp > cutoff)
            .cloned()
            .collect()
    }

    /// Get server metrics history
    pub async fn get_server_metrics_history(&self, server_id: Uuid, duration: Duration) -> Vec<ServerMetrics> {
        let server_metrics = self.server_metrics.read().await;
        if let Some(metrics) = server_metrics.get(&server_id) {
            let cutoff = chrono::Utc::now() - chrono::Duration::seconds(duration.as_secs() as i64);
            metrics.iter()
                .filter(|m| m.timestamp > cutoff)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get current server metrics
    pub async fn get_current_server_metrics(&self, server_id: Uuid) -> Option<ServerMetrics> {
        let server_metrics = self.server_metrics.read().await;
        server_metrics.get(&server_id)?.last().cloned()
    }

    /// Get resource usage summary
    pub async fn get_resource_summary(&self) -> ResourceSummary {
        let system_metrics = self.system_metrics.read().await;
        let server_metrics = self.server_metrics.read().await;

        let current_system = system_metrics.last().cloned();
        let total_servers = server_metrics.len();
        let running_servers = server_metrics.values()
            .filter(|metrics| !metrics.is_empty())
            .count();

        ResourceSummary {
            current_system,
            total_servers,
            running_servers,
            alert_count: 0, // Would be calculated based on current thresholds
        }
    }
}

/// Resource usage summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSummary {
    pub current_system: Option<SystemMetrics>,
    pub total_servers: usize,
    pub running_servers: usize,
    pub alert_count: u32,
}
