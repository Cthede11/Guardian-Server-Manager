use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use tracing::debug;
use sysinfo::System;

/// Performance metrics for monitoring system health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub memory_total: u64,
    pub memory_available: u64,
    pub disk_usage: u64,
    pub disk_total: u64,
    pub network_in: u64,
    pub network_out: u64,
    pub active_connections: u32,
    pub active_servers: u32,
    pub database_connections: u32,
    pub websocket_connections: u32,
    pub response_times: ResponseTimeMetrics,
    pub error_rates: ErrorRateMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeMetrics {
    pub api_avg: Duration,
    pub api_p95: Duration,
    pub api_p99: Duration,
    pub database_avg: Duration,
    pub database_p95: Duration,
    pub database_p99: Duration,
    pub websocket_avg: Duration,
    pub websocket_p95: Duration,
    pub websocket_p99: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRateMetrics {
    pub api_errors_per_minute: f32,
    pub database_errors_per_minute: f32,
    pub websocket_errors_per_minute: f32,
    pub total_errors_per_minute: f32,
    pub error_rate_percentage: f32,
}

/// Performance thresholds for alerting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    pub cpu_usage_warning: f32,
    pub cpu_usage_critical: f32,
    pub memory_usage_warning: f32,
    pub memory_usage_critical: f32,
    pub disk_usage_warning: f32,
    pub disk_usage_critical: f32,
    pub response_time_warning: Duration,
    pub response_time_critical: Duration,
    pub error_rate_warning: f32,
    pub error_rate_critical: f32,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            cpu_usage_warning: 70.0,
            cpu_usage_critical: 90.0,
            memory_usage_warning: 80.0,
            memory_usage_critical: 95.0,
            disk_usage_warning: 85.0,
            disk_usage_critical: 95.0,
            response_time_warning: Duration::from_millis(1000),
            response_time_critical: Duration::from_millis(5000),
            error_rate_warning: 5.0,
            error_rate_critical: 10.0,
        }
    }
}

/// Performance alert levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: AlertLevel,
    pub metric: String,
    pub value: f64,
    pub threshold: f64,
    pub message: String,
    pub recommendations: Vec<String>,
}

/// Performance monitor for tracking system metrics
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<Vec<PerformanceMetrics>>>,
    thresholds: PerformanceThresholds,
    alerts: Arc<RwLock<Vec<PerformanceAlert>>>,
    response_times: Arc<RwLock<HashMap<String, Vec<Duration>>>>,
    error_counts: Arc<RwLock<HashMap<String, u32>>>,
    system: Arc<RwLock<System>>,
    max_metrics_history: usize,
}

