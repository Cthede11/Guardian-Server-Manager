use crate::error::{GuardianError, utils as error_utils};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::{interval, sleep};
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// Health check status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub check_id: String,
    pub component: String,
    pub status: HealthStatus,
    pub message: String,
    pub timestamp: u64,
    pub duration_ms: u64,
    pub metadata: HashMap<String, String>,
    pub error: Option<String>,
}

/// Health check definition
#[derive(Clone)]
pub struct HealthCheck {
    pub id: String,
    pub name: String,
    pub component: String,
    pub check_fn: Arc<dyn Fn() -> Result<HealthCheckResult, GuardianError> + Send + Sync>,
    pub interval: Duration,
    pub timeout: Duration,
    pub critical: bool,
    pub enabled: bool,
}

/// Health monitor for the Guardian Platform
pub struct HealthMonitor {
    checks: Arc<RwLock<HashMap<String, HealthCheck>>>,
    results: Arc<RwLock<HashMap<String, HealthCheckResult>>>,
    last_check_times: Arc<RwLock<HashMap<String, Instant>>>,
    is_running: Arc<RwLock<bool>>,
    check_interval: Duration,
}

impl HealthMonitor {
    pub fn new(check_interval: Duration) -> Self {
        Self {
            checks: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
            last_check_times: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
            check_interval,
        }
    }

    /// Add a health check
    pub async fn add_check(&self, check: HealthCheck) {
        let name = check.name.clone();
        let mut checks = self.checks.write().await;
        checks.insert(check.id.clone(), check);
        info!("Added health check: {}", name);
    }

    /// Remove a health check
    pub async fn remove_check(&self, check_id: &str) {
        let mut checks = self.checks.write().await;
        if let Some(check) = checks.remove(check_id) {
            info!("Removed health check: {}", check.name);
        }
    }

