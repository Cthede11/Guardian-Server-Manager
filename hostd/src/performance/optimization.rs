use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

/// Database query optimization
pub struct QueryOptimizer {
    query_cache: Arc<RwLock<HashMap<String, CachedQuery>>>,
    cache_ttl: Duration,
}

#[derive(Debug, Clone)]
struct CachedQuery {
    data: serde_json::Value,
    timestamp: DateTime<Utc>,
    ttl: Duration,
}

impl QueryOptimizer {
    pub fn new(cache_ttl: Duration) -> Self {
        Self {
            query_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl,
        }
    }
    
    pub async fn get_cached_query(&self, query_key: &str) -> Option<serde_json::Value> {
        let cache = self.query_cache.read().await;
        if let Some(cached) = cache.get(query_key) {
            if Utc::now().signed_duration_since(cached.timestamp) < cached.ttl {
                return Some(cached.data.clone());
            }
        }
        None
    }
    
    pub async fn cache_query(&self, query_key: String, data: serde_json::Value) {
        let mut cache = self.query_cache.write().await;
        cache.insert(query_key, CachedQuery {
            data,
            timestamp: Utc::now(),
            ttl: self.cache_ttl,
        });
    }
    
    pub async fn invalidate_cache(&self, pattern: &str) {
        let mut cache = self.query_cache.write().await;
        cache.retain(|key, _| !key.contains(pattern));
    }
}

/// Memory optimization
pub struct MemoryOptimizer {
    memory_usage: Arc<RwLock<HashMap<String, MemoryStats>>>,
    max_memory_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub used_mb: u64,
    pub peak_mb: u64,
    pub timestamp: DateTime<Utc>,
}

impl MemoryOptimizer {
    pub fn new(max_memory_mb: u64) -> Self {
        Self {
            memory_usage: Arc::new(RwLock::new(HashMap::new())),
            max_memory_mb,
        }
    }
    
    pub async fn track_memory_usage(&self, component: &str, used_mb: u64) {
        let mut stats = self.memory_usage.write().await;
        let entry = stats.entry(component.to_string()).or_insert(MemoryStats {
            used_mb: 0,
            peak_mb: 0,
            timestamp: Utc::now(),
        });
        
        entry.used_mb = used_mb;
        if used_mb > entry.peak_mb {
            entry.peak_mb = used_mb;
        }
        entry.timestamp = Utc::now();
    }
    
    pub async fn get_memory_usage(&self) -> HashMap<String, MemoryStats> {
        self.memory_usage.read().await.clone()
    }
    
    pub async fn should_gc(&self) -> bool {
        let stats = self.memory_usage.read().await;
        let total_used: u64 = stats.values().map(|s| s.used_mb).sum();
        total_used > self.max_memory_mb
    }
}

/// Connection pooling
pub struct ConnectionPool {
    max_connections: usize,
    active_connections: Arc<RwLock<usize>>,
    connection_queue: Arc<RwLock<Vec<tokio::sync::oneshot::Sender<()>>>>,
}

impl ConnectionPool {
    pub fn new(max_connections: usize) -> Self {
        Self {
            max_connections,
            active_connections: Arc::new(RwLock::new(0)),
            connection_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub async fn acquire_connection(&self) -> ConnectionGuard {
        let mut active = self.active_connections.write().await;
        
        if *active < self.max_connections {
            *active += 1;
            ConnectionGuard {
                pool: self.clone(),
            }
        } else {
            // Wait for connection to be available
            let (tx, rx) = tokio::sync::oneshot::channel();
            {
                let mut queue = self.connection_queue.write().await;
                queue.push(tx);
            }
            
            let _ = rx.await;
            self.acquire_connection().await
        }
    }
    
    async fn release_connection(&self) {
        let mut active = self.active_connections.write().await;
        *active = active.saturating_sub(1);
        
        // Notify waiting connections
        let mut queue = self.connection_queue.write().await;
        if let Some(tx) = queue.pop() {
            let _ = tx.send(());
        }
    }
}

#[derive(Clone)]
pub struct ConnectionPool {
    max_connections: usize,
    active_connections: Arc<RwLock<usize>>,
    connection_queue: Arc<RwLock<Vec<tokio::sync::oneshot::Sender<()>>>>,
}

pub struct ConnectionGuard {
    pool: ConnectionPool,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        let pool = self.pool.clone();
        tokio::spawn(async move {
            pool.release_connection().await;
        });
    }
}

/// Async task optimization
pub struct TaskOptimizer {
    task_pool: tokio::task::JoinSet<()>,
    max_concurrent_tasks: usize,
    active_tasks: Arc<RwLock<usize>>,
}

impl TaskOptimizer {
    pub fn new(max_concurrent_tasks: usize) -> Self {
        Self {
            task_pool: tokio::task::JoinSet::new(),
            max_concurrent_tasks,
            active_tasks: Arc::new(RwLock::new(0)),
        }
    }
    
    pub async fn spawn_task<F>(&mut self, task: F) -> Result<(), TaskError>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let active = *self.active_tasks.read().await;
        
        if active >= self.max_concurrent_tasks {
            return Err(TaskError::TooManyTasks);
        }
        
        {
            let mut active_tasks = self.active_tasks.write().await;
            *active_tasks += 1;
        }
        
        let active_tasks = self.active_tasks.clone();
        self.task_pool.spawn(async move {
            task.await;
            let mut active = active_tasks.write().await;
            *active = active.saturating_sub(1);
        });
        
        Ok(())
    }
    
    pub async fn wait_for_completion(&mut self) {
        while let Some(_) = self.task_pool.join_next().await {}
    }
}

use std::future::Future;

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("Too many concurrent tasks")]
    TooManyTasks,
}

/// Performance monitoring
pub struct PerformanceMonitor {
    metrics: Arc<RwLock<HashMap<String, PerformanceMetric>>>,
    alerts: Arc<RwLock<Vec<PerformanceAlert>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
    pub threshold: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub metric_name: String,
    pub current_value: f64,
    pub threshold: f64,
    pub severity: AlertSeverity,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub async fn record_metric(&self, name: String, value: f64, unit: String, threshold: Option<f64>) {
        let metric = PerformanceMetric {
            name: name.clone(),
            value,
            unit,
            timestamp: Utc::now(),
            threshold,
        };
        
        {
            let mut metrics = self.metrics.write().await;
            metrics.insert(name.clone(), metric);
        }
        
        // Check for threshold violations
        if let Some(threshold) = threshold {
            if value > threshold {
                let alert = PerformanceAlert {
                    metric_name: name,
                    current_value: value,
                    threshold,
                    severity: self.determine_severity(value, threshold),
                    timestamp: Utc::now(),
                };
                
                let mut alerts = self.alerts.write().await;
                alerts.push(alert);
            }
        }
    }
    
    fn determine_severity(&self, value: f64, threshold: f64) -> AlertSeverity {
        let ratio = value / threshold;
        match ratio {
            r if r >= 2.0 => AlertSeverity::Critical,
            r if r >= 1.5 => AlertSeverity::High,
            r if r >= 1.2 => AlertSeverity::Medium,
            _ => AlertSeverity::Low,
        }
    }
    
    pub async fn get_metrics(&self) -> HashMap<String, PerformanceMetric> {
        self.metrics.read().await.clone()
    }
    
    pub async fn get_alerts(&self) -> Vec<PerformanceAlert> {
        self.alerts.read().await.clone()
    }
}
