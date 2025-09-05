use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error, debug};
use crate::mod_classification::{ModInfo, ModSide, ConflictSeverity};
use crate::version_manager::{ModLoader, VersionManager};

/// Compatibility issue types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompatibilityIssue {
    VersionMismatch {
        mod_id: String,
        required_version: String,
        available_versions: Vec<String>,
    },
    ModConflict {
        mod1_id: String,
        mod2_id: String,
        reason: String,
        severity: ConflictSeverity,
    },
    MissingDependency {
        mod_id: String,
        required_by: String,
        version_range: String,
    },
    LoaderIncompatibility {
        mod_id: String,
        required_loader: ModLoader,
        current_loader: ModLoader,
    },
    SideMismatch {
        mod_id: String,
        required_side: ModSide,
        current_side: ModSide,
    },
}

/// Compatibility report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityReport {
    pub is_compatible: bool,
    pub issues: Vec<CompatibilityIssue>,
    pub warnings: Vec<CompatibilityIssue>,
    pub recommendations: Vec<String>,
}

/// Modpack compatibility information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackCompatibility {
    pub minecraft_version: String,
    pub loader: ModLoader,
    pub client_mods: Vec<ModInfo>,
    pub server_mods: Vec<ModInfo>,
    pub report: CompatibilityReport,
}

/// Compatibility engine for checking mod conflicts and dependencies
#[derive(Debug, Clone)]
pub struct CompatibilityEngine {
    version_manager: VersionManager,
    conflict_database: HashMap<String, Vec<String>>, // mod_id -> conflicting mods
    dependency_database: HashMap<String, Vec<String>>, // mod_id -> dependencies
}

impl CompatibilityEngine {
    /// Create a new compatibility engine
    pub fn new(version_manager: VersionManager) -> Self {
        let mut engine = Self {
            version_manager,
            conflict_database: HashMap::new(),
            dependency_database: HashMap::new(),
        };
        
        engine.initialize_conflict_database();
        engine.initialize_dependency_database();
        engine
    }

    /// Initialize the conflict database with known conflicts
    fn initialize_conflict_database(&mut self) {
        info!("Initializing mod conflict database...");
        
        // Rendering optimization conflicts
        self.conflict_database.insert("optifine".to_string(), vec![
            "sodium".to_string(),
            "iris".to_string(),
            "canvas".to_string(),
        ]);
        
        // Item viewing conflicts
        self.conflict_database.insert("jei".to_string(), vec![
            "nei".to_string(),
            "waila".to_string(),
        ]);
        
        // World editing conflicts
        self.conflict_database.insert("worldedit".to_string(), vec![
            "voxelsniper".to_string(),
            "fastasyncworldedit".to_string(),
        ]);
        
        // Performance monitoring conflicts
        self.conflict_database.insert("spark".to_string(), vec![
            "laggoggles".to_string(),
            "tickprofiler".to_string(),
        ]);
        
        // Chat formatting conflicts
        self.conflict_database.insert("chatflow".to_string(), vec![
            "chatcontrol".to_string(),
            "luckperms".to_string(),
        ]);
        
        // Inventory management conflicts
        self.conflict_database.insert("inventorytweaks".to_string(), vec![
            "quark".to_string(),
            "inventoryprofiles".to_string(),
        ]);
        
        info!("Mod conflict database initialized with {} entries", self.conflict_database.len());
    }

    /// Initialize the dependency database with known dependencies
    fn initialize_dependency_database(&mut self) {
        info!("Initializing mod dependency database...");
        
        // Common dependencies
        self.dependency_database.insert("jei".to_string(), vec![
            "forge".to_string(),
            "fabric".to_string(),
        ]);
        
        self.dependency_database.insert("waila".to_string(), vec![
            "forge".to_string(),
        ]);
        
        self.dependency_database.insert("optifine".to_string(), vec![
            "forge".to_string(),
        ]);
        
        self.dependency_database.insert("sodium".to_string(), vec![
            "fabric".to_string(),
        ]);
        
        self.dependency_database.insert("iris".to_string(), vec![
            "sodium".to_string(),
            "fabric".to_string(),
        ]);
        
        self.dependency_database.insert("canvas".to_string(), vec![
            "fabric".to_string(),
        ]);
        
        info!("Mod dependency database initialized with {} entries", self.dependency_database.len());
    }