    /// Start the health monitor
    pub async fn start(&self) -> Result<(), GuardianError> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(error_utils::internal_error(
                "health_monitor",
                "start",
                "Health monitor is already running",
            ));
        }
        *is_running = true;
        drop(is_running);

        info!("Starting health monitor with interval: {:?}", self.check_interval);

        let checks = self.checks.clone();
        let results = self.results.clone();
        let last_check_times = self.last_check_times.clone();
        let is_running = self.is_running.clone();

        let check_interval = self.check_interval;
        tokio::spawn(async move {
            let mut interval_timer = interval(check_interval);
            
            while *is_running.read().await {
                interval_timer.tick().await;
                
                let checks_to_run = {
                    let checks = checks.read().await;
                    checks.values()
                        .filter(|check| check.enabled)
                        .cloned()
                        .collect::<Vec<_>>()
                };

                for check in checks_to_run {
                    let should_run = {
                        let last_times = last_check_times.read().await;
                        let last_time = last_times.get(&check.id);
                        last_time.map_or(true, |time| time.elapsed() >= check.interval)
                    };

                    if should_run {
                        let result = Self::run_check(&check).await;
                        
                        // Update results
                        {
                            let mut results = results.write().await;
                            results.insert(check.id.clone(), result.clone());
                        }
                        
                        // Update last check time
                        {
                            let mut last_times = last_check_times.write().await;
                            last_times.insert(check.id.clone(), Instant::now());
                        }

                        // Log critical failures
                        if check.critical && result.status == HealthStatus::Unhealthy {
                            error!("Critical health check failed: {} - {}", check.name, result.message);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the health monitor
    pub async fn stop(&self) {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        info!("Health monitor stopped");
    }

    /// Run a single health check
    async fn run_check(check: &HealthCheck) -> HealthCheckResult {
        let start_time = Instant::now();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let result = tokio::time::timeout(check.timeout, async {
            (check.check_fn)()
        }).await;

        let duration_ms = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(health_result)) => health_result,
            Ok(Err(error)) => HealthCheckResult {
                check_id: check.id.clone(),
                component: check.component.clone(),
                status: HealthStatus::Unhealthy,
                message: format!("Health check failed: {}", error),
                timestamp,
                duration_ms,
                metadata: HashMap::new(),
                error: Some(error.to_string()),
            },
            Err(_) => HealthCheckResult {
                check_id: check.id.clone(),
                component: check.component.clone(),
                status: HealthStatus::Unhealthy,
                message: "Health check timed out".to_string(),
                timestamp,
                duration_ms,
                metadata: HashMap::new(),
                error: Some("timeout".to_string()),
            },
        }
    }

    /// Get overall system health
    pub async fn get_system_health(&self) -> SystemHealth {
        let results = self.results.read().await;
        let checks = self.checks.read().await;

        let mut healthy_count = 0;
        let mut degraded_count = 0;
        let mut unhealthy_count = 0;
        let mut unknown_count = 0;
        let mut critical_failures = 0;

        for (check_id, result) in results.iter() {
            match result.status {
                HealthStatus::Healthy => healthy_count += 1,
                HealthStatus::Degraded => degraded_count += 1,
                HealthStatus::Unhealthy => {
                    unhealthy_count += 1;
                    if let Some(check) = checks.get(check_id) {
                        if check.critical {
                            critical_failures += 1;
                        }
                    }
                }
                HealthStatus::Unknown => unknown_count += 1,
            }
        }

        let total_checks = results.len();
        let overall_status = if critical_failures > 0 {
            HealthStatus::Unhealthy
        } else if unhealthy_count > 0 {
            HealthStatus::Degraded
        } else if degraded_count > 0 {
            HealthStatus::Degraded
        } else if total_checks == 0 {
            HealthStatus::Unknown
        } else {
            HealthStatus::Healthy
        };

        SystemHealth {
            status: overall_status,
            total_checks,
            healthy_count,
            degraded_count,
            unhealthy_count,
            unknown_count,
            critical_failures,
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            checks: results.values().cloned().collect(),
        }
    }

    /// Get health check results for a specific component
    pub async fn get_component_health(&self, component: &str) -> Vec<HealthCheckResult> {
        let results = self.results.read().await;
        results.values()
            .filter(|result| result.component == component)
            .cloned()
            .collect()
    }

    /// Get health check result by ID
    pub async fn get_check_result(&self, check_id: &str) -> Option<HealthCheckResult> {
        let results = self.results.read().await;
        results.get(check_id).cloned()
    }

    /// Force run a specific health check
    pub async fn run_check_now(&self, check_id: &str) -> Result<HealthCheckResult, GuardianError> {
        let check = {
            let checks = self.checks.read().await;
            checks.get(check_id).cloned()
        };

        match check {
            Some(check) => {
                let result = Self::run_check(&check).await;
                
                // Update results
                {
                    let mut results = self.results.write().await;
                    results.insert(check_id.to_string(), result.clone());
                }
                
                // Update last check time
                {
                    let mut last_times = self.last_check_times.write().await;
                    last_times.insert(check_id.to_string(), Instant::now());
                }

                Ok(result)
            }
            None => Err(error_utils::resource_error(
                crate::error::ResourceErrorKind::NotFound,
                "health_check",
                check_id,
                "Health check not found",
            ))
        }
    }
}

/// System health overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub status: HealthStatus,
    pub total_checks: usize,
    pub healthy_count: usize,
    pub degraded_count: usize,
    pub unhealthy_count: usize,
    pub unknown_count: usize,
    pub critical_failures: usize,
    pub last_updated: u64,
    pub checks: Vec<HealthCheckResult>,
}

/// Built-in health checks
pub mod builtin {
    use super::*;
    use std::process::Command;
    use std::net::{TcpStream, SocketAddr};
    use std::time::Duration;

    /// CPU usage health check
    pub fn cpu_usage_check() -> HealthCheck {
        HealthCheck {
            id: "cpu_usage".to_string(),
            name: "CPU Usage".to_string(),
            component: "system".to_string(),
            check_fn: Arc::new(|| {
                let usage = get_cpu_usage()?;
                let status = if usage > 90.0 {
                    HealthStatus::Unhealthy
                } else if usage > 80.0 {
                    HealthStatus::Degraded
                } else {
                    HealthStatus::Healthy
                };

                let mut metadata = HashMap::new();
                metadata.insert("cpu_usage_percent".to_string(), usage.to_string());

                Ok(HealthCheckResult {
                    check_id: "cpu_usage".to_string(),
                    component: "system".to_string(),
                    status,
                    message: format!("CPU usage: {:.1}%", usage),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    duration_ms: 0,
                    metadata,
                    error: None,
                })
            }),
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            critical: false,
            enabled: true,
        }
    }

    /// Memory usage health check
    pub fn memory_usage_check() -> HealthCheck {
        HealthCheck {
            id: "memory_usage".to_string(),
            name: "Memory Usage".to_string(),
            component: "system".to_string(),
            check_fn: Arc::new(|| {
                let usage = get_memory_usage()?;
                let status = if usage > 90.0 {
                    HealthStatus::Unhealthy
                } else if usage > 80.0 {
                    HealthStatus::Degraded
                } else {
                    HealthStatus::Healthy
                };

                let mut metadata = HashMap::new();
                metadata.insert("memory_usage_percent".to_string(), usage.to_string());

                Ok(HealthCheckResult {
                    check_id: "memory_usage".to_string(),
                    component: "system".to_string(),
                    status,
                    message: format!("Memory usage: {:.1}%", usage),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    duration_ms: 0,
                    metadata,
                    error: None,
                })
            }),
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            critical: false,
            enabled: true,
        }
    }

    /// Disk usage health check
    pub fn disk_usage_check() -> HealthCheck {
        HealthCheck {
            id: "disk_usage".to_string(),
            name: "Disk Usage".to_string(),
            component: "system".to_string(),
            check_fn: Arc::new(|| {
                let usage = get_disk_usage()?;
                let status = if usage > 95.0 {
                    HealthStatus::Unhealthy
                } else if usage > 85.0 {
                    HealthStatus::Degraded
                } else {
                    HealthStatus::Healthy
                };

                let mut metadata = HashMap::new();
                metadata.insert("disk_usage_percent".to_string(), usage.to_string());

                Ok(HealthCheckResult {
                    check_id: "disk_usage".to_string(),
                    component: "system".to_string(),
                    status,
                    message: format!("Disk usage: {:.1}%", usage),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    duration_ms: 0,
                    metadata,
                    error: None,
                })
            }),
            interval: Duration::from_secs(60),
            timeout: Duration::from_secs(10),
            critical: true,
            enabled: true,
        }
    }

    /// Network connectivity health check
    pub fn network_connectivity_check(endpoint: String) -> HealthCheck {
        let check_id = format!("network_{}", endpoint.replace(":", "_").replace("/", "_"));
        HealthCheck {
            id: check_id.clone(),
            name: format!("Network Connectivity: {}", endpoint),
            component: "network".to_string(),
            check_fn: Arc::new(move || {
                let start_time = Instant::now();
                let result = check_network_connectivity(&endpoint);
                let duration_ms = start_time.elapsed().as_millis() as u64;

                let (status, message) = match result {
                    Ok(_) => (HealthStatus::Healthy, "Network connectivity OK".to_string()),
                    Err(e) => (HealthStatus::Unhealthy, format!("Network connectivity failed: {}", e)),
                };

                let mut metadata = HashMap::new();
                metadata.insert("endpoint".to_string(), endpoint.clone());
                metadata.insert("response_time_ms".to_string(), duration_ms.to_string());

                Ok(HealthCheckResult {
                    check_id: check_id.clone(),
                    component: "network".to_string(),
                    status,
                    message,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    duration_ms,
                    metadata,
                    error: None,
                })
            }),
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(10),
            critical: false,
            enabled: true,
        }
    }

    /// Process health check
    pub fn process_health_check(process_name: String) -> HealthCheck {
        let check_id = format!("process_{}", process_name);
        HealthCheck {
            id: check_id.clone(),
            name: format!("Process: {}", process_name),
            component: "process".to_string(),
            check_fn: Arc::new(move || {
                let is_running = is_process_running(&process_name)?;
                let status = if is_running {
                    HealthStatus::Healthy
                } else {
                    HealthStatus::Unhealthy
                };

                let mut metadata = HashMap::new();
                metadata.insert("process_name".to_string(), process_name.clone());
                metadata.insert("is_running".to_string(), is_running.to_string());

                Ok(HealthCheckResult {
                    check_id: check_id.clone(),
                    component: "process".to_string(),
                    status,
                    message: if is_running {
                        "Process is running".to_string()
                    } else {
                        "Process is not running".to_string()
                    },
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    duration_ms: 0,
                    metadata,
                    error: None,
                })
            }),
            interval: Duration::from_secs(15),
            timeout: Duration::from_secs(5),
            critical: true,
            enabled: true,
        }
    }

    /// Database connectivity health check
    pub fn database_health_check(connection_string: String) -> HealthCheck {
        let check_id = "database_connectivity".to_string();
        HealthCheck {
            id: check_id.clone(),
            name: "Database Connectivity".to_string(),
            component: "database".to_string(),
            check_fn: Arc::new(move || {
                let start_time = Instant::now();
                let result = check_database_connectivity(&connection_string);
                let duration_ms = start_time.elapsed().as_millis() as u64;

                let (status, message) = match result {
                    Ok(_) => (HealthStatus::Healthy, "Database connection OK".to_string()),
                    Err(e) => (HealthStatus::Unhealthy, format!("Database connection failed: {}", e)),
                };

                let mut metadata = HashMap::new();
                metadata.insert("response_time_ms".to_string(), duration_ms.to_string());

                Ok(HealthCheckResult {
                    check_id: check_id.clone(),
                    component: "database".to_string(),
                    status,
                    message,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    duration_ms,
                    metadata,
                    error: None,
                })
            }),
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(10),
            critical: true,
            enabled: true,
        }
    }

    // Helper functions for system metrics
    fn get_cpu_usage() -> Result<f64, GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would use system-specific APIs
        Ok(25.0) // Placeholder
    }

    fn get_memory_usage() -> Result<f64, GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would use system-specific APIs
        Ok(45.0) // Placeholder
    }

    fn get_disk_usage() -> Result<f64, GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would use system-specific APIs
        Ok(60.0) // Placeholder
    }

    fn check_network_connectivity(endpoint: &str) -> Result<(), GuardianError> {
        // Parse endpoint (assuming format like "host:port")
        let parts: Vec<&str> = endpoint.split(':').collect();
        if parts.len() != 2 {
            return Err(error_utils::validation_error(
                "endpoint",
                endpoint,
                "format",
                "Endpoint must be in format 'host:port'",
            ));
        }

        let host = parts[0];
        let port: u16 = parts[1].parse()
            .map_err(|_| error_utils::validation_error(
                "port",
                parts[1],
                "numeric",
                "Port must be a valid number",
            ))?;

        let addr: SocketAddr = format!("{}:{}", host, port).parse()
            .map_err(|_| error_utils::network_error(
                endpoint,
                "Invalid address format",
                None,
            ))?;

        TcpStream::connect_timeout(&addr, Duration::from_secs(5))
            .map_err(|e| error_utils::network_error(
                endpoint,
                &format!("Connection failed: {}", e),
                None,
            ))?;

        Ok(())
    }

    fn is_process_running(process_name: &str) -> Result<bool, GuardianError> {
        // Use Windows-compatible process checking
        if cfg!(target_os = "windows") {
            let output = Command::new("tasklist")
                .arg("/FI")
                .arg(format!("IMAGENAME eq {}.exe", process_name))
                .output()
                .map_err(|e| error_utils::internal_error(
                    "process_check",
                    "tasklist",
                    &format!("Failed to check process: {}", e),
                ))?;

            let output_str = String::from_utf8_lossy(&output.stdout);
            Ok(output_str.contains(&format!("{}.exe", process_name)))
        } else {
            let output = Command::new("pgrep")
                .arg("-f")
                .arg(process_name)
                .output()
                .map_err(|e| error_utils::internal_error(
                    "process_check",
                    "pgrep",
                    &format!("Failed to check process: {}", e),
                ))?;

            Ok(output.status.success())
        }
    }

    fn check_database_connectivity(connection_string: &str) -> Result<(), GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would use the appropriate database client
        if connection_string.is_empty() {
            return Err(error_utils::validation_error(
                "connection_string",
                "",
                "non_empty",
                "Connection string cannot be empty",
            ));
        }

        // Simulate database check
        Ok(())
    }
}

