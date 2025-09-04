use crate::config::Config;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

/// Plugin metadata and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: String,
    pub license: String,
    pub entry_point: String,
    pub dependencies: Vec<String>,
    pub permissions: Vec<String>,
    pub config_schema: Option<serde_json::Value>,
    pub status: PluginStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Plugin status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginStatus {
    Installed,
    Enabled,
    Disabled,
    Error,
    Loading,
    Uninstalling,
}

/// Plugin runtime information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRuntime {
    pub plugin_id: String,
    pub process_id: Option<u32>,
    pub status: PluginStatus,
    pub last_heartbeat: DateTime<Utc>,
    pub resource_usage: PluginResourceUsage,
    pub logs: Vec<PluginLog>,
    pub events: Vec<PluginEvent>,
}

/// Plugin resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResourceUsage {
    pub cpu_percent: f64,
    pub memory_mb: u64,
    pub disk_mb: u64,
    pub network_rx_mb: u64,
    pub network_tx_mb: u64,
}

/// Plugin log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginLog {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
}

/// Log levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Plugin event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEvent {
    pub id: String,
    pub plugin_id: String,
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

/// Plugin manager for extensible architecture
pub struct PluginManager {
    plugins: Arc<RwLock<HashMap<String, Plugin>>>,
    runtimes: Arc<RwLock<HashMap<String, PluginRuntime>>>,
    plugin_dir: PathBuf,
    sandbox_config: SandboxConfig,
    event_bus: Arc<RwLock<EventBus>>,
}

/// Sandbox configuration for plugin isolation
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub enable_sandbox: bool,
    pub max_memory_mb: u64,
    pub max_cpu_percent: f64,
    pub allowed_network_hosts: Vec<String>,
    pub allowed_file_paths: Vec<PathBuf>,
    pub timeout_seconds: u64,
}

/// Event bus for plugin communication
#[derive(Debug)]
pub struct EventBus {
    subscribers: HashMap<String, Vec<String>>, // event_type -> plugin_ids
    events: Vec<PluginEvent>,
}

