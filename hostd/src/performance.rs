use crate::error::{GuardianError, utils as error_utils};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::{interval, sleep};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// Performance metrics for the Guardian Platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub timestamp: u64,
    pub server_id: String,
    
    // Server performance metrics
    pub tps: f64,
    pub tick_time_ms: f64,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f64,
    pub disk_io_mb_s: f64,
    pub network_io_mb_s: f64,
    
    // Game-specific metrics
    pub player_count: u32,
    pub chunk_count: u32,
    pub entity_count: u32,
    pub block_entity_count: u32,
    pub loaded_chunks: u32,
    
    // System metrics
    pub gc_time_ms: f64,
    pub gc_frequency: u32,
    pub thread_count: u32,
    pub open_file_descriptors: u32,
    
    // Custom metrics
    pub custom_metrics: HashMap<String, f64>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            server_id: String::new(),
            tps: 20.0,
            tick_time_ms: 50.0,
            memory_usage_mb: 0,
            cpu_usage_percent: 0.0,
            disk_io_mb_s: 0.0,
            network_io_mb_s: 0.0,
            player_count: 0,
            chunk_count: 0,
            entity_count: 0,
            block_entity_count: 0,
            loaded_chunks: 0,
            gc_time_ms: 0.0,
            gc_frequency: 0,
            thread_count: 0,
            open_file_descriptors: 0,
            custom_metrics: HashMap::new(),
        }
    }
}

