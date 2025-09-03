use crate::config::Config;
use crate::DaemonState;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration, Instant};
use tracing::{info, warn, error, debug};
use serde::{Deserialize, Serialize};

/// Metrics collector for gathering and exposing system metrics
pub struct MetricsCollector {
    config: Config,
    metrics: Arc<RwLock<SystemMetrics>>,
    is_running: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: Instant,
    pub server_metrics: ServerMetrics,
    pub gpu_metrics: GpuMetrics,
    pub system_metrics: SystemResourceMetrics,
    pub guardian_metrics: GuardianMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetrics {
    pub tps: f64,
    pub tick_time_ms: f64,
    pub memory_used_mb: u64,
    pub memory_max_mb: u64,
    pub players_online: u32,
    pub chunks_loaded: u32,
    pub entities_count: u32,
    pub block_entities_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuMetrics {
    pub is_healthy: bool,
    pub chunks_generated: u64,
    pub generation_time_ms: f64,
    pub cache_hit_rate: f64,
    pub memory_used_mb: u64,
    pub memory_max_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResourceMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub disk_usage_percent: f64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardianMetrics {
    pub frozen_entities: u32,
    pub frozen_block_entities: u32,
    pub applied_rules: u32,
    pub gpu_chunks_generated: u64,
    pub cpu_chunks_generated: u64,
    pub worldgen_acceleration_ratio: f64,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing Metrics Collector...");
        
        Ok(Self {
            config,
            metrics: Arc::new(RwLock::new(SystemMetrics::default())),
            is_running: Arc::new(RwLock::new(false)),
        })
    }
    
    /// Start the metrics collector
    pub async fn start(&self) -> Result<()> {
        info!("Starting Metrics Collector...");
        
        {
            let mut running_guard = self.is_running.write().await;
            *running_guard = true;
        }
        
        // Start metrics collection task
        self.start_metrics_collection().await;
        
        info!("Metrics Collector started");
        Ok(())
    }
    
    /// Stop the metrics collector
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Metrics Collector...");
        
        {
            let mut running_guard = self.is_running.write().await;
            *running_guard = false;
        }
        
        info!("Metrics Collector stopped");
        Ok(())
    }
    
    /// Start metrics collection task
    async fn start_metrics_collection(&self) {
        let config = self.config.clone();
        let metrics = self.metrics.clone();
        let is_running = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10)); // Collect every 10 seconds
            