/// Plugin API for communication with Guardian
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginApi {
    pub plugin_id: String,
    pub endpoints: Vec<ApiEndpoint>,
    pub webhooks: Vec<WebhookConfig>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoint {
    pub path: String,
    pub method: String,
    pub handler: String,
    pub auth_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub url: String,
    pub events: Vec<String>,
    pub secret: Option<String>,
    pub retry_count: u32,
}

impl PluginManager {
    pub fn new(plugin_dir: PathBuf) -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            runtimes: Arc::new(RwLock::new(HashMap::new())),
            plugin_dir,
            sandbox_config: SandboxConfig::default(),
            event_bus: Arc::new(RwLock::new(EventBus::new())),
        }
    }

    /// Initialize plugin manager
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing plugin manager...");
        
        // Create plugin directory if it doesn't exist
        tokio::fs::create_dir_all(&self.plugin_dir).await?;
        
        // Load existing plugins
        self.load_plugins().await?;
        
        info!("Plugin manager initialized");
        Ok(())
    }

    /// Load plugins from directory
    async fn load_plugins(&self) -> Result<()> {
        let mut entries = tokio::fs::read_dir(&self.plugin_dir).await?;
        let mut plugins = self.plugins.write().await;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(plugin) = self.load_plugin_manifest(&path).await {
                    plugins.insert(plugin.id.clone(), plugin);
                }
            }
        }
        
        info!("Loaded {} plugins", plugins.len());
        Ok(())
    }

    /// Load plugin manifest
    async fn load_plugin_manifest(&self, plugin_path: &PathBuf) -> Result<Plugin> {
        let manifest_path = plugin_path.join("plugin.json");
        let manifest_content = tokio::fs::read_to_string(manifest_path).await?;
        let mut plugin: Plugin = serde_json::from_str(&manifest_content)?;
        
        // Set timestamps
        let now = Utc::now();
        plugin.created_at = now;
        plugin.updated_at = now;
        
        Ok(plugin)
    }

    /// Install plugin from package
    pub async fn install_plugin(&self, package_path: PathBuf) -> Result<Plugin> {
        info!("Installing plugin from: {:?}", package_path);
        
        // Extract plugin package
        let plugin_id = Uuid::new_v4().to_string();
        let plugin_dir = self.plugin_dir.join(&plugin_id);
        tokio::fs::create_dir_all(&plugin_dir).await?;
        
        // TODO: Extract zip/tar package to plugin directory
        // For now, assume it's already extracted
        
        // Load plugin manifest
        let plugin = self.load_plugin_manifest(&plugin_dir).await?;
        
        // Validate plugin
        self.validate_plugin(&plugin).await?;
        
        // Install dependencies
        self.install_dependencies(&plugin).await?;
        
        // Register plugin
        let mut plugins = self.plugins.write().await;
        plugins.insert(plugin.id.clone(), plugin.clone());
        
        info!("Plugin installed: {}", plugin.name);
        Ok(plugin)
    }

    /// Validate plugin
    async fn validate_plugin(&self, plugin: &Plugin) -> Result<()> {
        // Check required fields
        if plugin.name.is_empty() || plugin.entry_point.is_empty() {
            return Err(anyhow::anyhow!("Invalid plugin manifest"));
        }
        
        // Check permissions
        for permission in &plugin.permissions {
            if !self.is_valid_permission(permission) {
                return Err(anyhow::anyhow!("Invalid permission: {}", permission));
            }
        }
        
        // Check dependencies
        let plugins = self.plugins.read().await;
        for dependency in &plugin.dependencies {
            if !plugins.contains_key(dependency) {
                return Err(anyhow::anyhow!("Missing dependency: {}", dependency));
            }
        }
        
        Ok(())
    }

    /// Check if permission is valid
    fn is_valid_permission(&self, permission: &str) -> bool {
        matches!(permission,
            "server:read" | "server:write" | "server:restart" |
            "snapshot:read" | "snapshot:write" | "snapshot:delete" |
            "config:read" | "config:write" |
            "metrics:read" | "webhook:write" | "api:access"
        )
    }

    /// Install plugin dependencies
    async fn install_dependencies(&self, plugin: &Plugin) -> Result<()> {
        for dependency in &plugin.dependencies {
            // TODO: Install dependency (download, extract, etc.)
            info!("Installing dependency: {}", dependency);
        }
        Ok(())
    }

    /// Enable plugin
    pub async fn enable_plugin(&self, plugin_id: &str) -> Result<()> {
        let mut plugins = self.plugins.write().await;
        if let Some(plugin) = plugins.get_mut(plugin_id) {
            plugin.status = PluginStatus::Loading;
            plugin.updated_at = Utc::now();
        }
        drop(plugins);
        
        // Start plugin process
        self.start_plugin(plugin_id).await?;
        
        // Update status
        let mut plugins = self.plugins.write().await;
        if let Some(plugin) = plugins.get_mut(plugin_id) {
            plugin.status = PluginStatus::Enabled;
            plugin.updated_at = Utc::now();
        }
        
        info!("Plugin enabled: {}", plugin_id);
        Ok(())
    }

    /// Start plugin process
    async fn start_plugin(&self, plugin_id: &str) -> Result<()> {
        let plugin = {
            let plugins = self.plugins.read().await;
            plugins.get(plugin_id).cloned()
                .ok_or_else(|| anyhow::anyhow!("Plugin not found"))?
        };
        
        let plugin_dir = self.plugin_dir.join(plugin_id);
        let entry_point = plugin_dir.join(&plugin.entry_point);
        
        // Start plugin process with sandboxing
        let mut cmd = Command::new("node"); // Assuming Node.js plugins for now
        cmd.arg(&entry_point)
            .current_dir(&plugin_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        // Apply sandbox constraints
        if self.sandbox_config.enable_sandbox {
            // TODO: Apply memory limits, CPU limits, network restrictions
            // This would use system calls or containerization
        }
        
        let child = cmd.spawn()?;
        let process_id = child.id();
        
        // Create runtime info
        let runtime = PluginRuntime {
            plugin_id: plugin_id.to_string(),
            process_id: process_id,
            status: PluginStatus::Enabled,
            last_heartbeat: Utc::now(),
            resource_usage: PluginResourceUsage::default(),
            logs: Vec::new(),
            events: Vec::new(),
        };
        
        let mut runtimes = self.runtimes.write().await;
        runtimes.insert(plugin_id.to_string(), runtime);
        
        info!("Plugin process started: {} (PID: {:?})", plugin_id, process_id);
        Ok(())
    }

    /// Disable plugin
    pub async fn disable_plugin(&self, plugin_id: &str) -> Result<()> {
        // Stop plugin process
        self.stop_plugin(plugin_id).await?;
        
        // Update status
        let mut plugins = self.plugins.write().await;
        if let Some(plugin) = plugins.get_mut(plugin_id) {
            plugin.status = PluginStatus::Disabled;
            plugin.updated_at = Utc::now();
        }
        
        info!("Plugin disabled: {}", plugin_id);
        Ok(())
    }

    /// Stop plugin process
    async fn stop_plugin(&self, plugin_id: &str) -> Result<()> {
        let mut runtimes = self.runtimes.write().await;
        if let Some(runtime) = runtimes.get_mut(plugin_id) {
            if let Some(pid) = runtime.process_id {
                // TODO: Gracefully terminate process
                runtime.process_id = None;
                runtime.status = PluginStatus::Disabled;
            }
        }
        
        info!("Plugin process stopped: {}", plugin_id);
        Ok(())
    }

    /// Uninstall plugin
    pub async fn uninstall_plugin(&self, plugin_id: &str) -> Result<()> {
        // Disable plugin first
        self.disable_plugin(plugin_id).await?;
        
        // Remove from registry
        let mut plugins = self.plugins.write().await;
        plugins.remove(plugin_id);
        
        let mut runtimes = self.runtimes.write().await;
        runtimes.remove(plugin_id);
        
        // Remove plugin files
        let plugin_dir = self.plugin_dir.join(plugin_id);
        if plugin_dir.exists() {
            tokio::fs::remove_dir_all(&plugin_dir).await?;
        }
        
        info!("Plugin uninstalled: {}", plugin_id);
        Ok(())
    }

    /// Get plugin by ID
    pub async fn get_plugin(&self, plugin_id: &str) -> Option<Plugin> {
        let plugins = self.plugins.read().await;
        plugins.get(plugin_id).cloned()
    }

    /// List all plugins
    pub async fn list_plugins(&self) -> Vec<Plugin> {
        let plugins = self.plugins.read().await;
        plugins.values().cloned().collect()
    }

    /// Get plugin runtime info
    pub async fn get_plugin_runtime(&self, plugin_id: &str) -> Option<PluginRuntime> {
        let runtimes = self.runtimes.read().await;
        runtimes.get(plugin_id).cloned()
    }

    /// Publish event to plugins
    pub async fn publish_event(&self, event_type: &str, data: serde_json::Value) -> Result<()> {
        let event = PluginEvent {
            id: Uuid::new_v4().to_string(),
            plugin_id: "system".to_string(),
            event_type: event_type.to_string(),
            data,
            timestamp: Utc::now(),
        };
        
        let mut event_bus = self.event_bus.write().await;
        event_bus.publish_event(event).await;
        
        Ok(())
    }

    /// Subscribe plugin to events
    pub async fn subscribe_plugin(&self, plugin_id: &str, event_types: Vec<String>) -> Result<()> {
        let mut event_bus = self.event_bus.write().await;
        event_bus.subscribe(plugin_id, event_types);
        Ok(())
    }

    /// Update plugin configuration
    pub async fn update_plugin_config(&self, plugin_id: &str, config: serde_json::Value) -> Result<()> {
        // TODO: Send configuration update to plugin
        info!("Updated plugin configuration: {}", plugin_id);
        Ok(())
    }

    /// Get plugin logs
    pub async fn get_plugin_logs(&self, plugin_id: &str, limit: usize) -> Vec<PluginLog> {
        let runtimes = self.runtimes.read().await;
        if let Some(runtime) = runtimes.get(plugin_id) {
            runtime.logs.iter()
                .rev()
                .take(limit)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enable_sandbox: true,
            max_memory_mb: 512,
            max_cpu_percent: 50.0,
            allowed_network_hosts: vec!["localhost".to_string(), "127.0.0.1".to_string()],
            allowed_file_paths: vec![PathBuf::from("/tmp")],
            timeout_seconds: 30,
        }
    }
}

impl Default for PluginResourceUsage {
    fn default() -> Self {
        Self {
            cpu_percent: 0.0,
            memory_mb: 0,
            disk_mb: 0,
            network_rx_mb: 0,
            network_tx_mb: 0,
        }
    }
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscribers: HashMap::new(),
            events: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, plugin_id: &str, event_types: Vec<String>) {
        for event_type in event_types {
            self.subscribers.entry(event_type)
                .or_insert_with(Vec::new)
                .push(plugin_id.to_string());
        }
    }

    pub async fn publish_event(&mut self, event: PluginEvent) {
        // Store event
        self.events.push(event.clone());
        
        // Notify subscribers
        if let Some(subscribers) = self.subscribers.get(&event.event_type) {
            for plugin_id in subscribers {
                // TODO: Send event to plugin via IPC/HTTP
                // Event sent to plugin
            }
        }
        
        // Keep only recent events (last 1000)
        if self.events.len() > 1000 {
            self.events.drain(0..self.events.len() - 1000);
        }
    }
}