/// Performance thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    pub tps_warning: f64,
    pub tps_critical: f64,
    pub tick_time_warning_ms: f64,
    pub tick_time_critical_ms: f64,
    pub memory_warning_mb: u64,
    pub memory_critical_mb: u64,
    pub cpu_warning_percent: f64,
    pub cpu_critical_percent: f64,
    pub disk_io_warning_mb_s: f64,
    pub disk_io_critical_mb_s: f64,
    pub network_io_warning_mb_s: f64,
    pub network_io_critical_mb_s: f64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            tps_warning: 18.0,
            tps_critical: 15.0,
            tick_time_warning_ms: 60.0,
            tick_time_critical_ms: 100.0,
            memory_warning_mb: 6000,
            memory_critical_mb: 8000,
            cpu_warning_percent: 80.0,
            cpu_critical_percent: 95.0,
            disk_io_warning_mb_s: 100.0,
            disk_io_critical_mb_s: 200.0,
            network_io_warning_mb_s: 50.0,
            network_io_critical_mb_s: 100.0,
        }
    }
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub alert_id: String,
    pub server_id: String,
    pub alert_type: PerformanceAlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: u64,
    pub metrics: PerformanceMetrics,
    pub threshold: f64,
    pub current_value: f64,
    pub resolved: bool,
    pub resolved_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PerformanceAlertType {
    TPSLow,
    TickTimeHigh,
    MemoryHigh,
    CPUHigh,
    DiskIOHigh,
    NetworkIOHigh,
    EntityCountHigh,
    ChunkCountHigh,
    GCTimeHigh,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Performance optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    pub recommendation_id: String,
    pub server_id: String,
    pub category: OptimizationCategory,
    pub priority: RecommendationPriority,
    pub title: String,
    pub description: String,
    pub expected_improvement: f64,
    pub implementation_effort: ImplementationEffort,
    pub risk_level: RiskLevel,
    pub actions: Vec<OptimizationAction>,
    pub created_at: u64,
    pub applied: bool,
    pub applied_at: Option<u64>,
    pub effectiveness: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptimizationCategory {
    Memory,
    CPU,
    Network,
    Storage,
    GarbageCollection,
    Threading,
    Caching,
    Configuration,
    ModOptimization,
    WorldGeneration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImplementationEffort {
    Low,
    Medium,
    High,
    VeryHigh,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationAction {
    pub action_type: ActionType,
    pub description: String,
    pub parameters: HashMap<String, String>,
    pub estimated_impact: f64,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionType {
    AdjustMemoryAllocation,
    ModifyGCSettings,
    EnableGPUAcceleration,
    DisableMod,
    OptimizeConfiguration,
    ScaleResources,
    EnableCaching,
    AdjustThreading,
    OptimizeWorldGeneration,
    NetworkOptimization,
}

/// Performance profiler
pub struct PerformanceProfiler {
    metrics_history: Arc<RwLock<Vec<PerformanceMetrics>>>,
    alerts: Arc<RwLock<Vec<PerformanceAlert>>>,
    recommendations: Arc<RwLock<Vec<OptimizationRecommendation>>>,
    thresholds: PerformanceThresholds,
    is_running: Arc<RwLock<bool>>,
    collection_interval: Duration,
    history_retention: Duration,
}

impl PerformanceProfiler {
    pub fn new(collection_interval: Duration, history_retention: Duration) -> Self {
        Self {
            metrics_history: Arc::new(RwLock::new(Vec::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            recommendations: Arc::new(RwLock::new(Vec::new())),
            thresholds: PerformanceThresholds::default(),
            is_running: Arc::new(RwLock::new(false)),
            collection_interval,
            history_retention,
        }
    }

    /// Start the performance profiler
    pub async fn start(&self) -> Result<(), GuardianError> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(error_utils::internal_error(
                "performance_profiler",
                "start",
                "Performance profiler is already running",
            ));
        }
        *is_running = true;
        drop(is_running);

        info!("Starting performance profiler with interval: {:?}", self.collection_interval);

        let metrics_history = self.metrics_history.clone();
        let alerts = self.alerts.clone();
        let recommendations = self.recommendations.clone();
        let thresholds = self.thresholds.clone();
        let is_running = self.is_running.clone();
        let collection_interval = self.collection_interval;
        let history_retention = self.history_retention;

        tokio::spawn(async move {
            let mut interval_timer = interval(collection_interval);
            
            while *is_running.read().await {
                interval_timer.tick().await;
                
                // Collect performance metrics
                if let Ok(metrics) = Self::collect_metrics().await {
                    // Store metrics
                    {
                        let mut history = metrics_history.write().await;
                        history.push(metrics.clone());
                        
                        // Clean up old metrics
                        let cutoff_time = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() - history_retention.as_secs();
                        
                        history.retain(|m| m.timestamp > cutoff_time);
                    }
                    
                    // Check for alerts
                    if let Err(e) = Self::check_alerts(&metrics, &thresholds, &alerts).await {
                        error!("Failed to check performance alerts: {}", e);
                    }
                    
                    // Generate recommendations
                    if let Err(e) = Self::generate_recommendations(&metrics, &recommendations).await {
                        error!("Failed to generate performance recommendations: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the performance profiler
    pub async fn stop(&self) {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        info!("Performance profiler stopped");
    }

    /// Collect current performance metrics
    async fn collect_metrics() -> Result<PerformanceMetrics, GuardianError> {
        let mut metrics = PerformanceMetrics::default();
        metrics.server_id = "guardian-server".to_string(); // TODO: Get actual server ID
        
        // Collect system metrics
        metrics.memory_usage_mb = Self::get_memory_usage().await?;
        metrics.cpu_usage_percent = Self::get_cpu_usage().await?;
        metrics.disk_io_mb_s = Self::get_disk_io().await?;
        metrics.network_io_mb_s = Self::get_network_io().await?;
        
        // Collect JVM metrics (simplified)
        metrics.gc_time_ms = Self::get_gc_time().await?;
        metrics.gc_frequency = Self::get_gc_frequency().await?;
        metrics.thread_count = Self::get_thread_count().await?;
        
        // Collect game metrics (simplified)
        metrics.tps = Self::get_tps().await?;
        metrics.tick_time_ms = Self::get_tick_time().await?;
        metrics.player_count = Self::get_player_count().await?;
        metrics.chunk_count = Self::get_chunk_count().await?;
        metrics.entity_count = Self::get_entity_count().await?;
        metrics.block_entity_count = Self::get_block_entity_count().await?;
        metrics.loaded_chunks = Self::get_loaded_chunks().await?;
        
        Ok(metrics)
    }

    /// Check for performance alerts
    async fn check_alerts(
        metrics: &PerformanceMetrics,
        thresholds: &PerformanceThresholds,
        alerts: &Arc<RwLock<Vec<PerformanceAlert>>>,
    ) -> Result<(), GuardianError> {
        let mut new_alerts = Vec::new();
        
        // Check TPS
        if metrics.tps < thresholds.tps_critical {
            new_alerts.push(PerformanceAlert {
                alert_id: Uuid::new_v4().to_string(),
                server_id: metrics.server_id.clone(),
                alert_type: PerformanceAlertType::TPSLow,
                severity: AlertSeverity::Critical,
                message: format!("TPS critically low: {:.1}", metrics.tps),
                timestamp: metrics.timestamp,
                metrics: metrics.clone(),
                threshold: thresholds.tps_critical,
                current_value: metrics.tps,
                resolved: false,
                resolved_at: None,
            });
        } else if metrics.tps < thresholds.tps_warning {
            new_alerts.push(PerformanceAlert {
                alert_id: Uuid::new_v4().to_string(),
                server_id: metrics.server_id.clone(),
                alert_type: PerformanceAlertType::TPSLow,
                severity: AlertSeverity::Warning,
                message: format!("TPS low: {:.1}", metrics.tps),
                timestamp: metrics.timestamp,
                metrics: metrics.clone(),
                threshold: thresholds.tps_warning,
                current_value: metrics.tps,
                resolved: false,
                resolved_at: None,
            });
        }
        
        // Check tick time
        if metrics.tick_time_ms > thresholds.tick_time_critical_ms {
            new_alerts.push(PerformanceAlert {
                alert_id: Uuid::new_v4().to_string(),
                server_id: metrics.server_id.clone(),
                alert_type: PerformanceAlertType::TickTimeHigh,
                severity: AlertSeverity::Critical,
                message: format!("Tick time critically high: {:.1}ms", metrics.tick_time_ms),
                timestamp: metrics.timestamp,
                metrics: metrics.clone(),
                threshold: thresholds.tick_time_critical_ms,
                current_value: metrics.tick_time_ms,
                resolved: false,
                resolved_at: None,
            });
        } else if metrics.tick_time_ms > thresholds.tick_time_warning_ms {
            new_alerts.push(PerformanceAlert {
                alert_id: Uuid::new_v4().to_string(),
                server_id: metrics.server_id.clone(),
                alert_type: PerformanceAlertType::TickTimeHigh,
                severity: AlertSeverity::Warning,
                message: format!("Tick time high: {:.1}ms", metrics.tick_time_ms),
                timestamp: metrics.timestamp,
                metrics: metrics.clone(),
                threshold: thresholds.tick_time_warning_ms,
                current_value: metrics.tick_time_ms,
                resolved: false,
                resolved_at: None,
            });
        }
        
        // Check memory usage
        if metrics.memory_usage_mb > thresholds.memory_critical_mb {
            new_alerts.push(PerformanceAlert {
                alert_id: Uuid::new_v4().to_string(),
                server_id: metrics.server_id.clone(),
                alert_type: PerformanceAlertType::MemoryHigh,
                severity: AlertSeverity::Critical,
                message: format!("Memory usage critically high: {}MB", metrics.memory_usage_mb),
                timestamp: metrics.timestamp,
                metrics: metrics.clone(),
                threshold: thresholds.memory_critical_mb as f64,
                current_value: metrics.memory_usage_mb as f64,
                resolved: false,
                resolved_at: None,
            });
        } else if metrics.memory_usage_mb > thresholds.memory_warning_mb {
            new_alerts.push(PerformanceAlert {
                alert_id: Uuid::new_v4().to_string(),
                server_id: metrics.server_id.clone(),
                alert_type: PerformanceAlertType::MemoryHigh,
                severity: AlertSeverity::Warning,
                message: format!("Memory usage high: {}MB", metrics.memory_usage_mb),
                timestamp: metrics.timestamp,
                metrics: metrics.clone(),
                threshold: thresholds.memory_warning_mb as f64,
                current_value: metrics.memory_usage_mb as f64,
                resolved: false,
                resolved_at: None,
            });
        }
        
        // Add new alerts
        if !new_alerts.is_empty() {
            let mut alerts_guard = alerts.write().await;
            for alert in new_alerts {
                info!("Performance alert: {}", alert.message);
                alerts_guard.push(alert);
            }
        }
        
        Ok(())
    }

    /// Generate performance optimization recommendations
    async fn generate_recommendations(
        metrics: &PerformanceMetrics,
        recommendations: &Arc<RwLock<Vec<OptimizationRecommendation>>>,
    ) -> Result<(), GuardianError> {
        let mut new_recommendations = Vec::new();
        
        // Memory optimization recommendations
        if metrics.memory_usage_mb > 4000 {
            new_recommendations.push(OptimizationRecommendation {
                recommendation_id: Uuid::new_v4().to_string(),
                server_id: metrics.server_id.clone(),
                category: OptimizationCategory::Memory,
                priority: RecommendationPriority::High,
                title: "Optimize Memory Usage".to_string(),
                description: "High memory usage detected. Consider optimizing memory allocation and garbage collection.".to_string(),
                expected_improvement: 20.0,
                implementation_effort: ImplementationEffort::Medium,
                risk_level: RiskLevel::Low,
                actions: vec![
                    OptimizationAction {
                        action_type: ActionType::AdjustMemoryAllocation,
                        description: "Increase heap size".to_string(),
                        parameters: {
                            let mut params = HashMap::new();
                            params.insert("heap_size".to_string(), "8GB".to_string());
                            params
                        },
                        estimated_impact: 15.0,
                        risk_level: RiskLevel::Low,
                    },
                    OptimizationAction {
                        action_type: ActionType::ModifyGCSettings,
                        description: "Optimize garbage collection settings".to_string(),
                        parameters: {
                            let mut params = HashMap::new();
                            params.insert("gc_algorithm".to_string(), "G1GC".to_string());
                            params.insert("max_gc_pause".to_string(), "100ms".to_string());
                            params
                        },
                        estimated_impact: 10.0,
                        risk_level: RiskLevel::Low,
                    },
                ],
                created_at: metrics.timestamp,
                applied: false,
                applied_at: None,
                effectiveness: None,
            });
        }
        
        // CPU optimization recommendations
        if metrics.cpu_usage_percent > 80.0 {
            new_recommendations.push(OptimizationRecommendation {
                recommendation_id: Uuid::new_v4().to_string(),
                server_id: metrics.server_id.clone(),
                category: OptimizationCategory::CPU,
                priority: RecommendationPriority::High,
                title: "Optimize CPU Usage".to_string(),
                description: "High CPU usage detected. Consider optimizing thread usage and reducing computational load.".to_string(),
                expected_improvement: 25.0,
                implementation_effort: ImplementationEffort::High,
                risk_level: RiskLevel::Medium,
                actions: vec![
                    OptimizationAction {
                        action_type: ActionType::EnableGPUAcceleration,
                        description: "Enable GPU acceleration for world generation".to_string(),
                        parameters: {
                            let mut params = HashMap::new();
                            params.insert("gpu_enabled".to_string(), "true".to_string());
                            params
                        },
                        estimated_impact: 20.0,
                        risk_level: RiskLevel::Medium,
                    },
                    OptimizationAction {
                        action_type: ActionType::AdjustThreading,
                        description: "Optimize thread pool configuration".to_string(),
                        parameters: {
                            let mut params = HashMap::new();
                            params.insert("thread_count".to_string(), "8".to_string());
                            params
                        },
                        estimated_impact: 10.0,
                        risk_level: RiskLevel::Low,
                    },
                ],
                created_at: metrics.timestamp,
                applied: false,
                applied_at: None,
                effectiveness: None,
            });
        }
        
        // Add new recommendations
        if !new_recommendations.is_empty() {
            let mut recommendations_guard = recommendations.write().await;
            for recommendation in new_recommendations {
                info!("Performance recommendation: {}", recommendation.title);
                recommendations_guard.push(recommendation);
            }
        }
        
        Ok(())
    }

    // Helper functions for collecting metrics (simplified implementations)
    async fn get_memory_usage() -> Result<u64, GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would use system-specific APIs
        Ok(2048) // Placeholder
    }

    async fn get_cpu_usage() -> Result<f64, GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would use system-specific APIs
        Ok(45.0) // Placeholder
    }

    async fn get_disk_io() -> Result<f64, GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would use system-specific APIs
        Ok(10.5) // Placeholder
    }

    async fn get_network_io() -> Result<f64, GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would use system-specific APIs
        Ok(5.2) // Placeholder
    }

    async fn get_gc_time() -> Result<f64, GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would use JVM-specific APIs
        Ok(25.0) // Placeholder
    }

    async fn get_gc_frequency() -> Result<u32, GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would use JVM-specific APIs
        Ok(5) // Placeholder
    }

    async fn get_thread_count() -> Result<u32, GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would use system-specific APIs
        Ok(32) // Placeholder
    }

    async fn get_tps() -> Result<f64, GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would get this from the Minecraft server
        Ok(19.5) // Placeholder
    }

    async fn get_tick_time() -> Result<f64, GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would get this from the Minecraft server
        Ok(45.0) // Placeholder
    }

    async fn get_player_count() -> Result<u32, GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would get this from the Minecraft server
        Ok(12) // Placeholder
    }

    async fn get_chunk_count() -> Result<u32, GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would get this from the Minecraft server
        Ok(1500) // Placeholder
    }

    async fn get_entity_count() -> Result<u32, GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would get this from the Minecraft server
        Ok(2500) // Placeholder
    }

    async fn get_block_entity_count() -> Result<u32, GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would get this from the Minecraft server
        Ok(150) // Placeholder
    }

    async fn get_loaded_chunks() -> Result<u32, GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would get this from the Minecraft server
        Ok(800) // Placeholder
    }

    /// Get performance metrics history
    pub async fn get_metrics_history(&self, duration: Option<Duration>) -> Vec<PerformanceMetrics> {
        let history = self.metrics_history.read().await;
        
        if let Some(duration) = duration {
            let cutoff_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() - duration.as_secs();
            
            history.iter()
                .filter(|m| m.timestamp > cutoff_time)
                .cloned()
                .collect()
        } else {
            history.clone()
        }
    }

    /// Get active performance alerts
    pub async fn get_active_alerts(&self) -> Vec<PerformanceAlert> {
        let alerts = self.alerts.read().await;
        alerts.iter()
            .filter(|a| !a.resolved)
            .cloned()
            .collect()
    }

    /// Get performance optimization recommendations
    pub async fn get_recommendations(&self, category: Option<OptimizationCategory>) -> Vec<OptimizationRecommendation> {
        let recommendations = self.recommendations.read().await;
        
        if let Some(category) = category {
            recommendations.iter()
                .filter(|r| r.category == category && !r.applied)
                .cloned()
                .collect()
        } else {
            recommendations.iter()
                .filter(|r| !r.applied)
                .cloned()
                .collect()
        }
    }

    /// Apply an optimization recommendation
    pub async fn apply_recommendation(&self, recommendation_id: &str) -> Result<(), GuardianError> {
        let mut recommendations = self.recommendations.write().await;
        
        if let Some(recommendation) = recommendations.iter_mut().find(|r| r.recommendation_id == recommendation_id) {
            if recommendation.applied {
                return Err(error_utils::resource_error(
                    crate::error::ResourceErrorKind::AlreadyExists,
                    "recommendation",
                    recommendation_id,
                    "Recommendation already applied",
                ));
            }
            
            // Apply the recommendation (simplified)
            info!("Applying optimization recommendation: {}", recommendation.title);
            
            recommendation.applied = true;
            recommendation.applied_at = Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            );
            
            Ok(())
        } else {
            Err(error_utils::resource_error(
                crate::error::ResourceErrorKind::NotFound,
                "recommendation",
                recommendation_id,
                "Recommendation not found",
            ))
        }
    }

    /// Get performance summary
    pub async fn get_performance_summary(&self) -> PerformanceSummary {
        let history = self.metrics_history.read().await;
        let alerts = self.alerts.read().await;
        let recommendations = self.recommendations.read().await;
        
        let recent_metrics = if history.len() > 10 {
            &history[history.len() - 10..]
        } else {
            &history
        };
        
        let avg_tps = if !recent_metrics.is_empty() {
            recent_metrics.iter().map(|m| m.tps).sum::<f64>() / recent_metrics.len() as f64
        } else {
            0.0
        };
        
        let avg_tick_time = if !recent_metrics.is_empty() {
            recent_metrics.iter().map(|m| m.tick_time_ms).sum::<f64>() / recent_metrics.len() as f64
        } else {
            0.0
        };
        
        let avg_memory = if !recent_metrics.is_empty() {
            recent_metrics.iter().map(|m| m.memory_usage_mb).sum::<u64>() / recent_metrics.len() as u64
        } else {
            0
        };
        
        let active_alerts = alerts.iter().filter(|a| !a.resolved).count();
        let pending_recommendations = recommendations.iter().filter(|r| !r.applied).count();
        
        PerformanceSummary {
            avg_tps,
            avg_tick_time_ms: avg_tick_time,
            avg_memory_usage_mb: avg_memory,
            active_alerts,
            pending_recommendations,
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

/// Performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub avg_tps: f64,
    pub avg_tick_time_ms: f64,
    pub avg_memory_usage_mb: u64,
    pub active_alerts: usize,
    pub pending_recommendations: usize,
    pub last_updated: u64,
}
