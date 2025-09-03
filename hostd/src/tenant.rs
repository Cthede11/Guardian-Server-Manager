use crate::config::Config;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Tenant represents a isolated server instance with its own resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: String,
    pub config: TenantConfig,
    pub resources: ResourceLimits,
    pub status: TenantStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Tenant configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantConfig {
    pub minecraft_version: String,
    pub modpack_id: Option<String>,
    pub server_properties: HashMap<String, String>,
    pub guardian_config: Config,
    pub custom_rules: Option<String>,
    pub plugins: Vec<String>,
}

/// Resource limits for tenant isolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub disk_gb: u32,
    pub network_bandwidth_mbps: u32,
    pub max_players: u32,
    pub max_instances: u32,
}

/// Tenant status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TenantStatus {
    Active,
    Suspended,
    Maintenance,
    Deleted,
}

/// Server instance within a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInstance {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub status: InstanceStatus,
    pub config: InstanceConfig,
    pub resources: ResourceUsage,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
}

/// Instance status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InstanceStatus {
    Running,
    Stopped,
    Starting,
    Stopping,
    Error,
    Maintenance,
}

/// Instance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceConfig {
    pub port: u16,
    pub world_name: String,
    pub difficulty: String,
    pub gamemode: String,
    pub max_players: u32,
    pub motd: String,
    pub whitelist_enabled: bool,
    pub whitelist: Vec<String>,
    pub ops: Vec<String>,
}

/// Resource usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f64,
    pub memory_mb: u64,
    pub disk_mb: u64,
    pub network_rx_mb: u64,
    pub network_tx_mb: u64,
    pub players_online: u32,
    pub tps: f64,
    pub uptime_seconds: u64,
}

/// Tenant manager for multi-tenancy
pub struct TenantManager {
    tenants: Arc<RwLock<HashMap<String, Tenant>>>,
    instances: Arc<RwLock<HashMap<String, ServerInstance>>>,
    resource_monitor: Arc<RwLock<ResourceMonitor>>,
}

/// Resource monitoring for tenant isolation
#[derive(Debug)]
pub struct ResourceMonitor {
    usage: HashMap<String, ResourceUsage>,
    alerts: Vec<ResourceAlert>,
}

#[derive(Debug, Clone)]
pub struct ResourceAlert {
    pub tenant_id: String,
    pub instance_id: Option<String>,
    pub alert_type: AlertType,
    pub message: String,
    pub severity: AlertSeverity,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlertType {
    CpuHigh,
    MemoryHigh,
    DiskFull,
    NetworkCongestion,
    PlayerLimit,
    InstanceLimit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl TenantManager {
    pub fn new() -> Self {
        Self {
            tenants: Arc::new(RwLock::new(HashMap::new())),
            instances: Arc::new(RwLock::new(HashMap::new())),
            resource_monitor: Arc::new(RwLock::new(ResourceMonitor::new())),
        }
    }

    /// Create a new tenant
    pub async fn create_tenant(&self, name: String, owner_id: String, config: TenantConfig) -> Result<Tenant> {
        let tenant_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let tenant = Tenant {
            id: tenant_id.clone(),
            name: name.clone(),
            description: None,
            owner_id,
            config,
            resources: ResourceLimits::default(),
            status: TenantStatus::Active,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        };

        let mut tenants = self.tenants.write().await;
        tenants.insert(tenant_id.clone(), tenant.clone());
        
        info!("Created tenant: {} ({})", name, tenant_id);
        Ok(tenant)
    }

    /// Get tenant by ID
    pub async fn get_tenant(&self, tenant_id: &str) -> Option<Tenant> {
        let tenants = self.tenants.read().await;
        tenants.get(tenant_id).cloned()
    }

    /// List tenants for a user
    pub async fn list_tenants(&self, owner_id: &str) -> Vec<Tenant> {
        let tenants = self.tenants.read().await;
        tenants.values()
            .filter(|t| t.owner_id == owner_id)
            .cloned()
            .collect()
    }

    /// Update tenant configuration
    pub async fn update_tenant(&self, tenant_id: &str, updates: TenantUpdate) -> Result<()> {
        let mut tenants = self.tenants.write().await;
        if let Some(tenant) = tenants.get_mut(tenant_id) {
            if let Some(name) = updates.name {
                tenant.name = name;
            }
            if let Some(description) = updates.description {
                tenant.description = Some(description);
            }
            if let Some(config) = updates.config {
                tenant.config = config;
            }
            if let Some(resources) = updates.resources {
                tenant.resources = resources;
            }
            tenant.updated_at = Utc::now();
            
            info!("Updated tenant: {}", tenant_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Tenant not found: {}", tenant_id))
        }
    }

    /// Create server instance within tenant
    pub async fn create_instance(&self, tenant_id: &str, name: String, config: InstanceConfig) -> Result<ServerInstance> {
        // Check tenant exists and is active
        let tenant = self.get_tenant(tenant_id).await
            .ok_or_else(|| anyhow::anyhow!("Tenant not found"))?;
        
        if tenant.status != TenantStatus::Active {
            return Err(anyhow::anyhow!("Tenant is not active"));
        }

        // Check instance limit
        let instances = self.instances.read().await;
        let tenant_instances: Vec<_> = instances.values()
            .filter(|i| i.tenant_id == tenant_id)
            .collect();
        
        if tenant_instances.len() >= tenant.resources.max_instances as usize {
            return Err(anyhow::anyhow!("Instance limit reached for tenant"));
        }
        drop(instances);

        let instance_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let instance = ServerInstance {
            id: instance_id.clone(),
            tenant_id: tenant_id.to_string(),
            name: name.clone(),
            status: InstanceStatus::Stopped,
            config,
            resources: ResourceUsage::default(),
            created_at: now,
            last_activity: now,
        };

        let mut instances = self.instances.write().await;
        instances.insert(instance_id.clone(), instance.clone());
        
        info!("Created instance: {} in tenant: {}", name, tenant_id);
        Ok(instance)
    }

    /// Get instance by ID
    pub async fn get_instance(&self, instance_id: &str) -> Option<ServerInstance> {
        let instances = self.instances.read().await;
        instances.get(instance_id).cloned()
    }

    /// List instances for a tenant
    pub async fn list_instances(&self, tenant_id: &str) -> Vec<ServerInstance> {
        let instances = self.instances.read().await;
        instances.values()
            .filter(|i| i.tenant_id == tenant_id)
            .cloned()
            .collect()
    }

    /// Start instance
    pub async fn start_instance(&self, instance_id: &str) -> Result<()> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(instance_id) {
            instance.status = InstanceStatus::Starting;
            instance.last_activity = Utc::now();
            
            info!("Starting instance: {}", instance_id);
            
            // TODO: Actually start the Minecraft server process
            // This would involve:
            // 1. Allocating resources
            // 2. Starting the server container/process
            // 3. Monitoring startup
            // 4. Updating status to Running
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("Instance not found: {}", instance_id))
        }
    }

    /// Stop instance
    pub async fn stop_instance(&self, instance_id: &str) -> Result<()> {
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(instance_id) {
            instance.status = InstanceStatus::Stopping;
            instance.last_activity = Utc::now();
            
            info!("Stopping instance: {}", instance_id);
            
            // TODO: Actually stop the Minecraft server process
            // This would involve:
            // 1. Graceful shutdown
            // 2. Resource cleanup
            // 3. Update status to Stopped
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("Instance not found: {}", instance_id))
        }
    }