    /// Check compatibility of a modpack
    pub async fn check_modpack_compatibility(&self, modpack: &ModpackCompatibility) -> Result<CompatibilityReport> {
        let mut report = CompatibilityReport {
            is_compatible: true,
            issues: Vec::new(),
            warnings: Vec::new(),
            recommendations: Vec::new(),
        };
        
        info!("Checking modpack compatibility for {} mods", 
              modpack.client_mods.len() + modpack.server_mods.len());
        
        // Check Minecraft version compatibility
        self.check_minecraft_version_compatibility(&modpack.minecraft_version, &mut report).await?;
        
        // Check loader compatibility
        self.check_loader_compatibility(&modpack.loader, &modpack.client_mods, &modpack.server_mods, &mut report).await?;
        
        // Check mod conflicts
        self.check_mod_conflicts(&modpack.client_mods, &modpack.server_mods, &mut report).await?;
        
        // Check dependencies
        self.check_dependencies(&modpack.client_mods, &modpack.server_mods, &mut report).await?;
        
        // Check side compatibility
        self.check_side_compatibility(&modpack.client_mods, &modpack.server_mods, &mut report).await?;
        
        // Generate recommendations
        self.generate_recommendations(&modpack, &mut report).await?;
        
        // Determine overall compatibility
        report.is_compatible = report.issues.is_empty();
        
        info!("Modpack compatibility check completed: {} issues, {} warnings", 
              report.issues.len(), report.warnings.len());
        
        Ok(report)
    }

    /// Check Minecraft version compatibility
    async fn check_minecraft_version_compatibility(&self, version: &str, report: &mut CompatibilityReport) -> Result<()> {
        let versions = self.version_manager.get_minecraft_versions();
        if !versions.iter().any(|v| v.id == version) {
            report.issues.push(CompatibilityIssue::VersionMismatch {
                mod_id: "minecraft".to_string(),
                required_version: version.to_string(),
                available_versions: self.version_manager.get_minecraft_versions()
                    .into_iter()
                    .map(|v| v.id)
                    .collect(),
            });
        }
        Ok(())
    }

