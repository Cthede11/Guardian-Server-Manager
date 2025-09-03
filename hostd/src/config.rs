use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::Result;
use tracing::{info, error};

/// Configuration for the Guardian Host Daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub minecraft: MinecraftConfig,
    pub paths: PathsConfig,
    pub compat: CompatConfig,
    pub self_heal: SelfHealConfig,
    pub gpu: GpuConfig,
    pub worldgen: WorldgenConfig,
    pub ha: HighAvailabilityConfig,
    pub monitoring: MonitoringConfig,
    pub performance: PerformanceConfig,
    pub mod_configs: ModConfigs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftConfig {
    pub loader: String,
    pub version: String,
    pub java: JavaConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaConfig {
    pub heap_gb: u32,
    pub flags: String,
    pub extra_flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    pub mods_dir: String,
    pub config_dir: String,
    pub world_dir: String,
    pub backup_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatConfig {
    pub rules_file: String,
    pub allow_bake_for_permissive_licenses: bool,
    pub auto_apply_rules: bool,
    pub rule_refresh_interval_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfHealConfig {
    pub entity_freeze_threshold: u32,
    pub block_entity_freeze_threshold: u32,
    pub quarantine_dimension: String,
    pub thaw_on_rule_update: bool,
    pub max_frozen_entities: u32,
    pub max_frozen_block_entities: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuConfig {
    pub enabled: bool,
    pub worker_ipc: String,
    pub batch_size_chunks: u32,
    pub max_cache_size: u32,
    pub health_check_interval_seconds: u32,
    pub fallback_to_cpu: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldgenConfig {
    pub gpu_acceleration: bool,
    pub async_generation: bool,
    pub pregeneration_enabled: bool,
    pub pregeneration_radius: u32,
    pub pregeneration_threads: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighAvailabilityConfig {
    pub autosave_minutes: u32,
    pub snapshot_keep: u32,
    pub blue_green: bool,
    pub rollback_on_failure: bool,
    pub max_restart_attempts: u32,
    pub restart_delay_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub metrics_port: u16,

    pub log_level: String,
    pub crash_reporting: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub tick_time_warning_ms: u32,
    pub tick_time_critical_ms: u32,
    pub memory_warning_percent: u32,
    pub memory_critical_percent: u32,
    pub gc_optimization: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModConfigs {
    pub create: ModConfig,
    pub flywheel: ModConfig,
    pub embeddium: ModConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModConfig {
    pub contraption_optimization: Option<bool>,
    pub flywheel_integration: Option<bool>,
    pub render_optimization: Option<bool>,
    pub batching_enabled: Option<bool>,
    pub shader_optimization: Option<bool>,
}

impl Config {
    /// Load configuration from a YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        if !path.exists() {
            error!("Configuration file not found: {:?}", path);
            return Err(anyhow::anyhow!("Configuration file not found: {:?}", path));
        }
        
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        
        info!("Configuration loaded from: {:?}", path);
        Ok(config)
    }
    
    /// Save configuration to a YAML file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let content = serde_yaml::to_string(self)?;
        std::fs::write(path, content)?;
        
        info!("Configuration saved to: {:?}", path);
        Ok(())
    }
    
    /// Create default configuration
    pub fn default() -> Self {
        Self {
            minecraft: MinecraftConfig {
                loader: "neoforge".to_string(),
                version: "1.20.1".to_string(),
                java: JavaConfig {
                    heap_gb: 10,
                    flags: "g1gc-balanced".to_string(),
                    extra_flags: vec![],
                },
            },
            paths: PathsConfig {
                mods_dir: "./mods".to_string(),
                config_dir: "./config".to_string(),
                world_dir: "./world".to_string(),
                backup_dir: "./backups".to_string(),
            },
            compat: CompatConfig {
                rules_file: "./configs/rules.yaml".to_string(),
                allow_bake_for_permissive_licenses: false,
                auto_apply_rules: true,
                rule_refresh_interval_minutes: 5,
            },
            self_heal: SelfHealConfig {
                entity_freeze_threshold: 3,
                block_entity_freeze_threshold: 3,
                quarantine_dimension: "guardian_hold".to_string(),
                thaw_on_rule_update: true,
                max_frozen_entities: 1000,
                max_frozen_block_entities: 500,
            },
            gpu: GpuConfig {
                enabled: true,
                worker_ipc: "shm".to_string(),
                batch_size_chunks: 64,
                max_cache_size: 1000,
                health_check_interval_seconds: 60,
                fallback_to_cpu: true,
            },
            worldgen: WorldgenConfig {
                gpu_acceleration: true,
                async_generation: true,
                pregeneration_enabled: false,
                pregeneration_radius: 1000,
                pregeneration_threads: 4,
            },
            ha: HighAvailabilityConfig {
                autosave_minutes: 5,
                snapshot_keep: 24,
                blue_green: true,
                rollback_on_failure: true,
                max_restart_attempts: 3,
                restart_delay_seconds: 10,
            },
            monitoring: MonitoringConfig {
                metrics_enabled: true,
                metrics_port: 9090,

                log_level: "INFO".to_string(),
                crash_reporting: true,
            },
            performance: PerformanceConfig {
                tick_time_warning_ms: 50,
                tick_time_critical_ms: 100,
                memory_warning_percent: 80,
                memory_critical_percent: 90,
                gc_optimization: true,
            },
            mod_configs: ModConfigs {
                create: ModConfig {
                    contraption_optimization: Some(true),
                    flywheel_integration: Some(true),
                    render_optimization: None,
                    batching_enabled: None,
                    shader_optimization: None,
                },
                flywheel: ModConfig {
                    contraption_optimization: None,
                    flywheel_integration: None,
                    render_optimization: Some(true),
                    batching_enabled: Some(true),
                    shader_optimization: None,
                },
                embeddium: ModConfig {
                    contraption_optimization: None,
                    flywheel_integration: None,
                    render_optimization: Some(true),
                    batching_enabled: None,
                    shader_optimization: Some(true),
                },
            },
        }
    }
}
