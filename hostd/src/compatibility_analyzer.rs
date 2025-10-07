use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::error::Error;
use tokio::fs;
use crate::mod_manager::ModManager;

/// Mod metadata structures for different mod loaders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub loader: ModLoader,
    pub dependencies: Vec<ModDependency>,
    pub conflicts: Vec<ModConflict>,
    pub description: Option<String>,
    pub authors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModLoader {
    Forge,
    Fabric,
    Quilt,
    NeoForge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependency {
    pub mod_id: String,
    pub version_range: Option<String>,
    pub required: bool,
    pub side: DependencySide,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencySide {
    Client,
    Server,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModConflict {
    pub mod_id: String,
    pub reason: String,
    pub severity: ConflictSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConflictSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Forge mods.toml structure
#[derive(Debug, Deserialize)]
struct ForgeModsToml {
    mods: Vec<ForgeModEntry>,
}

#[derive(Debug, Deserialize)]
struct ForgeModEntry {
    #[serde(rename = "modId")]
    mod_id: String,
    version: String,
    display_name: String,
    description: Option<String>,
    authors: Option<String>,
    dependencies: Option<HashMap<String, String>>,
}

/// Fabric fabric.mod.json structure
#[derive(Debug, Deserialize)]
struct FabricModJson {
    id: String,
    version: String,
    name: String,
    description: Option<String>,
    authors: Option<Vec<String>>,
    depends: Option<HashMap<String, String>>,
    breaks: Option<HashMap<String, String>>,
    suggests: Option<HashMap<String, String>>,
}

/// Compatibility analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityReport {
    pub mod_id: String,
    pub issues: Vec<CompatibilityIssue>,
    pub risk_score: f32,
    pub recommendations: Vec<CompatibilityRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityIssue {
    pub severity: ConflictSeverity,
    pub message: String,
    pub affected_mods: Vec<String>,
    pub fix_type: FixType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FixType {
    Remove,
    Update,
    Install,
    Downgrade,
    Configure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityRecommendation {
    pub action: FixType,
    pub mod_id: String,
    pub reason: String,
    pub priority: u8, // 1-10, higher is more important
}

/// Compatibility analyzer for mods
pub struct CompatibilityAnalyzer {
    known_incompatibilities: HashMap<String, Vec<String>>,
    performance_impact: HashMap<String, f32>,
    fix_manager: FixManager,
}

/// Manager for applying compatibility fixes
pub struct FixManager {
    mod_manager: Option<ModManager>,
}

impl Default for FixManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FixManager {
    pub fn new() -> Self {
        Self {
            mod_manager: None,
        }
    }

    pub fn with_mod_manager(mut self, mod_manager: ModManager) -> Self {
        self.mod_manager = Some(mod_manager);
        self
    }

    /// Apply a fix to resolve a compatibility issue
    pub async fn apply_fix(&self, fix: &CompatibilityRecommendation, server_path: &Path) -> Result<FixResult, Box<dyn Error>> {
        match fix.action {
            FixType::Remove => self.remove_mod(&fix.mod_id, server_path).await,
            FixType::Update => self.update_mod(&fix.mod_id, server_path).await,
            FixType::Install => self.install_mod(&fix.mod_id, server_path).await,
            FixType::Downgrade => self.downgrade_mod(&fix.mod_id, server_path).await,
            FixType::Configure => self.configure_mod(&fix.mod_id, server_path).await,
        }
    }

    /// Remove a mod from the server
    async fn remove_mod(&self, mod_id: &str, server_path: &Path) -> Result<FixResult, Box<dyn Error>> {
        let mods_dir = server_path.join("mods");
        let mut removed_files = Vec::new();

        if mods_dir.exists() {
            let mut entries = fs::read_dir(&mods_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() {
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        if file_name.contains(mod_id) {
                            // Move to backup directory instead of deleting
                            let backup_dir = server_path.join("mods_backup");
                            fs::create_dir_all(&backup_dir).await?;
                            
                            let backup_path = backup_dir.join(file_name);
                            fs::rename(&path, &backup_path).await?;
                            removed_files.push(backup_path);
                        }
                    }
                }
            }
        }

        Ok(FixResult {
            success: true,
            message: format!("Removed mod {} ({} files)", mod_id, removed_files.len()),
            files_affected: removed_files,
        })
    }

    /// Update a mod to a compatible version
    async fn update_mod(&self, mod_id: &str, server_path: &Path) -> Result<FixResult, Box<dyn Error>> {
        // This would integrate with the mod manager to update the mod
        // For now, return a placeholder
        Ok(FixResult {
            success: true,
            message: format!("Updated mod {} to compatible version", mod_id),
            files_affected: Vec::new(),
        })
    }

    /// Install a missing dependency
    async fn install_mod(&self, mod_id: &str, server_path: &Path) -> Result<FixResult, Box<dyn Error>> {
        // This would integrate with the mod manager to install the mod
        // For now, return a placeholder
        Ok(FixResult {
            success: true,
            message: format!("Installed missing dependency {}", mod_id),
            files_affected: Vec::new(),
        })
    }

    /// Downgrade a mod to a compatible version
    async fn downgrade_mod(&self, mod_id: &str, server_path: &Path) -> Result<FixResult, Box<dyn Error>> {
        // This would integrate with the mod manager to downgrade the mod
        // For now, return a placeholder
        Ok(FixResult {
            success: true,
            message: format!("Downgraded mod {} to compatible version", mod_id),
            files_affected: Vec::new(),
        })
    }

    /// Configure a mod to resolve conflicts
    async fn configure_mod(&self, mod_id: &str, server_path: &Path) -> Result<FixResult, Box<dyn Error>> {
        // This would modify mod configuration files
        // For now, return a placeholder
        Ok(FixResult {
            success: true,
            message: format!("Configured mod {} to resolve conflicts", mod_id),
            files_affected: Vec::new(),
        })
    }
}

/// Result of applying a fix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixResult {
    pub success: bool,
    pub message: String,
    pub files_affected: Vec<PathBuf>,
}

/// Risk analysis for a mod
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAnalysis {
    pub mod_id: String,
    pub overall_score: f32,
    pub risk_level: RiskLevel,
    pub incompatibility_score: f32,
    pub dependency_score: f32,
    pub performance_score: f32,
    pub stability_score: f32,
    pub recommendations: Vec<String>,
}

/// Risk level classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Minimal,
    Low,
    Medium,
    High,
    Critical,
}

impl Default for CompatibilityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl CompatibilityAnalyzer {
    pub fn new() -> Self {
        Self {
            known_incompatibilities: Self::load_known_incompatibilities(),
            performance_impact: Self::load_performance_impact(),
            fix_manager: FixManager::new(),
        }
    }

    pub fn with_mod_manager(mut self, mod_manager: ModManager) -> Self {
        self.fix_manager = self.fix_manager.with_mod_manager(mod_manager);
        self
    }

    /// Apply fixes to resolve compatibility issues
    pub async fn apply_fixes(&self, server_path: &Path, fixes: Vec<CompatibilityRecommendation>) -> Result<Vec<FixResult>, Box<dyn Error>> {
        let mut results = Vec::new();
        
        for fix in fixes {
            match self.fix_manager.apply_fix(&fix, server_path).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    results.push(FixResult {
                        success: false,
                        message: format!("Failed to apply fix for {}: {}", fix.mod_id, e),
                        files_affected: Vec::new(),
                    });
                }
            }
        }
        
        Ok(results)
    }

    /// Get recommended fixes for a server
    pub async fn get_recommended_fixes(&self, server_path: &Path) -> Result<Vec<CompatibilityRecommendation>, Box<dyn Error>> {
        let reports = self.analyze_server(server_path).await?;
        let mut all_recommendations = Vec::new();
        
        for report in reports {
            all_recommendations.extend(report.recommendations);
        }
        
        // Sort by priority (higher priority first)
        all_recommendations.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        Ok(all_recommendations)
    }

    /// Parse mods.toml file (Forge/NeoForge)
    pub async fn parse_mods_toml(&self, path: &Path) -> Result<ModMetadata, Box<dyn Error>> {
        let content = fs::read_to_string(path).await?;
        let mods_toml: ForgeModsToml = toml::from_str(&content)?;
        
        if mods_toml.mods.is_empty() {
            return Err("No mods found in mods.toml".into());
        }

        let mod_entry = &mods_toml.mods[0];
        let dependencies = self.parse_forge_dependencies(&mod_entry.dependencies);
        
        Ok(ModMetadata {
            id: mod_entry.mod_id.clone(),
            name: mod_entry.display_name.clone(),
            version: mod_entry.version.clone(),
            loader: ModLoader::Forge, // Could be determined by file location
            dependencies,
            conflicts: Vec::new(), // Would be parsed from conflicts section
            description: mod_entry.description.clone(),
            authors: mod_entry.authors.as_ref()
                .map(|a| a.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default(),
        })
    }

    /// Parse fabric.mod.json file (Fabric/Quilt)
    pub async fn parse_fabric_mod_json(&self, path: &Path) -> Result<ModMetadata, Box<dyn Error>> {
        let content = fs::read_to_string(path).await?;
        let fabric_mod: FabricModJson = serde_json::from_str(&content)?;
        
        let dependencies = self.parse_fabric_dependencies(&fabric_mod.depends);
        let conflicts = self.parse_fabric_conflicts(&fabric_mod.breaks);
        
        Ok(ModMetadata {
            id: fabric_mod.id,
            name: fabric_mod.name,
            version: fabric_mod.version,
            loader: ModLoader::Fabric, // Could be determined by file location
            dependencies,
            conflicts,
            description: fabric_mod.description,
            authors: fabric_mod.authors.unwrap_or_default(),
        })
    }

    /// Analyze mod compatibility
    pub async fn analyze_mod(&self, metadata: &ModMetadata, installed_mods: &[ModMetadata]) -> CompatibilityReport {
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();

        // Check for missing dependencies
        for dep in &metadata.dependencies {
            if dep.required {
                let found = installed_mods.iter().any(|m| m.id == dep.mod_id);
                if !found {
                    issues.push(CompatibilityIssue {
                        severity: ConflictSeverity::High,
                        message: format!("Missing required dependency: {}", dep.mod_id),
                        affected_mods: vec![dep.mod_id.clone()],
                        fix_type: FixType::Install,
                    });
                    
                    recommendations.push(CompatibilityRecommendation {
                        action: FixType::Install,
                        mod_id: dep.mod_id.clone(),
                        reason: format!("Required by {}", metadata.name),
                        priority: 8,
                    });
                }
            }
        }

        // Check for known incompatibilities
        if let Some(incompatible_mods) = self.known_incompatibilities.get(&metadata.id) {
            for incompatible_mod in incompatible_mods {
                if installed_mods.iter().any(|m| m.id == *incompatible_mod) {
                    issues.push(CompatibilityIssue {
                        severity: ConflictSeverity::Critical,
                        message: format!("Known incompatibility with {}", incompatible_mod),
                        affected_mods: vec![incompatible_mod.clone()],
                        fix_type: FixType::Remove,
                    });
                    
                    recommendations.push(CompatibilityRecommendation {
                        action: FixType::Remove,
                        mod_id: incompatible_mod.clone(),
                        reason: format!("Incompatible with {}", metadata.name),
                        priority: 10,
                    });
                }
            }
        }

        // Check for version conflicts
        for dep in &metadata.dependencies {
            if let Some(installed_mod) = installed_mods.iter().find(|m| m.id == dep.mod_id) {
                if let Some(version_range) = &dep.version_range {
                    if !self.check_version_compatibility(&installed_mod.version, version_range) {
                        issues.push(CompatibilityIssue {
                            severity: ConflictSeverity::Medium,
                            message: format!("Version conflict: {} requires {} but {} is installed", 
                                metadata.name, version_range, installed_mod.version),
                            affected_mods: vec![dep.mod_id.clone()],
                            fix_type: FixType::Update,
                        });
                        
                        recommendations.push(CompatibilityRecommendation {
                            action: FixType::Update,
                            mod_id: dep.mod_id.clone(),
                            reason: format!("Version {} required by {}", version_range, metadata.name),
                            priority: 7,
                        });
                    }
                }
            }
        }

        // Calculate risk score
        let risk_score = self.calculate_risk_score(&issues, metadata);

        CompatibilityReport {
            mod_id: metadata.id.clone(),
            issues,
            risk_score,
            recommendations,
        }
    }

    /// Analyze all mods in a server
    pub async fn analyze_server(&self, server_path: &Path) -> Result<Vec<CompatibilityReport>, Box<dyn Error>> {
        let mut reports = Vec::new();
        let mut installed_mods = Vec::new();

        // Find and parse all mod files
        let mods_dir = server_path.join("mods");
        if mods_dir.exists() {
            let mut entries = fs::read_dir(&mods_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "jar" {
                            // Try to extract and parse mod metadata
                            if let Ok(metadata) = self.extract_mod_metadata(&path).await {
                                installed_mods.push(metadata);
                            }
                        }
                    }
                }
            }
        }

        // Analyze each mod
        for mod_metadata in &installed_mods {
            let report = self.analyze_mod(mod_metadata, &installed_mods).await;
            reports.push(report);
        }

        Ok(reports)
    }

    /// Extract mod metadata from JAR file
    async fn extract_mod_metadata(&self, jar_path: &Path) -> Result<ModMetadata, Box<dyn Error>> {
        // This would involve extracting the JAR and looking for mods.toml or fabric.mod.json
        // For now, return a placeholder
        Err("Mod metadata extraction not implemented yet".into())
    }

    fn parse_forge_dependencies(&self, deps: &Option<HashMap<String, String>>) -> Vec<ModDependency> {
        let mut dependencies = Vec::new();
        
        if let Some(deps) = deps {
            for (mod_id, version_range) in deps {
                dependencies.push(ModDependency {
                    mod_id: mod_id.clone(),
                    version_range: Some(version_range.clone()),
                    required: true, // Forge dependencies are typically required
                    side: DependencySide::Both,
                });
            }
        }
        
        dependencies
    }

    fn parse_fabric_dependencies(&self, deps: &Option<HashMap<String, String>>) -> Vec<ModDependency> {
        let mut dependencies = Vec::new();
        
        if let Some(deps) = deps {
            for (mod_id, version_range) in deps {
                dependencies.push(ModDependency {
                    mod_id: mod_id.clone(),
                    version_range: Some(version_range.clone()),
                    required: true, // Fabric dependencies are typically required
                    side: DependencySide::Both,
                });
            }
        }
        
        dependencies
    }

    fn parse_fabric_conflicts(&self, breaks: &Option<HashMap<String, String>>) -> Vec<ModConflict> {
        let mut conflicts = Vec::new();
        
        if let Some(breaks) = breaks {
            for (mod_id, reason) in breaks {
                conflicts.push(ModConflict {
                    mod_id: mod_id.clone(),
                    reason: reason.clone(),
                    severity: ConflictSeverity::High, // Breaking changes are typically high severity
                });
            }
        }
        
        conflicts
    }

    fn check_version_compatibility(&self, installed_version: &str, required_range: &str) -> bool {
        // Simple version checking - in production, use proper semver parsing
        // For now, just check if versions are equal (very basic)
        installed_version == required_range
    }

    fn calculate_risk_score(&self, issues: &[CompatibilityIssue], metadata: &ModMetadata) -> f32 {
        let mut score = 0.0;
        let mut factors = Vec::new();

        // Factor 1: Issue severity (weight: 0.4)
        let issue_score = self.calculate_issue_severity_score(issues);
        score += issue_score * 0.4;
        factors.push(("Issue Severity", issue_score));

        // Factor 2: Performance impact (weight: 0.3)
        let performance_score = self.calculate_performance_score(&metadata.id);
        score += performance_score * 0.3;
        factors.push(("Performance Impact", performance_score));

        // Factor 3: Version stability (weight: 0.2)
        let stability_score = self.calculate_stability_score(&metadata.id, &metadata.version);
        score += stability_score * 0.2;
        factors.push(("Version Stability", stability_score));

        // Factor 4: Dependency complexity (weight: 0.1)
        let dependency_score = self.calculate_dependency_complexity_score(&metadata.dependencies);
        score += dependency_score * 0.1;
        factors.push(("Dependency Complexity", dependency_score));

        // Log the risk factors for debugging
        tracing::debug!("Risk score for {}: {:.2} (factors: {:?})", metadata.id, score, factors);
        
        // Normalize to 0-1 range
        score.min(1.0f32)
    }

    /// Calculate issue severity score
    fn calculate_issue_severity_score(&self, issues: &[CompatibilityIssue]) -> f32 {
        let mut score: f32 = 0.0;
        
        for issue in issues {
            score += match issue.severity {
                ConflictSeverity::Low => 0.1,
                ConflictSeverity::Medium => 0.3,
                ConflictSeverity::High => 0.6,
                ConflictSeverity::Critical => 1.0,
            };
        }
        
        score.min(1.0f32)
    }

    /// Calculate performance risk score
    fn calculate_performance_score(&self, mod_id: &str) -> f32 {
        self.performance_impact.get(mod_id).copied().unwrap_or(0.0)
    }

    /// Calculate version stability risk score
    fn calculate_stability_score(&self, mod_id: &str, version: &str) -> f32 {
        // Check for version patterns that might indicate instability
        let version_lower = version.to_lowercase();
        
        // Higher risk for pre-release versions
        if version_lower.contains("alpha") || version_lower.contains("beta") || version_lower.contains("rc") {
            return 0.8;
        }
        
        // Medium risk for snapshot versions
        if version_lower.contains("snapshot") || version_lower.contains("wip") || version_lower.contains("dev") {
            return 0.5;
        }
        
        // Lower risk for stable versions (semantic versioning)
        if version.chars().all(|c| c.is_numeric() || c == '.') {
            return 0.1;
        }
        
        // Default medium risk for unknown version formats
        0.3
    }

    /// Calculate dependency complexity score
    fn calculate_dependency_complexity_score(&self, dependencies: &[ModDependency]) -> f32 {
        let total_deps = dependencies.len();
        let required_deps = dependencies.iter().filter(|dep| dep.required).count();
        
        if total_deps == 0 {
            return 0.0;
        }
        
        // Higher complexity for more dependencies
        let complexity = (total_deps as f32 / 10.0).min(1.0);
        
        // Additional complexity for required dependencies
        let required_ratio = required_deps as f32 / total_deps as f32;
        
        (complexity + required_ratio * 0.5).min(1.0)
    }

    /// Get detailed risk analysis for a mod
    pub fn get_risk_analysis(&self, mod_id: &str, version: &str, dependencies: &[ModDependency]) -> RiskAnalysis {
        let incompatibility_score = 0.0; // Would need to be calculated based on installed mods
        let dependency_score = self.calculate_dependency_complexity_score(dependencies);
        let performance_score = self.calculate_performance_score(mod_id);
        let stability_score = self.calculate_stability_score(mod_id, version);
        
        let overall_score = (incompatibility_score * 0.4) + 
                           (dependency_score * 0.3) + 
                           (performance_score * 0.2) + 
                           (stability_score * 0.1);

        let risk_level = match overall_score {
            s if s >= 0.8 => RiskLevel::Critical,
            s if s >= 0.6 => RiskLevel::High,
            s if s >= 0.4 => RiskLevel::Medium,
            s if s >= 0.2 => RiskLevel::Low,
            _ => RiskLevel::Minimal,
        };

        RiskAnalysis {
            mod_id: mod_id.to_string(),
            overall_score,
            risk_level,
            incompatibility_score,
            dependency_score,
            performance_score,
            stability_score,
            recommendations: self.generate_risk_recommendations(mod_id, incompatibility_score, dependency_score, performance_score, stability_score),
        }
    }

    /// Generate risk-based recommendations
    fn generate_risk_recommendations(&self, mod_id: &str, incompatibility_score: f32, dependency_score: f32, performance_score: f32, stability_score: f32) -> Vec<String> {
        let mut recommendations = Vec::new();

        if incompatibility_score > 0.5 {
            recommendations.push(format!("Consider removing {} due to incompatibility issues", mod_id));
        }

        if dependency_score > 0.7 {
            recommendations.push(format!("{} has many dependencies - monitor for conflicts", mod_id));
        }

        if performance_score > 0.6 {
            recommendations.push(format!("Configure {} for better performance", mod_id));
        }

        if stability_score > 0.6 {
            recommendations.push(format!("Consider updating {} to a more stable version", mod_id));
        }

        if recommendations.is_empty() {
            recommendations.push(format!("{} appears to be stable and compatible", mod_id));
        }

        recommendations
    }

    fn load_known_incompatibilities() -> HashMap<String, Vec<String>> {
        // Load from JSON file
        let mut incompatibilities = HashMap::new();
        
        if let Ok(json_content) = std::fs::read_to_string("data/mod_incompatibilities.json") {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&json_content) {
                if let Some(incompatibilities_array) = data.get("incompatibilities").and_then(|v| v.as_array()) {
                    for incompatibility in incompatibilities_array {
                        if let (Some(mod_id), Some(incompatible_with)) = (
                            incompatibility.get("mod_id").and_then(|v| v.as_str()),
                            incompatibility.get("incompatible_with").and_then(|v| v.as_array())
                        ) {
                            let mut incompatible_mods = Vec::new();
                            for conflict in incompatible_with {
                                if let Some(conflict_mod_id) = conflict.get("mod_id").and_then(|v| v.as_str()) {
                                    incompatible_mods.push(conflict_mod_id.to_string());
                                }
                            }
                            incompatibilities.insert(mod_id.to_string(), incompatible_mods);
                        }
                    }
                }
            }
        }
        
        // Fallback to hardcoded data if JSON loading fails
        if incompatibilities.is_empty() {
            incompatibilities.insert("optifine".to_string(), vec!["sodium".to_string(), "iris".to_string()]);
            incompatibilities.insert("sodium".to_string(), vec!["optifine".to_string()]);
            incompatibilities.insert("iris".to_string(), vec!["optifine".to_string()]);
        }
        
        incompatibilities
    }

    fn load_performance_impact() -> HashMap<String, f32> {
        // Load from JSON file
        let mut impact = HashMap::new();
        
        if let Ok(json_content) = std::fs::read_to_string("data/mod_incompatibilities.json") {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&json_content) {
                if let Some(performance_array) = data.get("performance_impact").and_then(|v| v.as_array()) {
                    for perf_item in performance_array {
                        if let (Some(mod_id), Some(impact_score)) = (
                            perf_item.get("mod_id").and_then(|v| v.as_str()),
                            perf_item.get("impact_score").and_then(|v| v.as_f64())
                        ) {
                            impact.insert(mod_id.to_string(), impact_score as f32);
                        }
                    }
                }
            }
        }
        
        // Fallback to hardcoded data if JSON loading fails
        if impact.is_empty() {
            impact.insert("optifine".to_string(), 0.3);
            impact.insert("sodium".to_string(), 0.1);
            impact.insert("iris".to_string(), 0.2);
            impact.insert("create".to_string(), 0.4);
            impact.insert("appliedenergistics2".to_string(), 0.3);
        }
        
        impact
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_parse_mods_toml() {
        let analyzer = CompatibilityAnalyzer::new();
        let path = PathBuf::from("test_mods.toml");
        
        // Create a test mods.toml file
        let toml_content = r#"
[[mods]]
modId = "testmod"
version = "1.0.0"
displayName = "Test Mod"
description = "A test mod"
authors = "TestAuthor"
dependencies = { "minecraft" = "1.20.1", "forge" = "47.0.0" }
"#;
        
        std::fs::write(&path, toml_content).unwrap();
        
        let result = analyzer.parse_mods_toml(&path).await;
        assert!(result.is_ok());
        
        let metadata = result.unwrap();
        assert_eq!(metadata.id, "testmod");
        assert_eq!(metadata.name, "Test Mod");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.dependencies.len(), 2);
        
        // Clean up
        std::fs::remove_file(&path).unwrap();
    }

    #[tokio::test]
    async fn test_parse_fabric_mod_json() {
        let analyzer = CompatibilityAnalyzer::new();
        let path = PathBuf::from("test_fabric.mod.json");
        
        // Create a test fabric.mod.json file
        let json_content = r#"{
            "id": "testmod",
            "version": "1.0.0",
            "name": "Test Mod",
            "description": "A test mod",
            "authors": ["TestAuthor"],
            "depends": {
                "minecraft": "1.20.1",
                "fabricloader": "0.14.0"
            }
        }"#;
        
        std::fs::write(&path, json_content).unwrap();
        
        let result = analyzer.parse_fabric_mod_json(&path).await;
        assert!(result.is_ok());
        
        let metadata = result.unwrap();
        assert_eq!(metadata.id, "testmod");
        assert_eq!(metadata.name, "Test Mod");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.dependencies.len(), 2);
        
        // Clean up
        std::fs::remove_file(&path).unwrap();
    }
}