impl PerformanceMonitor {
    pub fn new(thresholds: PerformanceThresholds) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
            thresholds,
            alerts: Arc::new(RwLock::new(Vec::new())),
            response_times: Arc::new(RwLock::new(HashMap::new())),
            error_counts: Arc::new(RwLock::new(HashMap::new())),
            system: Arc::new(RwLock::new(System::new_all())),
            max_metrics_history: 1000,
        }
    }
    
    /// Start monitoring performance metrics
    pub async fn start_monitoring(&self) -> Result<(), Box<dyn std::error::Error>> {
        let metrics = self.metrics.clone();
        let system = self.system.clone();
        let thresholds = self.thresholds.clone();
        let alerts = self.alerts.clone();
        let response_times = self.response_times.clone();
        let error_counts = self.error_counts.clone();
        let max_history = self.max_metrics_history;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            
            loop {
                interval.tick().await;
                
                // Collect system metrics
                let mut sys = system.write().await;
                sys.refresh_all();
                
                let cpu_usage = sys.global_cpu_info().cpu_usage();
                let memory_usage = sys.used_memory();
                let memory_total = sys.total_memory();
                let memory_available = sys.available_memory();
                
                // Calculate disk usage (simplified)
                let disk_usage = 0; // Would need to implement disk monitoring
                let disk_total = 0;
                
                // Calculate network usage (simplified)
                let network_in = 0; // Would need to implement network monitoring
                let network_out = 0;
                
                // Count active connections and servers
                let active_connections = 0; // Would need to track from connection manager
                let active_servers = 0; // Would need to track from server manager
                let database_connections = 1; // Would need to track from database pool
                let websocket_connections = 0; // Would need to track from websocket manager
                
                // Calculate response times
                let response_times_guard = response_times.read().await;
                let api_response_times = response_times_guard.get("api").cloned().unwrap_or_default();
                let database_response_times = response_times_guard.get("database").cloned().unwrap_or_default();
                let websocket_response_times = response_times_guard.get("websocket").cloned().unwrap_or_default();
                drop(response_times_guard);
                
                let response_time_metrics = Self::calculate_response_times(
                    api_response_times,
                    database_response_times,
                    websocket_response_times,
                );
                
                // Calculate error rates
                let error_counts_guard = error_counts.read().await;
                let api_errors = error_counts_guard.get("api").cloned().unwrap_or(0);
                let database_errors = error_counts_guard.get("database").cloned().unwrap_or(0);
                let websocket_errors = error_counts_guard.get("websocket").cloned().unwrap_or(0);
                drop(error_counts_guard);
                
                let error_rate_metrics = Self::calculate_error_rates(api_errors, database_errors, websocket_errors);
                
                // Create performance metrics
                let metrics_entry = PerformanceMetrics {
                    timestamp: chrono::Utc::now(),
                    cpu_usage,
                    memory_usage,
                    memory_total,
                    memory_available,
                    disk_usage,
                    disk_total,
                    network_in,
                    network_out,
                    active_connections,
                    active_servers,
                    database_connections,
                    websocket_connections,
                    response_times: response_time_metrics,
                    error_rates: error_rate_metrics,
                };
                
                // Store metrics
                let mut metrics_guard = metrics.write().await;
                metrics_guard.push(metrics_entry.clone());
                if metrics_guard.len() > max_history {
                    metrics_guard.remove(0);
                }
                drop(metrics_guard);
                
                // Check thresholds and generate alerts
                Self::check_thresholds(&metrics_entry, &thresholds, &alerts).await;
                
                // Log performance info
                debug!("Performance metrics collected: CPU: {:.1}%, Memory: {:.1}%", 
                       cpu_usage, (memory_usage as f32 / memory_total as f32) * 100.0);
            }
        });
        
        Ok(())
    }
    
    /// Record a response time for a specific operation
    pub async fn record_response_time(&self, operation: &str, duration: Duration) {
        let mut response_times = self.response_times.write().await;
        let times = response_times.entry(operation.to_string()).or_insert_with(Vec::new);
        times.push(duration);
        
        // Keep only last 100 measurements per operation
        if times.len() > 100 {
            times.remove(0);
        }
    }
    
    /// Record an error for a specific operation
    pub async fn record_error(&self, operation: &str) {
        let mut error_counts = self.error_counts.write().await;
        *error_counts.entry(operation.to_string()).or_insert(0) += 1;
    }
    
    /// Get current performance metrics
    pub async fn get_current_metrics(&self) -> Option<PerformanceMetrics> {
        let metrics = self.metrics.read().await;
        metrics.last().cloned()
    }
    
    /// Get performance metrics history
    pub async fn get_metrics_history(&self, limit: Option<usize>) -> Vec<PerformanceMetrics> {
        let metrics = self.metrics.read().await;
        let limit = limit.unwrap_or(100);
        metrics.iter().rev().take(limit).cloned().collect()
    }
    
    /// Get performance alerts
    pub async fn get_alerts(&self, level: Option<AlertLevel>) -> Vec<PerformanceAlert> {
        let alerts = self.alerts.read().await;
        if let Some(level) = level {
            alerts.iter().filter(|alert| alert.level == level).cloned().collect()
        } else {
            alerts.clone()
        }
    }
    
    /// Clear old alerts
    pub async fn clear_old_alerts(&self, older_than: Duration) {
        let cutoff = chrono::Utc::now() - chrono::Duration::from_std(older_than).unwrap();
        let mut alerts = self.alerts.write().await;
        alerts.retain(|alert| alert.timestamp > cutoff);
    }
    
    /// Calculate response time statistics
    fn calculate_response_times(
        api_times: Vec<Duration>,
        database_times: Vec<Duration>,
        websocket_times: Vec<Duration>,
    ) -> ResponseTimeMetrics {
        ResponseTimeMetrics {
            api_avg: Self::calculate_average(&api_times),
            api_p95: Self::calculate_percentile(&api_times, 95),
            api_p99: Self::calculate_percentile(&api_times, 99),
            database_avg: Self::calculate_average(&database_times),
            database_p95: Self::calculate_percentile(&database_times, 95),
            database_p99: Self::calculate_percentile(&database_times, 99),
            websocket_avg: Self::calculate_average(&websocket_times),
            websocket_p95: Self::calculate_percentile(&websocket_times, 95),
            websocket_p99: Self::calculate_percentile(&websocket_times, 99),
        }
    }
    
    /// Calculate error rates
    fn calculate_error_rates(api_errors: u32, database_errors: u32, websocket_errors: u32) -> ErrorRateMetrics {
        let total_errors = api_errors + database_errors + websocket_errors;
        let total_requests = 1000; // Would need to track actual request count
        
        ErrorRateMetrics {
            api_errors_per_minute: api_errors as f32,
            database_errors_per_minute: database_errors as f32,
            websocket_errors_per_minute: websocket_errors as f32,
            total_errors_per_minute: total_errors as f32,
            error_rate_percentage: (total_errors as f32 / total_requests as f32) * 100.0,
        }
    }
    
    /// Calculate average duration
    fn calculate_average(times: &[Duration]) -> Duration {
        if times.is_empty() {
            return Duration::from_millis(0);
        }
        
        let total: Duration = times.iter().sum();
        total / times.len() as u32
    }
    
    /// Calculate percentile duration
    fn calculate_percentile(times: &[Duration], percentile: u8) -> Duration {
        if times.is_empty() {
            return Duration::from_millis(0);
        }
        
        let mut sorted_times = times.to_vec();
        sorted_times.sort();
        
        let index = ((percentile as f32 / 100.0) * (sorted_times.len() - 1) as f32) as usize;
        sorted_times[index.min(sorted_times.len() - 1)]
    }
    
    /// Check performance thresholds and generate alerts
    async fn check_thresholds(
        metrics: &PerformanceMetrics,
        thresholds: &PerformanceThresholds,
        alerts: &Arc<RwLock<Vec<PerformanceAlert>>>,
    ) {
        let mut new_alerts = Vec::new();
        
        // Check CPU usage
        if metrics.cpu_usage >= thresholds.cpu_usage_critical {
            new_alerts.push(PerformanceAlert {
                timestamp: chrono::Utc::now(),
                level: AlertLevel::Critical,
                metric: "cpu_usage".to_string(),
                value: metrics.cpu_usage as f64,
                threshold: thresholds.cpu_usage_critical as f64,
                message: format!("CPU usage is critically high: {:.1}%", metrics.cpu_usage),
                recommendations: vec![
                    "Consider scaling up resources".to_string(),
                    "Check for runaway processes".to_string(),
                    "Optimize CPU-intensive operations".to_string(),
                ],
            });
        } else if metrics.cpu_usage >= thresholds.cpu_usage_warning {
            new_alerts.push(PerformanceAlert {
                timestamp: chrono::Utc::now(),
                level: AlertLevel::Warning,
                metric: "cpu_usage".to_string(),
                value: metrics.cpu_usage as f64,
                threshold: thresholds.cpu_usage_warning as f64,
                message: format!("CPU usage is high: {:.1}%", metrics.cpu_usage),
                recommendations: vec![
                    "Monitor CPU usage closely".to_string(),
                    "Consider optimizing operations".to_string(),
                ],
            });
        }
        
        // Check memory usage
        let memory_usage_percent = (metrics.memory_usage as f32 / metrics.memory_total as f32) * 100.0;
        if memory_usage_percent >= thresholds.memory_usage_critical {
            new_alerts.push(PerformanceAlert {
                timestamp: chrono::Utc::now(),
                level: AlertLevel::Critical,
                metric: "memory_usage".to_string(),
                value: memory_usage_percent as f64,
                threshold: thresholds.memory_usage_critical as f64,
                message: format!("Memory usage is critically high: {:.1}%", memory_usage_percent),
                recommendations: vec![
                    "Consider scaling up memory".to_string(),
                    "Check for memory leaks".to_string(),
                    "Optimize memory usage".to_string(),
                ],
            });
        } else if memory_usage_percent >= thresholds.memory_usage_warning {
            new_alerts.push(PerformanceAlert {
                timestamp: chrono::Utc::now(),
                level: AlertLevel::Warning,
                metric: "memory_usage".to_string(),
                value: memory_usage_percent as f64,
                threshold: thresholds.memory_usage_warning as f64,
                message: format!("Memory usage is high: {:.1}%", memory_usage_percent),
                recommendations: vec![
                    "Monitor memory usage closely".to_string(),
                    "Consider memory optimization".to_string(),
                ],
            });
        }
        
        // Check response times
        if metrics.response_times.api_avg >= thresholds.response_time_critical {
            new_alerts.push(PerformanceAlert {
                timestamp: chrono::Utc::now(),
                level: AlertLevel::Critical,
                metric: "api_response_time".to_string(),
                value: metrics.response_times.api_avg.as_millis() as f64,
                threshold: thresholds.response_time_critical.as_millis() as f64,
                message: format!("API response time is critically slow: {:?}", metrics.response_times.api_avg),
                recommendations: vec![
                    "Check database performance".to_string(),
                    "Optimize API endpoints".to_string(),
                    "Consider caching".to_string(),
                ],
            });
        }
        
        // Check error rates
        if metrics.error_rates.error_rate_percentage >= thresholds.error_rate_critical {
            new_alerts.push(PerformanceAlert {
                timestamp: chrono::Utc::now(),
                level: AlertLevel::Critical,
                metric: "error_rate".to_string(),
                value: metrics.error_rates.error_rate_percentage as f64,
                threshold: thresholds.error_rate_critical as f64,
                message: format!("Error rate is critically high: {:.1}%", metrics.error_rates.error_rate_percentage),
                recommendations: vec![
                    "Check system logs for errors".to_string(),
                    "Verify external dependencies".to_string(),
                    "Review error handling".to_string(),
                ],
            });
        }
        
        // Add new alerts
        if !new_alerts.is_empty() {
            let mut alerts_guard = alerts.write().await;
            alerts_guard.extend(new_alerts);
        }
    }
}