            loop {
                interval.tick().await;
                
                // Check if we should still be running
                {
                    let running_guard = is_running.read().await;
                    if !*running_guard {
                        break;
                    }
                }
                
                // Collect metrics
                if let Err(e) = Self::collect_metrics_internal(&config, &metrics).await {
                    error!("Failed to collect metrics: {}", e);
                }
            }
        });
    }
    
    /// Update metrics from daemon state
    pub async fn update_metrics(&self, state: &Arc<RwLock<DaemonState>>) -> Result<()> {
        Self::collect_metrics_internal(&self.config, &self.metrics).await
    }
    
    /// Internal method to collect metrics
    async fn collect_metrics_internal(
        config: &Config,
        metrics: &Arc<RwLock<SystemMetrics>>,
    ) -> Result<()> {
        let timestamp = Instant::now();
        
        // Collect server metrics (placeholder - would integrate with actual server)
        let server_metrics = Self::collect_server_metrics().await?;
        
        // Collect GPU metrics (placeholder - would integrate with GPU worker)
        let gpu_metrics = Self::collect_gpu_metrics().await?;
        
        // Collect system resource metrics
        let system_metrics = Self::collect_system_metrics().await?;
        
        // Collect Guardian-specific metrics
        let guardian_metrics = Self::collect_guardian_metrics().await?;
        
        let new_metrics = SystemMetrics {
            timestamp,
            server_metrics,
            gpu_metrics,
            system_metrics,
            guardian_metrics,
        };
        
        {
            let mut metrics_guard = metrics.write().await;
            *metrics_guard = new_metrics;
        }
        
        debug!("Metrics collected successfully");
        Ok(())
    }
    
    /// Collect server metrics
    async fn collect_server_metrics() -> Result<ServerMetrics> {
        // This would integrate with the actual Minecraft server to get real metrics
        // For now, return placeholder data
        
        Ok(ServerMetrics {
            tps: 20.0, // Target TPS
            tick_time_ms: 45.0, // Average tick time
            memory_used_mb: 2048, // Used memory
            memory_max_mb: 10240, // Max memory (10GB)
            players_online: 0, // Online players
            chunks_loaded: 100, // Loaded chunks
            entities_count: 150, // Entity count
            block_entities_count: 50, // Block entity count
        })
    }
    
    /// Collect GPU metrics
    async fn collect_gpu_metrics() -> Result<GpuMetrics> {
        // This would integrate with the GPU worker to get real metrics
        // For now, return placeholder data
        
        Ok(GpuMetrics {
            is_healthy: true,
            chunks_generated: 1000,
            generation_time_ms: 15.0,
            cache_hit_rate: 0.85,
            memory_used_mb: 512,
            memory_max_mb: 2048,
        })
    }
    
    /// Collect system resource metrics
    async fn collect_system_metrics() -> Result<SystemResourceMetrics> {
        // This would use system APIs to get real resource usage
        // For now, return placeholder data
        
        Ok(SystemResourceMetrics {
            cpu_usage_percent: 45.0,
            memory_usage_percent: 65.0,
            disk_usage_percent: 30.0,
            network_rx_bytes: 1024 * 1024, // 1MB
            network_tx_bytes: 512 * 1024,  // 512KB
        })
    }
    
    /// Collect Guardian-specific metrics
    async fn collect_guardian_metrics() -> Result<GuardianMetrics> {
        // This would integrate with the Guardian Agent to get real metrics
        // For now, return placeholder data
        
        Ok(GuardianMetrics {
            frozen_entities: 0,
            frozen_block_entities: 0,
            applied_rules: 15,
            gpu_chunks_generated: 800,
            cpu_chunks_generated: 200,
            worldgen_acceleration_ratio: 4.0, // 4x faster with GPU
        })
    }
    
    /// Get current metrics
    pub async fn get_metrics(&self) -> SystemMetrics {
        let metrics_guard = self.metrics.read().await;
        metrics_guard.clone()
    }
    
    /// Get metrics in Prometheus format
    pub async fn get_prometheus_metrics(&self) -> String {
        let metrics_guard = self.metrics.read().await;
        let metrics = metrics_guard.clone();
        
        let mut prometheus_output = String::new();
        
        // Server metrics
        prometheus_output.push_str(&format!(
            "# HELP minecraft_tps Minecraft server TPS\n# TYPE minecraft_tps gauge\nminecraft_tps {}\n",
            metrics.server_metrics.tps
        ));
        
        prometheus_output.push_str(&format!(
            "# HELP minecraft_tick_time_ms Minecraft server tick time in milliseconds\n# TYPE minecraft_tick_time_ms gauge\nminecraft_tick_time_ms {}\n",
            metrics.server_metrics.tick_time_ms
        ));
        
        prometheus_output.push_str(&format!(
            "# HELP minecraft_memory_used_mb Minecraft server memory used in MB\n# TYPE minecraft_memory_used_mb gauge\nminecraft_memory_used_mb {}\n",
            metrics.server_metrics.memory_used_mb
        ));
        
        prometheus_output.push_str(&format!(
            "# HELP minecraft_players_online Minecraft server players online\n# TYPE minecraft_players_online gauge\nminecraft_players_online {}\n",
            metrics.server_metrics.players_online
        ));
        
        // GPU metrics
        prometheus_output.push_str(&format!(
            "# HELP gpu_worker_healthy GPU worker health status\n# TYPE gpu_worker_healthy gauge\ngpu_worker_healthy {}\n",
            if metrics.gpu_metrics.is_healthy { 1 } else { 0 }
        ));
        
        prometheus_output.push_str(&format!(
            "# HELP gpu_chunks_generated_total Total chunks generated by GPU\n# TYPE gpu_chunks_generated_total counter\ngpu_chunks_generated_total {}\n",
            metrics.gpu_metrics.chunks_generated
        ));
        
        // Guardian metrics
        prometheus_output.push_str(&format!(
            "# HELP guardian_frozen_entities Number of frozen entities\n# TYPE guardian_frozen_entities gauge\nguardian_frozen_entities {}\n",
            metrics.guardian_metrics.frozen_entities
        ));
        
        prometheus_output.push_str(&format!(
            "# HELP guardian_applied_rules Number of applied compatibility rules\n# TYPE guardian_applied_rules gauge\nguardian_applied_rules {}\n",
            metrics.guardian_metrics.applied_rules
        ));
        
        prometheus_output.push_str(&format!(
            "# HELP guardian_worldgen_acceleration_ratio World generation acceleration ratio\n# TYPE guardian_worldgen_acceleration_ratio gauge\nguardian_worldgen_acceleration_ratio {}\n",
            metrics.guardian_metrics.worldgen_acceleration_ratio
        ));
        
        prometheus_output
    }
    
    /// Get metrics statistics
    pub async fn get_stats(&self) -> serde_json::Map<String, serde_json::Value> {
        let mut stats = serde_json::Map::new();
        
        let metrics_guard = self.metrics.read().await;
        let metrics = metrics_guard.clone();
        
        stats.insert("last_collection".to_string(), 
                    serde_json::Value::String(metrics.timestamp.to_string()));
        stats.insert("server_tps".to_string(), 
                    serde_json::Value::Number(serde_json::Number::from_f64(metrics.server_metrics.tps).unwrap()));
        stats.insert("gpu_healthy".to_string(), 
                    serde_json::Value::Bool(metrics.gpu_metrics.is_healthy));
        stats.insert("frozen_entities".to_string(), 
                    serde_json::Value::Number(metrics.guardian_metrics.frozen_entities.into()));
        
        stats
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            timestamp: Instant::now(),
            server_metrics: ServerMetrics {
                tps: 0.0,
                tick_time_ms: 0.0,
                memory_used_mb: 0,
                memory_max_mb: 0,
                players_online: 0,
                chunks_loaded: 0,
                entities_count: 0,
                block_entities_count: 0,
            },
            gpu_metrics: GpuMetrics {
                is_healthy: false,
                chunks_generated: 0,
                generation_time_ms: 0.0,
                cache_hit_rate: 0.0,
                memory_used_mb: 0,
                memory_max_mb: 0,
            },
            system_metrics: SystemResourceMetrics {
                cpu_usage_percent: 0.0,
                memory_usage_percent: 0.0,
                disk_usage_percent: 0.0,
                network_rx_bytes: 0,
                network_tx_bytes: 0,
            },
            guardian_metrics: GuardianMetrics {
                frozen_entities: 0,
                frozen_block_entities: 0,
                applied_rules: 0,
                gpu_chunks_generated: 0,
                cpu_chunks_generated: 0,
                worldgen_acceleration_ratio: 1.0,
            },
        }
    }
}
