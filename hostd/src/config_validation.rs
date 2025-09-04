use crate::config::Config;
use crate::error::{GuardianError, utils as error_utils};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{info, warn, error};
use validator::{Validate, ValidationError, ValidationErrors};

// Validation functions
fn validate_minecraft_config(_config: &MinecraftValidation) -> Result<(), ValidationError> {
    Ok(())
}

fn validate_paths_config(_config: &PathsValidation) -> Result<(), ValidationError> {
    Ok(())
}

fn validate_performance_config(_config: &PerformanceValidation) -> Result<(), ValidationError> {
    Ok(())
}

fn validate_monitoring_config(_config: &MonitoringValidation) -> Result<(), ValidationError> {
    Ok(())
}

/// Configuration validation rules and schemas
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ConfigValidationRules {
    #[validate(custom = "validate_minecraft_config")]
    pub minecraft: MinecraftValidation,
    
    #[validate(custom = "validate_paths_config")]
    pub paths: PathsValidation,
    
    #[validate(custom = "validate_compat_config")]
    pub compat: CompatValidation,
    
    #[validate(custom = "validate_self_heal_config")]
    pub self_heal: SelfHealValidation,
    
    #[validate(custom = "validate_gpu_config")]
    pub gpu: GpuValidation,
    
    #[validate(custom = "validate_worldgen_config")]
    pub worldgen: WorldgenValidation,
    
    #[validate(custom = "validate_ha_config")]
    pub ha: HighAvailabilityValidation,
    
    #[validate(custom = "validate_monitoring_config")]
    pub monitoring: MonitoringValidation,
    
    #[validate(custom = "validate_performance_config")]
    pub performance: PerformanceValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MinecraftValidation {
    #[validate(length(min = 1, max = 50))]
    pub loader: String,
    
    #[validate(regex = "VERSION_REGEX")]
    pub version: String,
    
    #[validate(custom = "validate_java_config")]
    pub java: JavaValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct JavaValidation {
    #[validate(range(min = 1, max = 64))]
    pub heap_gb: u32,
    
    #[validate(length(min = 1, max = 100))]
    pub flags: String,
    
    #[validate(length(max = 20))]
    pub extra_flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PathsValidation {
    #[validate(custom = "validate_path")]
    pub mods_dir: String,
    
    #[validate(custom = "validate_path")]
    pub config_dir: String,
    
    #[validate(custom = "validate_path")]
    pub world_dir: String,
    
    #[validate(custom = "validate_path")]
    pub backup_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CompatValidation {
    #[validate(custom = "validate_path")]
    pub rules_file: String,
    
    pub allow_bake_for_permissive_licenses: bool,
    
    pub auto_apply_rules: bool,
    
    #[validate(range(min = 1, max = 1440))]
    pub rule_refresh_interval_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SelfHealValidation {
    #[validate(range(min = 1, max = 100))]
    pub entity_freeze_threshold: u32,
    
    #[validate(range(min = 1, max = 100))]
    pub block_entity_freeze_threshold: u32,
    
    #[validate(length(min = 1, max = 50))]
    pub quarantine_dimension: String,
    
    pub thaw_on_rule_update: bool,
    
    #[validate(range(min = 1, max = 10000))]
    pub max_frozen_entities: u32,
    
    #[validate(range(min = 1, max = 10000))]
    pub max_frozen_block_entities: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct GpuValidation {
    pub enabled: bool,
    
    #[validate(custom = "validate_ipc_method")]
    pub worker_ipc: String,
    
    #[validate(range(min = 1, max = 1000))]
    pub batch_size_chunks: u32,
    
    #[validate(range(min = 1, max = 10000))]
    pub max_cache_size: u32,
    
    #[validate(range(min = 1, max = 3600))]
    pub health_check_interval_seconds: u32,
    
    pub fallback_to_cpu: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct WorldgenValidation {
    pub gpu_acceleration: bool,
    
    pub async_generation: bool,
    
    pub pregeneration_enabled: bool,
    
    #[validate(range(min = 100, max = 10000))]
    pub pregeneration_radius: u32,
    
    #[validate(range(min = 1, max = 32))]
    pub pregeneration_threads: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct HighAvailabilityValidation {
    #[validate(range(min = 1, max = 60))]
    pub autosave_minutes: u32,
    
    #[validate(range(min = 1, max = 1000))]
    pub snapshot_keep: u32,
    
    pub blue_green: bool,
    
    pub rollback_on_failure: bool,
    
    #[validate(range(min = 1, max = 10))]
    pub max_restart_attempts: u32,
    
    #[validate(range(min = 1, max = 300))]
    pub restart_delay_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MonitoringValidation {
    pub metrics_enabled: bool,
    
    #[validate(range(min = 1024, max = 65535))]
    pub metrics_port: u16,
    
    #[validate(range(min = 1024, max = 65535))]

    
    #[validate(custom = "validate_log_level")]
    pub log_level: String,
    
    pub crash_reporting: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PerformanceValidation {
    #[validate(range(min = 10, max = 1000))]
    pub tick_time_warning_ms: u32,
    
    #[validate(range(min = 10, max = 1000))]
    pub tick_time_critical_ms: u32,
    
    #[validate(range(min = 50, max = 100))]
    pub memory_warning_percent: u32,
    
    #[validate(range(min = 50, max = 100))]
    pub memory_critical_percent: u32,
    
    pub gc_optimization: bool,
}

/// Configuration validator
pub struct ConfigValidator {
    rules: ConfigValidationRules,
    environment: Environment,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl ConfigValidator {
    pub fn new(environment: Environment) -> Self {
        Self {
            rules: ConfigValidationRules::default(),
            environment,
        }
    }

    /// Validate configuration with comprehensive checks
    pub async fn validate_config(&self, config: &Config) -> Result<(), GuardianError> {
        info!("Starting configuration validation for environment: {:?}", self.environment);

        // Convert config to validation struct
        let validation_config = self.convert_to_validation_config(config)?;

        // Run validation rules
        if let Err(validation_errors) = validation_config.validate() {
            return Err(self.handle_validation_errors(validation_errors));
        }

        // Environment-specific validation
        self.validate_environment_specific(config).await?;

        // Security validation
        self.validate_security(config).await?;

        // Performance validation
        self.validate_performance(config).await?;

        // Cross-field validation
        self.validate_cross_fields(config).await?;

        info!("Configuration validation completed successfully");
        Ok(())
    }

    /// Convert Config to validation struct
    fn convert_to_validation_config(&self, config: &Config) -> Result<ConfigValidationRules, GuardianError> {
        Ok(ConfigValidationRules {
            minecraft: MinecraftValidation {
                loader: config.minecraft.loader.clone(),
                version: config.minecraft.version.clone(),
                java: JavaValidation {
                    heap_gb: config.minecraft.java.heap_gb,
                    flags: config.minecraft.java.flags.clone(),
                    extra_flags: config.minecraft.java.extra_flags.clone(),
                },
            },
            paths: PathsValidation {
                mods_dir: config.paths.mods_dir.clone(),
                config_dir: config.paths.config_dir.clone(),
                world_dir: config.paths.world_dir.clone(),
                backup_dir: config.paths.backup_dir.clone(),
            },
            compat: CompatValidation {
                rules_file: config.compat.rules_file.clone(),
                allow_bake_for_permissive_licenses: config.compat.allow_bake_for_permissive_licenses,
                auto_apply_rules: config.compat.auto_apply_rules,
                rule_refresh_interval_minutes: config.compat.rule_refresh_interval_minutes,
            },
            self_heal: SelfHealValidation {
                entity_freeze_threshold: config.self_heal.entity_freeze_threshold,
                block_entity_freeze_threshold: config.self_heal.block_entity_freeze_threshold,
                quarantine_dimension: config.self_heal.quarantine_dimension.clone(),
                thaw_on_rule_update: config.self_heal.thaw_on_rule_update,
                max_frozen_entities: config.self_heal.max_frozen_entities,
                max_frozen_block_entities: config.self_heal.max_frozen_block_entities,
            },
            gpu: GpuValidation {
                enabled: config.gpu.enabled,
                worker_ipc: config.gpu.worker_ipc.clone(),
                batch_size_chunks: config.gpu.batch_size_chunks,
                max_cache_size: config.gpu.max_cache_size,
                health_check_interval_seconds: config.gpu.health_check_interval_seconds,
                fallback_to_cpu: config.gpu.fallback_to_cpu,
            },
            worldgen: WorldgenValidation {
                gpu_acceleration: config.worldgen.gpu_acceleration,
                async_generation: config.worldgen.async_generation,
                pregeneration_enabled: config.worldgen.pregeneration_enabled,
                pregeneration_radius: config.worldgen.pregeneration_radius,
                pregeneration_threads: config.worldgen.pregeneration_threads,
            },
            ha: HighAvailabilityValidation {
                autosave_minutes: config.ha.autosave_minutes,
                snapshot_keep: config.ha.snapshot_keep,
                blue_green: config.ha.blue_green,
                rollback_on_failure: config.ha.rollback_on_failure,
                max_restart_attempts: config.ha.max_restart_attempts,
                restart_delay_seconds: config.ha.restart_delay_seconds,
            },
            monitoring: MonitoringValidation {
                metrics_enabled: config.monitoring.metrics_enabled,
                metrics_port: config.monitoring.metrics_port,

                log_level: config.monitoring.log_level.clone(),
                crash_reporting: config.monitoring.crash_reporting,
            },
            performance: PerformanceValidation {
                tick_time_warning_ms: config.performance.tick_time_warning_ms,
                tick_time_critical_ms: config.performance.tick_time_critical_ms,
                memory_warning_percent: config.performance.memory_warning_percent,
                memory_critical_percent: config.performance.memory_critical_percent,
                gc_optimization: config.performance.gc_optimization,
            },
        })
    }

    /// Handle validation errors and convert to GuardianError
    fn handle_validation_errors(&self, errors: ValidationErrors) -> GuardianError {
        let mut error_messages = Vec::new();
        
        for (field, field_errors) in errors.field_errors() {
            for error in field_errors {
                let message = format!("Validation failed for field '{}': {:?}", field, error);
                error_messages.push(format!("{}: {}", field, message));
            }
        }

        error_utils::config_error(
            "validation",
            &error_messages.join("; "),
            None,
        )
    }

    /// Environment-specific validation
    async fn validate_environment_specific(&self, config: &Config) -> Result<(), GuardianError> {
        match self.environment {
            Environment::Production => {
                // Production-specific validations
                if config.monitoring.log_level == "DEBUG" {
                    return Err(error_utils::config_error(
                        "log_level",
                        "DEBUG logging is not allowed in production",
                        Some(&config.monitoring.log_level),
                    ));
                }

                if !config.ha.blue_green {
                    warn!("Blue-green deployment is disabled in production environment");
                }

                if config.ha.max_restart_attempts > 3 {
                    return Err(error_utils::config_error(
                        "max_restart_attempts",
                        "Too many restart attempts allowed in production",
                        Some(&config.ha.max_restart_attempts.to_string()),
                    ));
                }
            }
            Environment::Staging => {
                // Staging-specific validations
                if config.ha.max_restart_attempts > 5 {
                    return Err(error_utils::config_error(
                        "max_restart_attempts",
                        "Too many restart attempts allowed in staging",
                        Some(&config.ha.max_restart_attempts.to_string()),
                    ));
                }
            }
            Environment::Development => {
                // Development-specific validations
                // More lenient rules for development
            }
        }

        Ok(())
    }

    /// Security validation
    async fn validate_security(&self, config: &Config) -> Result<(), GuardianError> {
        // Check for insecure configurations

        if config.monitoring.metrics_port < 1024 {
            return Err(error_utils::config_error(
                "metrics_port",
                "Metrics port must be >= 1024 for security",
                Some(&config.monitoring.metrics_port.to_string()),
            ));
        }

        // Check for default/insecure values

        Ok(())
    }

    /// Performance validation
    async fn validate_performance(&self, config: &Config) -> Result<(), GuardianError> {
        // Validate performance thresholds
        if config.performance.tick_time_critical_ms <= config.performance.tick_time_warning_ms {
            return Err(error_utils::config_error(
                "tick_time_critical_ms",
                "Critical tick time must be greater than warning tick time",
                Some(&config.performance.tick_time_critical_ms.to_string()),
            ));
        }

        if config.performance.memory_critical_percent <= config.performance.memory_warning_percent {
            return Err(error_utils::config_error(
                "memory_critical_percent",
                "Critical memory percentage must be greater than warning percentage",
                Some(&config.performance.memory_critical_percent.to_string()),
            ));
        }

        // Validate resource allocation
        if config.minecraft.java.heap_gb < 2 {
            warn!("Low heap size configured: {}GB - may cause performance issues", config.minecraft.java.heap_gb);
        }

        if config.gpu.enabled && config.gpu.batch_size_chunks > 100 {
            warn!("Large GPU batch size: {} - may cause memory issues", config.gpu.batch_size_chunks);
        }

        Ok(())
    }

    /// Cross-field validation
    async fn validate_cross_fields(&self, config: &Config) -> Result<(), GuardianError> {
        // Validate GPU configuration consistency
        if config.gpu.enabled && !config.worldgen.gpu_acceleration {
            warn!("GPU is enabled but worldgen acceleration is disabled");
        }

        // Validate monitoring configuration


        // Validate self-healing configuration
        if config.self_heal.entity_freeze_threshold > config.self_heal.max_frozen_entities {
            return Err(error_utils::config_error(
                "entity_freeze_threshold",
                "Entity freeze threshold cannot be greater than max frozen entities",
                Some(&config.self_heal.entity_freeze_threshold.to_string()),
            ));
        }

        Ok(())
    }

    /// Validate configuration file exists and is readable
    pub async fn validate_config_file<P: AsRef<Path>>(&self, path: P) -> Result<(), GuardianError> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Err(error_utils::config_error(
                "config_file",
                "Configuration file does not exist",
                Some(&path.to_string_lossy()),
            ));
        }

        if !path.is_file() {
            return Err(error_utils::config_error(
                "config_file",
                "Configuration path is not a file",
                Some(&path.to_string_lossy()),
            ));
        }

        // Check file permissions
        if let Err(e) = std::fs::metadata(path) {
            return Err(error_utils::config_error(
                "config_file",
                &format!("Cannot access configuration file: {}", e),
                Some(&path.to_string_lossy()),
            ));
        }

        Ok(())
    }

    /// Generate configuration recommendations
    pub fn generate_recommendations(&self, config: &Config) -> Vec<ConfigRecommendation> {
        let mut recommendations = Vec::new();

        // Performance recommendations
        if config.minecraft.java.heap_gb < 4 {
            recommendations.push(ConfigRecommendation {
                category: "performance".to_string(),
                severity: RecommendationSeverity::Medium,
                message: "Consider increasing heap size to at least 4GB for better performance".to_string(),
                field: "minecraft.java.heap_gb".to_string(),
                suggested_value: Some("4".to_string()),
            });
        }

        // Security recommendations

        // Reliability recommendations
        if !config.ha.blue_green {
            recommendations.push(ConfigRecommendation {
                category: "reliability".to_string(),
                severity: RecommendationSeverity::High,
                message: "Enable blue-green deployment for zero-downtime updates".to_string(),
                field: "ha.blue_green".to_string(),
                suggested_value: Some("true".to_string()),
            });
        }

        recommendations
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRecommendation {
    pub category: String,
    pub severity: RecommendationSeverity,
    pub message: String,
    pub field: String,
    pub suggested_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for ConfigValidationRules {
    fn default() -> Self {
        Self {
            minecraft: MinecraftValidation {
                loader: "neoforge".to_string(),
                version: "1.20.1".to_string(),
                java: JavaValidation {
                    heap_gb: 4,
                    flags: "g1gc-balanced".to_string(),
                    extra_flags: Vec::new(),
                },
            },
            paths: PathsValidation {
                mods_dir: "./mods".to_string(),
                config_dir: "./config".to_string(),
                world_dir: "./world".to_string(),
                backup_dir: "./backups".to_string(),
            },
            compat: CompatValidation {
                rules_file: "./configs/rules.yaml".to_string(),
                allow_bake_for_permissive_licenses: false,
                auto_apply_rules: true,
                rule_refresh_interval_minutes: 5,
            },
            self_heal: SelfHealValidation {
                entity_freeze_threshold: 3,
                block_entity_freeze_threshold: 3,
                quarantine_dimension: "guardian_hold".to_string(),
                thaw_on_rule_update: true,
                max_frozen_entities: 1000,
                max_frozen_block_entities: 500,
            },
            gpu: GpuValidation {
                enabled: true,
                worker_ipc: "shm".to_string(),
                batch_size_chunks: 64,
                max_cache_size: 1000,
                health_check_interval_seconds: 60,
                fallback_to_cpu: true,
            },
            worldgen: WorldgenValidation {
                gpu_acceleration: true,
                async_generation: true,
                pregeneration_enabled: false,
                pregeneration_radius: 1000,
                pregeneration_threads: 4,
            },
            ha: HighAvailabilityValidation {
                autosave_minutes: 5,
                snapshot_keep: 24,
                blue_green: true,
                rollback_on_failure: true,
                max_restart_attempts: 3,
                restart_delay_seconds: 10,
            },
            monitoring: MonitoringValidation {
                metrics_enabled: true,
                metrics_port: 9090,

                log_level: "INFO".to_string(),
                crash_reporting: true,
            },
            performance: PerformanceValidation {
                tick_time_warning_ms: 50,
                tick_time_critical_ms: 100,
                memory_warning_percent: 80,
                memory_critical_percent: 90,
                gc_optimization: true,
            },
        }
    }
}

// Custom validation functions
fn validate_minecraft_config(config: &MinecraftValidation) -> Result<(), ValidationError> {
    if !["forge", "neoforge", "fabric", "quilt"].contains(&config.loader.to_lowercase().as_str()) {
        return Err(ValidationError::new("invalid_loader"));
    }
    Ok(())
}

fn validate_java_config(config: &JavaValidation) -> Result<(), ValidationError> {
    if !["g1gc-balanced", "g1gc-performance", "zgc", "parallel"].contains(&config.flags.as_str()) {
        return Err(ValidationError::new("invalid_jvm_flags"));
    }
    Ok(())
}

fn validate_path(path: &str) -> Result<(), ValidationError> {
    if path.is_empty() {
        return Err(ValidationError::new("empty_path"));
    }
    if path.contains("..") {
        return Err(ValidationError::new("unsafe_path"));
    }
    Ok(())
}

fn validate_ipc_method(ipc: &str) -> Result<(), ValidationError> {
    if !["shm", "tcp", "unix"].contains(&ipc) {
        return Err(ValidationError::new("invalid_ipc_method"));
    }
    Ok(())
}

fn validate_log_level(level: &str) -> Result<(), ValidationError> {
    if !["TRACE", "DEBUG", "INFO", "WARN", "ERROR"].contains(&level) {
        return Err(ValidationError::new("invalid_log_level"));
    }
    Ok(())
}

// Regex patterns for validation
lazy_static::lazy_static! {
    static ref VERSION_REGEX: regex::Regex = regex::Regex::new(r"^\d+\.\d+(\.\d+)?$").unwrap();
}