/// Resource manager for optimizing system resources
pub struct ResourceManager {
    cpu_cores: usize,
    memory_limit: u64,
    disk_limit: u64,
    network_limit: u64,
    current_usage: Arc<RwLock<ResourceUsage>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_cores_used: usize,
    pub memory_used: u64,
    pub disk_used: u64,
    pub network_used: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ResourceManager {
    pub fn new(cpu_cores: usize, memory_limit: u64, disk_limit: u64, network_limit: u64) -> Self {
        Self {
            cpu_cores,
            memory_limit,
            disk_limit,
            network_limit,
            current_usage: Arc::new(RwLock::new(ResourceUsage {
                cpu_cores_used: 0,
                memory_used: 0,
                disk_used: 0,
                network_used: 0,
                timestamp: chrono::Utc::now(),
            })),
        }
    }
    
    /// Check if resources are available for a new server
    pub async fn can_allocate_server(&self, required_memory: u64, required_cpu: usize) -> bool {
        let usage = self.current_usage.read().await;
        
        let memory_available = self.memory_limit - usage.memory_used;
        let cpu_available = self.cpu_cores - usage.cpu_cores_used;
        
        memory_available >= required_memory && cpu_available >= required_cpu
    }
    
    /// Allocate resources for a server
    pub async fn allocate_server(&self, memory: u64, cpu_cores: usize) -> Result<(), String> {
        if !self.can_allocate_server(memory, cpu_cores).await {
            return Err("Insufficient resources available".to_string());
        }
        
        let mut usage = self.current_usage.write().await;
        usage.memory_used += memory;
        usage.cpu_cores_used += cpu_cores;
        usage.timestamp = chrono::Utc::now();
        
        Ok(())
    }
    
    /// Deallocate resources for a server
    pub async fn deallocate_server(&self, memory: u64, cpu_cores: usize) {
        let mut usage = self.current_usage.write().await;
        usage.memory_used = usage.memory_used.saturating_sub(memory);
        usage.cpu_cores_used = usage.cpu_cores_used.saturating_sub(cpu_cores);
        usage.timestamp = chrono::Utc::now();
    }
    
    /// Get current resource usage
    pub async fn get_usage(&self) -> ResourceUsage {
        self.current_usage.read().await.clone()
    }
    
    /// Get resource utilization percentages
    pub async fn get_utilization(&self) -> (f32, f32, f32, f32) {
        let usage = self.current_usage.read().await;
        
        let cpu_utilization = (usage.cpu_cores_used as f32 / self.cpu_cores as f32) * 100.0;
        let memory_utilization = (usage.memory_used as f32 / self.memory_limit as f32) * 100.0;
        let disk_utilization = (usage.disk_used as f32 / self.disk_limit as f32) * 100.0;
        let network_utilization = (usage.network_used as f32 / self.network_limit as f32) * 100.0;
        
        (cpu_utilization, memory_utilization, disk_utilization, network_utilization)
    }
}

/// Performance optimization strategies
pub struct PerformanceOptimizer {
    cache_strategies: HashMap<String, CacheStrategy>,
    connection_pooling: bool,
    compression_enabled: bool,
    rate_limiting: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStrategy {
    pub ttl: Duration,
    pub max_size: usize,
    pub eviction_policy: EvictionPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionPolicy {
    LRU,
    LFU,
    TTL,
}

impl Default for PerformanceOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceOptimizer {
    pub fn new() -> Self {
        Self {
            cache_strategies: HashMap::new(),
            connection_pooling: true,
            compression_enabled: true,
            rate_limiting: true,
        }
    }
    
    /// Add a cache strategy for a specific operation
    pub fn add_cache_strategy(&mut self, operation: &str, strategy: CacheStrategy) {
        self.cache_strategies.insert(operation.to_string(), strategy);
    }
    
    /// Optimize database queries
    pub fn optimize_database_queries(&self) -> Vec<String> {
        vec![
            "Add database indexes for frequently queried columns".to_string(),
            "Use prepared statements for repeated queries".to_string(),
            "Implement query result caching".to_string(),
            "Optimize JOIN operations".to_string(),
            "Use connection pooling".to_string(),
        ]
    }
    
    /// Optimize API endpoints
    pub fn optimize_api_endpoints(&self) -> Vec<String> {
        vec![
            "Implement response caching".to_string(),
            "Add compression for large responses".to_string(),
            "Use pagination for large datasets".to_string(),
            "Implement rate limiting".to_string(),
            "Add request/response validation".to_string(),
        ]
    }
    
    /// Optimize WebSocket connections
    pub fn optimize_websocket_connections(&self) -> Vec<String> {
        vec![
            "Implement connection pooling".to_string(),
            "Use binary frames for large data".to_string(),
            "Implement message batching".to_string(),
            "Add connection heartbeat".to_string(),
            "Implement graceful disconnection".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_performance_monitor() {
        let thresholds = PerformanceThresholds::default();
        let monitor = PerformanceMonitor::new(thresholds);
        
        // Test response time recording
        monitor.record_response_time("api", Duration::from_millis(100)).await;
        monitor.record_response_time("api", Duration::from_millis(200)).await;
        
        // Test error recording
        monitor.record_error("api").await;
        monitor.record_error("database").await;
        
        // Test metrics retrieval
        let metrics = monitor.get_current_metrics().await;
        assert!(metrics.is_some());
    }
    
    #[tokio::test]
    async fn test_resource_manager() {
        let manager = ResourceManager::new(4, 8192, 100000, 1000);
        
        // Test resource allocation
        assert!(manager.can_allocate_server(1024, 1).await);
        assert!(manager.allocate_server(1024, 1).await.is_ok());
        
        // Test resource deallocation
        manager.deallocate_server(1024, 1).await;
        
        // Test utilization calculation
        let (cpu, memory, disk, network) = manager.get_utilization().await;
        assert!(cpu >= 0.0 && cpu <= 100.0);
        assert!(memory >= 0.0 && memory <= 100.0);
    }
    
    #[test]
    fn test_performance_optimizer() {
        let optimizer = PerformanceOptimizer::new();
        
        let db_optimizations = optimizer.optimize_database_queries();
        assert!(!db_optimizations.is_empty());
        
        let api_optimizations = optimizer.optimize_api_endpoints();
        assert!(!api_optimizations.is_empty());
    }
}
