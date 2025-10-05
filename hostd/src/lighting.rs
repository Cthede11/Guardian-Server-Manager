// Lighting module placeholder
// This will be implemented later

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationLevel {
    Low,
    Medium,
    High,
    Ultra,
    Balanced,
}

/// Lighting job configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightingJob {
    pub id: String,
    pub server_id: String,
    pub world_path: PathBuf,
    pub optimization_level: OptimizationLevel,
    pub progress: f32,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Lighting settings for a server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightingSettings {
    pub enabled: bool,
    pub optimization_level: OptimizationLevel,
    pub default_level: OptimizationLevel,
    pub auto_optimize: bool,
    pub gpu_acceleration: bool,
    pub auto_optimize_after_pregeneration: bool,
    pub preserve_lighting_data: bool,
    pub max_concurrent_jobs: u32,
    pub chunk_batch_size: u32,
    pub schedule: Option<String>,
    pub chunk_radius: u32,
    pub priority: u8,
}