    /// Check loader compatibility
    async fn check_loader_compatibility(&self, loader: &ModLoader, client_mods: &[ModInfo], server_mods: &[ModInfo], report: &mut CompatibilityReport) -> Result<()> {
        let all_mods = client_mods.iter().chain(server_mods.iter());
        
        for mod_info in all_mods {
            if let Some(required_loaders) = mod_info.loader_versions.get(&loader.to_string()) {
                if required_loaders.is_empty() {
                    report.issues.push(CompatibilityIssue::LoaderIncompatibility {
                        mod_id: mod_info.id.clone(),
                        required_loader: loader.clone(),
                        current_loader: loader.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    /// Check mod conflicts
    async fn check_mod_conflicts(&self, client_mods: &[ModInfo], server_mods: &[ModInfo], report: &mut CompatibilityReport) -> Result<()> {
        let all_mods = client_mods.iter().chain(server_mods.iter()).collect::<Vec<_>>();
        
        for (i, mod1) in all_mods.iter().enumerate() {
            for mod2 in &all_mods[i+1..] {
                if let Some(conflicts) = self.conflict_database.get(&mod1.id.to_lowercase()) {
                    if conflicts.contains(&mod2.id.to_lowercase()) {
                        report.issues.push(CompatibilityIssue::ModConflict {
                            mod1_id: mod1.id.clone(),
                            mod2_id: mod2.id.clone(),
                            reason: format!("Known conflict between {} and {}", mod1.name, mod2.name),
                            severity: ConflictSeverity::Critical,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// Check mod dependencies
    async fn check_dependencies(&self, client_mods: &[ModInfo], server_mods: &[ModInfo], report: &mut CompatibilityReport) -> Result<()> {
        let all_mods = client_mods.iter().chain(server_mods.iter()).collect::<Vec<_>>();
        let mod_ids: std::collections::HashSet<String> = all_mods.iter().map(|m| m.id.clone()).collect();
        
        for mod_info in all_mods {
            for dependency in &mod_info.dependencies {
                if !mod_ids.contains(&dependency.mod_id) {
                    report.issues.push(CompatibilityIssue::MissingDependency {
                        mod_id: dependency.mod_id.clone(),
                        required_by: mod_info.id.clone(),
                        version_range: dependency.version_range.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    /// Check side compatibility
    async fn check_side_compatibility(&self, client_mods: &[ModInfo], server_mods: &[ModInfo], report: &mut CompatibilityReport) -> Result<()> {
        // Check that client mods are appropriate for client side
        for mod_info in client_mods {
            if mod_info.side == ModSide::Server {
                report.warnings.push(CompatibilityIssue::SideMismatch {
                    mod_id: mod_info.id.clone(),
                    required_side: ModSide::Client,
                    current_side: ModSide::Server,
                });
            }
        }
        
        // Check that server mods are appropriate for server side
        for mod_info in server_mods {
            if mod_info.side == ModSide::Client {
                report.warnings.push(CompatibilityIssue::SideMismatch {
                    mod_id: mod_info.id.clone(),
                    required_side: ModSide::Server,
                    current_side: ModSide::Client,
                });
            }
        }
        Ok(())
    }

    /// Generate recommendations for improving compatibility
    async fn generate_recommendations(&self, modpack: &ModpackCompatibility, report: &mut CompatibilityReport) -> Result<()> {
        // Recommend alternative mods for conflicts
        for issue in &report.issues {
            if let CompatibilityIssue::ModConflict { mod1_id, mod2_id, .. } = issue {
                if mod1_id.to_lowercase() == "optifine" && mod2_id.to_lowercase() == "sodium" {
                    report.recommendations.push(
                        "Consider using Sodium instead of OptiFine for better Fabric compatibility".to_string()
                    );
                } else if mod1_id.to_lowercase() == "jei" && mod2_id.to_lowercase() == "nei" {
                    report.recommendations.push(
                        "Consider using JEI instead of NEI for better modern mod support".to_string()
                    );
                }
            }
        }
        
        // Recommend performance optimizations
        if modpack.client_mods.len() > 50 {
            report.recommendations.push(
                "Consider reducing the number of client mods for better performance".to_string()
            );
        }
        
        if modpack.server_mods.len() > 100 {
            report.recommendations.push(
                "Consider reducing the number of server mods for better performance".to_string()
            );
        }
        
        // Recommend version updates
        let latest_version = self.version_manager.get_latest_version(&crate::version_manager::ReleaseType::Release);
        if let Some(latest) = latest_version {
            if modpack.minecraft_version != latest.id {
                report.recommendations.push(
                    format!("Consider updating to Minecraft {} for better mod support", latest.id)
                );
            }
        }
        
        Ok(())
    }

    /// Check if two mods are compatible
    pub fn are_mods_compatible(&self, mod1: &ModInfo, mod2: &ModInfo) -> bool {
        // Check direct conflicts
        if let Some(conflicts) = self.conflict_database.get(&mod1.id.to_lowercase()) {
            if conflicts.contains(&mod2.id.to_lowercase()) {
                return false;
            }
        }
        
        if let Some(conflicts) = self.conflict_database.get(&mod2.id.to_lowercase()) {
            if conflicts.contains(&mod1.id.to_lowercase()) {
                return false;
            }
        }
        
        // Check version compatibility
        let mod1_versions = &mod1.minecraft_versions;
        let mod2_versions = &mod2.minecraft_versions;
        
        // Check if they have any common Minecraft versions
        let has_common_version = mod1_versions.iter().any(|v| mod2_versions.contains(v));
        if !has_common_version {
            return false;
        }
        
        // Check loader compatibility
        let mod1_loaders: std::collections::HashSet<String> = mod1.loader_versions.keys().cloned().collect();
        let mod2_loaders: std::collections::HashSet<String> = mod2.loader_versions.keys().cloned().collect();
        
        let has_common_loader = mod1_loaders.intersection(&mod2_loaders).next().is_some();
        if !has_common_loader {
            return false;
        }
        
        true
    }

    /// Get conflicting mods for a given mod
    pub fn get_conflicting_mods(&self, mod_id: &str) -> Vec<String> {
        self.conflict_database
            .get(&mod_id.to_lowercase())
            .cloned()
            .unwrap_or_default()
    }

    /// Get dependencies for a given mod
    pub fn get_dependencies(&self, mod_id: &str) -> Vec<String> {
        self.dependency_database
            .get(&mod_id.to_lowercase())
            .cloned()
            .unwrap_or_default()
    }

    /// Add a new conflict to the database
    pub fn add_conflict(&mut self, mod1_id: String, mod2_id: String) {
        self.conflict_database
            .entry(mod1_id.clone())
            .or_insert_with(Vec::new)
            .push(mod2_id.clone());
        
        self.conflict_database
            .entry(mod2_id)
            .or_insert_with(Vec::new)
            .push(mod1_id);
    }

    /// Add a new dependency to the database
    pub fn add_dependency(&mut self, mod_id: String, dependency: String) {
        self.dependency_database
            .entry(mod_id)
            .or_insert_with(Vec::new)
            .push(dependency);
    }

    /// Get compatibility statistics
    pub fn get_compatibility_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        
        stats.insert("total_conflicts".to_string(), 
                    serde_json::Value::Number(self.conflict_database.len().into()));
        
        stats.insert("total_dependencies".to_string(), 
                    serde_json::Value::Number(self.dependency_database.len().into()));
        
        let total_conflict_pairs: usize = self.conflict_database.values()
            .map(|v| v.len())
            .sum();
        
        stats.insert("total_conflict_pairs".to_string(), 
                    serde_json::Value::Number(total_conflict_pairs.into()));
        
        stats
    }
}

impl Default for CompatibilityEngine {
    fn default() -> Self {
        Self::new(VersionManager::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mod_classification::{ModCategory, ModSource};
    use std::collections::HashMap;

    fn create_test_mod(id: &str, name: &str, side: ModSide) -> ModInfo {
        ModInfo {
            id: id.to_string(),
            name: name.to_string(),
            description: Some("Test mod".to_string()),
            version: "1.0.0".to_string(),
            minecraft_versions: vec!["1.21.1".to_string()],
            loader_versions: HashMap::new(),
            side,
            category: ModCategory::Utility,
            dependencies: Vec::new(),
            conflicts: Vec::new(),
            source: ModSource::CurseForge,
            download_url: "https://example.com".to_string(),
            file_size: 1024,
            sha256: "abc123".to_string(),
            last_updated: chrono::Utc::now(),
            download_count: Some(1000),
            rating: Some(4.0),
            tags: vec!["test".to_string()],
        }
    }

    #[tokio::test]
    async fn test_modpack_compatibility_check() {
        let version_manager = VersionManager::new();
        let engine = CompatibilityEngine::new(version_manager);
        
        let modpack = ModpackCompatibility {
            minecraft_version: "1.21.1".to_string(),
            loader: ModLoader::Forge { version: "latest".to_string() },
            client_mods: vec![
                create_test_mod("optifine", "OptiFine", ModSide::Client),
                create_test_mod("sodium", "Sodium", ModSide::Client),
            ],
            server_mods: vec![],
            report: CompatibilityReport {
                is_compatible: true,
                issues: Vec::new(),
                warnings: Vec::new(),
                recommendations: Vec::new(),
            },
        };
        
        let report = engine.check_modpack_compatibility(&modpack).await.unwrap();
        assert!(!report.is_compatible);
        assert!(!report.issues.is_empty());
    }

    #[test]
    fn test_mod_compatibility() {
        let version_manager = VersionManager::new();
        let engine = CompatibilityEngine::new(version_manager);
        
        let mod1 = create_test_mod("optifine", "OptiFine", ModSide::Client);
        let mod2 = create_test_mod("sodium", "Sodium", ModSide::Client);
        
        assert!(!engine.are_mods_compatible(&mod1, &mod2));
    }

    #[test]
    fn test_conflict_database() {
        let version_manager = VersionManager::new();
        let engine = CompatibilityEngine::new(version_manager);
        
        let conflicts = engine.get_conflicting_mods("optifine");
        assert!(conflicts.contains(&"sodium".to_string()));
    }
}