    /// Update resource usage
    pub async fn update_resource_usage(&self, instance_id: &str, usage: ResourceUsage) -> Result<()> {
        // Update instance resource usage
        let mut instances = self.instances.write().await;
        if let Some(instance) = instances.get_mut(instance_id) {
            instance.resources = usage.clone();
            instance.last_activity = Utc::now();
        }
        drop(instances);

        // Update resource monitor
        let mut monitor = self.resource_monitor.write().await;
        monitor.update_usage(instance_id, usage).await;
        
        Ok(())
    }

    /// Get resource alerts
    pub async fn get_resource_alerts(&self) -> Vec<ResourceAlert> {
        let monitor = self.resource_monitor.read().await;
        monitor.alerts.clone()
    }

    /// Delete tenant (soft delete)
    pub async fn delete_tenant(&self, tenant_id: &str) -> Result<()> {
        let mut tenants = self.tenants.write().await;
        if let Some(tenant) = tenants.get_mut(tenant_id) {
            tenant.status = TenantStatus::Deleted;
            tenant.updated_at = Utc::now();
            
            info!("Deleted tenant: {}", tenant_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Tenant not found: {}", tenant_id))
        }
    }
}

#[derive(Debug, Clone)]
pub struct TenantUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub config: Option<TenantConfig>,
    pub resources: Option<ResourceLimits>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu_cores: 4,
            memory_gb: 8,
            disk_gb: 100,
            network_bandwidth_mbps: 1000,
            max_players: 20,
            max_instances: 5,
        }
    }
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_percent: 0.0,
            memory_mb: 0,
            disk_mb: 0,
            network_rx_mb: 0,
            network_tx_mb: 0,
            players_online: 0,
            tps: 20.0,
            uptime_seconds: 0,
        }
    }
}

impl ResourceMonitor {
    pub fn new() -> Self {
        Self {
            usage: HashMap::new(),
            alerts: Vec::new(),
        }
    }

    pub async fn update_usage(&mut self, instance_id: &str, usage: ResourceUsage) {
        self.usage.insert(instance_id.to_string(), usage.clone());
        
        // Check for resource alerts
        self.check_alerts(instance_id, &usage).await;
    }

    async fn check_alerts(&mut self, instance_id: &str, usage: &ResourceUsage) {
        let now = Utc::now();
        
        // CPU alert
        if usage.cpu_percent > 90.0 {
            self.add_alert(ResourceAlert {
                tenant_id: "unknown".to_string(), // TODO: Get from instance
                instance_id: Some(instance_id.to_string()),
                alert_type: AlertType::CpuHigh,
                message: format!("High CPU usage: {:.1}%", usage.cpu_percent),
                severity: AlertSeverity::Warning,
                timestamp: now,
            });
        }

        // Memory alert
        if usage.memory_mb > 6 * 1024 { // 6GB
            self.add_alert(ResourceAlert {
                tenant_id: "unknown".to_string(),
                instance_id: Some(instance_id.to_string()),
                alert_type: AlertType::MemoryHigh,
                message: format!("High memory usage: {}MB", usage.memory_mb),
                severity: AlertSeverity::Warning,
                timestamp: now,
            });
        }

        // TPS alert
        if usage.tps < 15.0 {
            self.add_alert(ResourceAlert {
                tenant_id: "unknown".to_string(),
                instance_id: Some(instance_id.to_string()),
                alert_type: AlertType::CpuHigh, // Reuse for TPS
                message: format!("Low TPS: {:.1}", usage.tps),
                severity: AlertSeverity::Critical,
                timestamp: now,
            });
        }
    }

    fn add_alert(&mut self, alert: ResourceAlert) {
        // Remove old alerts (older than 1 hour)
        let cutoff = Utc::now() - chrono::Duration::hours(1);
        self.alerts.retain(|a| a.timestamp > cutoff);
        
        self.alerts.push(alert);
    }
}