/// Health check registry for managing all health checks
pub struct HealthCheckRegistry {
    monitor: Arc<HealthMonitor>,
}

impl HealthCheckRegistry {
    pub fn new(monitor: Arc<HealthMonitor>) -> Self {
        Self { monitor }
    }

    /// Register all built-in health checks
    pub async fn register_builtin_checks(&self) -> Result<(), GuardianError> {
        // System health checks
        self.monitor.add_check(builtin::cpu_usage_check()).await;
        self.monitor.add_check(builtin::memory_usage_check()).await;
        self.monitor.add_check(builtin::disk_usage_check()).await;

        // Network health checks
        self.monitor.add_check(builtin::network_connectivity_check("localhost:8080".to_string())).await;
        self.monitor.add_check(builtin::network_connectivity_check("localhost:9090".to_string())).await;

        // Process health checks (non-critical)
        let mut java_check = builtin::process_health_check("java".to_string());
        java_check.critical = false; // Make Java check non-critical
        self.monitor.add_check(java_check).await;

        // Database health check (if configured)
        // self.monitor.add_check(builtin::database_health_check("postgresql://...".to_string())).await;

        info!("Registered built-in health checks");
        Ok(())
    }

    /// Register custom health check
    pub async fn register_custom_check(&self, check: HealthCheck) {
        self.monitor.add_check(check).await;
    }

    /// Get the health monitor
    pub fn get_monitor(&self) -> Arc<HealthMonitor> {
        self.monitor.clone()
    }
}
