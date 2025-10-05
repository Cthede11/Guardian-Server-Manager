// Mod management module placeholder
// This will be implemented later

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Mod operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModOperation {
    Install,
    Uninstall,
    Update,
    Enable,
    Disable,
    Configure,
}

/// Mod installation plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInstallationPlan {
    pub mod_id: String,
    pub operation: ModOperation,
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub dependencies: Vec<String>,
    pub conflicts: Vec<String>,
    pub backup_required: bool,
    pub rollback_plan: Option<String>,
}