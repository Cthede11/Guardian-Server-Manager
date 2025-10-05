use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use std::time::Duration;

use crate::core::{
    config::MonitoringConfig,
    error_handler::{AppError, Result},
};

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub status: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub details: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_usage: f32,
    pub network_in: u64,
    pub network_out: u64,
    pub uptime: Duration,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub id: Uuid,
    pub server_id: Option<Uuid>,
    pub level: AlertLevel,
    pub title: String,
    pub message: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_resolved: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

pub struct MonitoringManager {
    config: MonitoringConfig,
    health_status: Arc<RwLock<HealthStatus>>,
    system_metrics: Arc<RwLock<SystemMetrics>>,
    alerts: Arc<RwLock<HashMap<Uuid, Alert>>>,
    is_running: Arc<RwLock<bool>>,
}

impl MonitoringManager {
    pub fn new(config: &MonitoringConfig) -> Result<Self> {
        let health_status = HealthStatus {
            status: "healthy".to_string(),
            message: "System is running normally".to_string(),
            timestamp: chrono::Utc::now(),
            details: HashMap::new(),
        };
        
        let system_metrics = SystemMetrics {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            disk_usage: 0.0,
            network_in: 0,
            network_out: 0,
            uptime: Duration::ZERO,
            timestamp: chrono::Utc::now(),
        };
        
        Ok(Self {
            config: config.clone(),
            health_status: Arc::new(RwLock::new(health_status)),
            system_metrics: Arc::new(RwLock::new(system_metrics)),
            alerts: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
        })
    }
    
    pub async fn start(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = true;
        drop(is_running);
        
        // Start system monitoring
        self.start_system_monitoring().await?;
        
        // Start health checks
        self.start_health_checks().await?;
        
        // Start alert processing
        self.start_alert_processing().await?;
        
        Ok(())
    }
    
    pub async fn stop(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        
        Ok(())
    }
    
    pub async fn get_health_status(&self) -> HealthStatus {
        let health_status = self.health_status.read().await;
        health_status.clone()
    }
    
    pub async fn get_system_metrics(&self) -> SystemMetrics {
        let system_metrics = self.system_metrics.read().await;
        system_metrics.clone()
    }
    
    pub async fn get_alerts(&self) -> Vec<Alert> {
        let alerts = self.alerts.read().await;
        alerts.values().cloned().collect()
    }
    
    pub async fn create_alert(&self, server_id: Option<Uuid>, level: AlertLevel, title: String, message: String) -> Result<Uuid> {
        let alert = Alert {
            id: Uuid::new_v4(),
            server_id,
            level,
            title,
            message,
            created_at: chrono::Utc::now(),
            resolved_at: None,
            is_resolved: false,
        };
        
        let mut alerts = self.alerts.write().await;
        alerts.insert(alert.id, alert.clone());
        
        // Log the alert
        tracing::warn!("Alert created: {} - {}", alert.title, alert.message);
        
        Ok(alert.id)
    }
    
    pub async fn resolve_alert(&self, alert_id: Uuid) -> Result<()> {
        let mut alerts = self.alerts.write().await;
        
        if let Some(alert) = alerts.get_mut(&alert_id) {
            alert.is_resolved = true;
            alert.resolved_at = Some(chrono::Utc::now());
        }
        
        Ok(())
    }
    
    async fn start_system_monitoring(&self) -> Result<()> {
        let system_metrics = self.system_metrics.clone();
        let is_running = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            
            loop {
                interval.tick().await;
                
                let running = is_running.read().await;
                if !*running {
                    break;
                }
                drop(running);
                
                // Collect system metrics
                let metrics = Self::collect_system_metrics().await;
                
                let mut system_metrics_guard = system_metrics.write().await;
                *system_metrics_guard = metrics;
            }
        });
        
        Ok(())
    }
    
    async fn start_health_checks(&self) -> Result<()> {
        let health_status = self.health_status.clone();
        let is_running = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                let running = is_running.read().await;
                if !*running {
                    break;
                }
                drop(running);
                
                // Perform health checks
                let health = Self::perform_health_checks().await;
                
                let mut health_status_guard = health_status.write().await;
                *health_status_guard = health;
            }
        });
        
        Ok(())
    }
    
    async fn start_alert_processing(&self) -> Result<()> {
        let system_metrics = self.system_metrics.clone();
        let alerts = self.alerts.clone();
        let is_running = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                let running = is_running.read().await;
                if !*running {
                    break;
                }
                drop(running);
                
                // Check for alert conditions
                let metrics = system_metrics.read().await;
                
                // Check CPU usage
                if metrics.cpu_usage > 90.0 {
                    Self::create_alert_if_not_exists(
                        &alerts,
                        None,
                        AlertLevel::Critical,
                        "High CPU Usage".to_string(),
                        format!("CPU usage is at {:.1}%", metrics.cpu_usage),
                    ).await;
                }
                
                // Check memory usage
                if metrics.memory_usage > 90.0 {
                    Self::create_alert_if_not_exists(
                        &alerts,
                        None,
                        AlertLevel::Critical,
                        "High Memory Usage".to_string(),
                        format!("Memory usage is at {:.1}%", metrics.memory_usage),
                    ).await;
                }
                
                // Check disk usage
                if metrics.disk_usage > 90.0 {
                    Self::create_alert_if_not_exists(
                        &alerts,
                        None,
                        AlertLevel::Critical,
                        "High Disk Usage".to_string(),
                        format!("Disk usage is at {:.1}%", metrics.disk_usage),
                    ).await;
                }
            }
        });
        
        Ok(())
    }
    
    async fn collect_system_metrics() -> SystemMetrics {
        // This is a simplified implementation
        // In production, you would use system monitoring libraries
        
        SystemMetrics {
            cpu_usage: 50.0 + (rand::random::<f32>() - 0.5) * 20.0,
            memory_usage: 60.0 + (rand::random::<f32>() - 0.5) * 20.0,
            disk_usage: 40.0 + (rand::random::<f32>() - 0.5) * 20.0,
            network_in: rand::random::<u64>() % 1000000,
            network_out: rand::random::<u64>() % 1000000,
            uptime: Duration::from_secs(3600), // 1 hour
            timestamp: chrono::Utc::now(),
        }
    }
    
    async fn perform_health_checks() -> HealthStatus {
        // This is a simplified implementation
        // In production, you would check various system components
        
        HealthStatus {
            status: "healthy".to_string(),
            message: "All systems operational".to_string(),
            timestamp: chrono::Utc::now(),
            details: HashMap::new(),
        }
    }
    
    async fn create_alert_if_not_exists(
        alerts: &Arc<RwLock<HashMap<Uuid, Alert>>>,
        server_id: Option<Uuid>,
        level: AlertLevel,
        title: String,
        message: String,
    ) {
        let alerts_guard = alerts.read().await;
        
        // Check if similar alert already exists
        let exists = alerts_guard.values().any(|alert| {
            alert.server_id == server_id && 
            alert.title == title && 
            !alert.is_resolved
        });
        
        if !exists {
            drop(alerts_guard);
            
            let alert = Alert {
                id: Uuid::new_v4(),
                server_id,
                level,
                title,
                message,
                created_at: chrono::Utc::now(),
                resolved_at: None,
                is_resolved: false,
            };
            
            let mut alerts_guard = alerts.write().await;
            alerts_guard.insert(alert.id, alert);
        }
    }
}
