use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramVec, 
    Registry, TextEncoder, Encoder, Opts, HistogramOpts
};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use anyhow::Result;

/// Metrics collector for Guardian Server Manager
pub struct MetricsCollector {
    registry: Registry,
    
    // HTTP metrics
    http_requests_total: CounterVec,
    http_request_duration: HistogramVec,
    http_requests_in_flight: GaugeVec,
    
    // Server metrics
    servers_total: Gauge,
    servers_running: Gauge,
    servers_stopped: Gauge,
    server_memory_usage: GaugeVec,
    server_cpu_usage: GaugeVec,
    server_tps: GaugeVec,
    server_players_online: GaugeVec,
    
    // Database metrics
    database_connections: Gauge,
    database_queries_total: CounterVec,
    database_query_duration: HistogramVec,
    database_errors_total: CounterVec,
    
    // System metrics
    system_memory_usage: Gauge,
    system_cpu_usage: Gauge,
    system_disk_usage: Gauge,
    system_uptime: Gauge,
    
    // Business metrics
    active_users: Gauge,
    api_calls_total: CounterVec,
    error_rate: Gauge,
    
    // Custom metrics storage
    custom_metrics: Arc<RwLock<HashMap<String, CustomMetric>>>,
}

/// Custom metric types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomMetric {
    Counter { value: f64, labels: HashMap<String, String> },
    Gauge { value: f64, labels: HashMap<String, String> },
    Histogram { values: Vec<f64>, labels: HashMap<String, String> },
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();
        
        // HTTP metrics
        let http_requests_total = CounterVec::new(
            Opts::new("http_requests_total", "Total number of HTTP requests"),
            &["method", "endpoint", "status_code"]
        )?;
        
        let http_request_duration = HistogramVec::new(
            HistogramOpts::new("http_request_duration_seconds", "HTTP request duration in seconds"),
            &["method", "endpoint"]
        )?;
        
        let http_requests_in_flight = GaugeVec::new(
            Opts::new("http_requests_in_flight", "Number of HTTP requests currently in flight"),
            &["method", "endpoint"]
        )?;
        
        // Server metrics
        let servers_total = Gauge::new("servers_total", "Total number of servers")?;
        let servers_running = Gauge::new("servers_running", "Number of running servers")?;
        let servers_stopped = Gauge::new("servers_stopped", "Number of stopped servers")?;
        
        let server_memory_usage = GaugeVec::new(
            Opts::new("server_memory_usage_bytes", "Server memory usage in bytes"),
            &["server_id", "server_name"]
        )?;
        
        let server_cpu_usage = GaugeVec::new(
            Opts::new("server_cpu_usage_percent", "Server CPU usage percentage"),
            &["server_id", "server_name"]
        )?;
        
        let server_tps = GaugeVec::new(
            Opts::new("server_tps", "Server ticks per second"),
            &["server_id", "server_name"]
        )?;
        
        let server_players_online = GaugeVec::new(
            Opts::new("server_players_online", "Number of players online"),
            &["server_id", "server_name"]
        )?;
        
        // Database metrics
        let database_connections = Gauge::new("database_connections", "Number of database connections")?;
        
        let database_queries_total = CounterVec::new(
            Opts::new("database_queries_total", "Total number of database queries"),
            &["query_type", "table"]
        )?;
        
        let database_query_duration = HistogramVec::new(
            HistogramOpts::new("database_query_duration_seconds", "Database query duration in seconds"),
            &["query_type", "table"]
        )?;
        
        let database_errors_total = CounterVec::new(
            Opts::new("database_errors_total", "Total number of database errors"),
            &["error_type", "table"]
        )?;
        
        // System metrics
        let system_memory_usage = Gauge::new("system_memory_usage_bytes", "System memory usage in bytes")?;
        let system_cpu_usage = Gauge::new("system_cpu_usage_percent", "System CPU usage percentage")?;
        let system_disk_usage = Gauge::new("system_disk_usage_bytes", "System disk usage in bytes")?;
        let system_uptime = Gauge::new("system_uptime_seconds", "System uptime in seconds")?;
        
        // Business metrics
        let active_users = Gauge::new("active_users", "Number of active users")?;
        
        let api_calls_total = CounterVec::new(
            Opts::new("api_calls_total", "Total number of API calls"),
            &["endpoint", "method", "status"]
        )?;
        
        let error_rate = Gauge::new("error_rate_percent", "Error rate percentage")?;
        
        // Register all metrics
        registry.register(Box::new(http_requests_total.clone()))?;
        registry.register(Box::new(http_request_duration.clone()))?;
        registry.register(Box::new(http_requests_in_flight.clone()))?;
        registry.register(Box::new(servers_total.clone()))?;
        registry.register(Box::new(servers_running.clone()))?;
        registry.register(Box::new(servers_stopped.clone()))?;
        registry.register(Box::new(server_memory_usage.clone()))?;
        registry.register(Box::new(server_cpu_usage.clone()))?;
        registry.register(Box::new(server_tps.clone()))?;
        registry.register(Box::new(server_players_online.clone()))?;
        registry.register(Box::new(database_connections.clone()))?;
        registry.register(Box::new(database_queries_total.clone()))?;
        registry.register(Box::new(database_query_duration.clone()))?;
        registry.register(Box::new(database_errors_total.clone()))?;
        registry.register(Box::new(system_memory_usage.clone()))?;
        registry.register(Box::new(system_cpu_usage.clone()))?;
        registry.register(Box::new(system_disk_usage.clone()))?;
        registry.register(Box::new(system_uptime.clone()))?;
        registry.register(Box::new(active_users.clone()))?;
        registry.register(Box::new(api_calls_total.clone()))?;
        registry.register(Box::new(error_rate.clone()))?;
        
        Ok(Self {
            registry,
            http_requests_total,
            http_request_duration,
            http_requests_in_flight,
            servers_total,
            servers_running,
            servers_stopped,
            server_memory_usage,
            server_cpu_usage,
            server_tps,
            server_players_online,
            database_connections,
            database_queries_total,
            database_query_duration,
            database_errors_total,
            system_memory_usage,
            system_cpu_usage,
            system_disk_usage,
            system_uptime,
            active_users,
            api_calls_total,
            error_rate,
            custom_metrics: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Get the Prometheus registry
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
    
    /// Record HTTP request metrics
    pub fn record_http_request(&self, method: &str, endpoint: &str, status_code: u16, duration: Duration) {
        let status = status_code.to_string();
        self.http_requests_total.with_label_values(&[method, endpoint, &status]).inc();
        self.http_request_duration.with_label_values(&[method, endpoint])
            .observe(duration.as_secs_f64());
    }
    
    /// Increment HTTP requests in flight
    pub fn increment_http_requests_in_flight(&self, method: &str, endpoint: &str) {
        self.http_requests_in_flight.with_label_values(&[method, endpoint]).inc();
    }
    
    /// Decrement HTTP requests in flight
    pub fn decrement_http_requests_in_flight(&self, method: &str, endpoint: &str) {
        self.http_requests_in_flight.with_label_values(&[method, endpoint]).dec();
    }
    
    /// Update server metrics
    pub fn update_server_metrics(&self, server_id: &str, server_name: &str, metrics: &ServerMetrics) {
        self.server_memory_usage.with_label_values(&[server_id, server_name])
            .set(metrics.memory_usage_bytes as f64);
        self.server_cpu_usage.with_label_values(&[server_id, server_name])
            .set(metrics.cpu_usage_percent);
        self.server_tps.with_label_values(&[server_id, server_name])
            .set(metrics.tps);
        self.server_players_online.with_label_values(&[server_id, server_name])
            .set(metrics.players_online as f64);
    }
    
    /// Update server count metrics
    pub fn update_server_counts(&self, total: u32, running: u32, stopped: u32) {
        self.servers_total.set(total as f64);
        self.servers_running.set(running as f64);
        self.servers_stopped.set(stopped as f64);
    }
    
    /// Record database query metrics
    pub fn record_database_query(&self, query_type: &str, table: &str, duration: Duration) {
        self.database_queries_total.with_label_values(&[query_type, table]).inc();
        self.database_query_duration.with_label_values(&[query_type, table])
            .observe(duration.as_secs_f64());
    }
    
    /// Record database error
    pub fn record_database_error(&self, error_type: &str, table: &str) {
        self.database_errors_total.with_label_values(&[error_type, table]).inc();
    }
    
    /// Update database connection count
    pub fn update_database_connections(&self, count: u32) {
        self.database_connections.set(count as f64);
    }
    
    /// Update system metrics
    pub fn update_system_metrics(&self, metrics: &SystemMetrics) {
        self.system_memory_usage.set(metrics.memory_usage_bytes as f64);
        self.system_cpu_usage.set(metrics.cpu_usage_percent);
        self.system_disk_usage.set(metrics.disk_usage_bytes as f64);
        self.system_uptime.set(metrics.uptime_seconds as f64);
    }
    
    /// Update business metrics
    pub fn update_business_metrics(&self, active_users: u32, error_rate: f64) {
        self.active_users.set(active_users as f64);
        self.error_rate.set(error_rate);
    }
    
    /// Record API call
    pub fn record_api_call(&self, endpoint: &str, method: &str, status: &str) {
        self.api_calls_total.with_label_values(&[endpoint, method, status]).inc();
    }
    
    /// Add custom metric
    pub async fn add_custom_metric(&self, name: String, metric: CustomMetric) {
        let mut custom_metrics = self.custom_metrics.write().await;
        custom_metrics.insert(name, metric);
    }
    
    /// Get custom metric
    pub async fn get_custom_metric(&self, name: &str) -> Option<CustomMetric> {
        let custom_metrics = self.custom_metrics.read().await;
        custom_metrics.get(name).cloned()
    }
    
    /// Export metrics in Prometheus format
    pub fn export_metrics(&self) -> Result<String, prometheus::Error> {
        let metric_families = self.registry.gather();
        let encoder = TextEncoder::new();
        encoder.encode_to_string(&metric_families)
    }
    
    /// Get metrics summary
    pub async fn get_metrics_summary(&self) -> MetricsSummary {
        let custom_metrics = self.custom_metrics.read().await;
        
        MetricsSummary {
            http_requests_total: self.http_requests_total.get_metric_with_label_values(&[]).map(|m| m.get()).unwrap_or(0.0),
            servers_total: self.servers_total.get(),
            servers_running: self.servers_running.get(),
            servers_stopped: self.servers_stopped.get(),
            database_connections: self.database_connections.get(),
            system_memory_usage: self.system_memory_usage.get(),
            system_cpu_usage: self.system_cpu_usage.get(),
            active_users: self.active_users.get(),
            error_rate: self.error_rate.get(),
            custom_metrics_count: custom_metrics.len(),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Server metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetrics {
    pub memory_usage_bytes: u64,
    pub cpu_usage_percent: f64,
    pub tps: f64,
    pub players_online: u32,
    pub uptime_seconds: u64,
}

/// System metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub memory_usage_bytes: u64,
    pub cpu_usage_percent: f64,
    pub disk_usage_bytes: u64,
    pub uptime_seconds: u64,
}

/// Metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub http_requests_total: f64,
    pub servers_total: f64,
    pub servers_running: f64,
    pub servers_stopped: f64,
    pub database_connections: f64,
    pub system_memory_usage: f64,
    pub system_cpu_usage: f64,
    pub active_users: f64,
    pub error_rate: f64,
    pub custom_metrics_count: usize,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Metrics middleware for HTTP requests
pub struct MetricsMiddleware {
    metrics: Arc<MetricsCollector>,
}

impl MetricsMiddleware {
    pub fn new(metrics: Arc<MetricsCollector>) -> Self {
        Self { metrics }
    }
    
    pub async fn record_request<F, R>(&self, method: &str, endpoint: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start_time = Instant::now();
        self.metrics.increment_http_requests_in_flight(method, endpoint);
        
        let result = f();
        
        let duration = start_time.elapsed();
        self.metrics.decrement_http_requests_in_flight(method, endpoint);
        self.metrics.record_http_request(method, endpoint, 200, duration);
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[test]
    fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new().unwrap();
        assert!(collector.registry().gather().len() > 0);
    }
    
    #[test]
    fn test_http_metrics_recording() {
        let collector = MetricsCollector::new().unwrap();
        
        collector.record_http_request("GET", "/api/servers", 200, Duration::from_millis(100));
        collector.record_http_request("POST", "/api/servers", 201, Duration::from_millis(200));
        
        let metrics = collector.export_metrics().unwrap();
        assert!(metrics.contains("http_requests_total"));
        assert!(metrics.contains("http_request_duration_seconds"));
    }
    
    #[test]
    fn test_server_metrics_update() {
        let collector = MetricsCollector::new().unwrap();
        
        let server_metrics = ServerMetrics {
            memory_usage_bytes: 1024 * 1024 * 1024, // 1GB
            cpu_usage_percent: 50.0,
            tps: 20.0,
            players_online: 5,
            uptime_seconds: 3600,
        };
        
        collector.update_server_metrics("server-1", "Test Server", &server_metrics);
        
        let metrics = collector.export_metrics().unwrap();
        assert!(metrics.contains("server_memory_usage_bytes"));
        assert!(metrics.contains("server_cpu_usage_percent"));
        assert!(metrics.contains("server_tps"));
        assert!(metrics.contains("server_players_online"));
    }
    
    #[tokio::test]
    async fn test_custom_metrics() {
        let collector = MetricsCollector::new().unwrap();
        
        let custom_metric = CustomMetric::Gauge {
            value: 42.0,
            labels: HashMap::new(),
        };
        
        collector.add_custom_metric("test_metric".to_string(), custom_metric).await;
        
        let retrieved = collector.get_custom_metric("test_metric").await;
        assert!(retrieved.is_some());
        
        if let Some(CustomMetric::Gauge { value, .. }) = retrieved {
            assert_eq!(value, 42.0);
        }
    }
    
    #[tokio::test]
    async fn test_metrics_summary() {
        let collector = MetricsCollector::new().unwrap();
        
        collector.update_server_counts(10, 5, 5);
        collector.update_business_metrics(100, 2.5);
        
        let summary = collector.get_metrics_summary().await;
        assert_eq!(summary.servers_total, 10.0);
        assert_eq!(summary.servers_running, 5.0);
        assert_eq!(summary.servers_stopped, 5.0);
        assert_eq!(summary.active_users, 100.0);
        assert_eq!(summary.error_rate, 2.5);
    }
}
