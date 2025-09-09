use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

/// Mod compatibility analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityReport {
    pub server_id: String,
    pub scan_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub overall_status: CompatibilityStatus,
    pub issues: Vec<CompatibilityIssue>,
    pub suggestions: Vec<CompatibilitySuggestion>,
    pub summary: CompatibilitySummary,
}

/// Overall compatibility status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompatibilityStatus {
    Compatible,
    Warnings,
    Conflicts,
    Critical,
}

/// Individual compatibility issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityIssue {
    pub id: String,
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub title: String,
    pub description: String,
    pub affected_mods: Vec<String>,
    pub suggested_fixes: Vec<String>,
    pub auto_fixable: bool,
}

/// Issue severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Issue categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueCategory {
    MixinConflict,
    LoaderMismatch,
    VersionConflict,
    DependencyMissing,
    DuplicateMod,
    ABIIncompatibility,
    LoadOrder,
    ResourceConflict,
}

/// Compatibility suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilitySuggestion {
    pub id: String,
    pub title: String,
    pub description: String,
    pub action: SuggestionAction,
    pub confidence: f64, // 0.0 to 1.0
    pub affected_mods: Vec<String>,
}

/// Suggestion action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionAction {
    UpdateMod { from_version: String, to_version: String },
    DowngradeMod { from_version: String, to_version: String },
    RemoveMod { mod_id: String },
    AddDependency { mod_id: String, version: String },
    ReorderMods { mod_order: Vec<String> },
    ReplaceMod { from_mod: String, to_mod: String },
}

/// Compatibility summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilitySummary {
    pub total_mods: usize,
    pub compatible_mods: usize,
    pub warning_mods: usize,
    pub error_mods: usize,
    pub critical_mods: usize,
    pub auto_fixable_issues: usize,
    pub manual_fixes_required: usize,
}

/// Mod information extracted from files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub loader: String,
    pub mc_version: String,
    pub dependencies: Vec<Dependency>,
    pub mixins: Vec<MixinConfig>,
    pub conflicts: Vec<String>,
    pub file_path: String,
    pub sha1: String,
}

/// Mod dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub mod_id: String,
    pub version_range: String,
    pub required: bool,
}

/// Mixin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixinConfig {
    pub package: String,
    pub priority: Option<i32>,
    pub targets: Vec<String>,
}

/// Mod compatibility scanner
pub struct CompatibilityScanner {
    known_conflicts: HashMap<String, Vec<String>>,
    version_compatibility: HashMap<String, HashMap<String, Vec<String>>>,
    loader_compatibility: HashMap<String, Vec<String>>,
}

impl CompatibilityScanner {
    pub fn new() -> Self {
        Self {
            known_conflicts: Self::load_known_conflicts(),
            version_compatibility: Self::load_version_compatibility(),
            loader_compatibility: Self::load_loader_compatibility(),
        }
    }

    /// Scan a server's mods for compatibility issues
    pub async fn scan_server(&self, server_id: &str, mods_dir: &Path) -> Result<CompatibilityReport> {
        info!("Starting compatibility scan for server: {}", server_id);
        
        let scan_id = Uuid::new_v4().to_string();
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();
        let mut mod_infos = Vec::new();

        // Scan all mod files
        if mods_dir.exists() {
            for entry in std::fs::read_dir(mods_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.extension().and_then(|s| s.to_str()) == Some("jar") {
                    if let Ok(mod_info) = self.extract_mod_info(&path).await {
                        mod_infos.push(mod_info);
                    }
                }
            }
        }

        // Analyze compatibility
        self.analyze_mixin_conflicts(&mod_infos, &mut issues, &mut suggestions);
        self.analyze_loader_compatibility(&mod_infos, &mut issues, &mut suggestions);
        self.analyze_version_conflicts(&mod_infos, &mut issues, &mut suggestions);
        self.analyze_dependencies(&mod_infos, &mut issues, &mut suggestions);
        self.analyze_duplicates(&mod_infos, &mut issues, &mut suggestions);
        self.analyze_known_conflicts(&mod_infos, &mut issues, &mut suggestions);

        // Generate summary
        let summary = self.generate_summary(&mod_infos, &issues);
        let overall_status = self.determine_overall_status(&issues);

        Ok(CompatibilityReport {
            server_id: server_id.to_string(),
            scan_id,
            timestamp: chrono::Utc::now(),
            overall_status,
            issues,
            suggestions,
            summary,
        })
    }

    /// Extract mod information from a JAR file
    async fn extract_mod_info(&self, jar_path: &Path) -> Result<ModInfo> {
        // This is a simplified implementation
        // In a real implementation, you would:
        // 1. Extract the JAR file
        // 2. Parse fabric.mod.json or mods.toml
        // 3. Extract mixin configurations
        // 4. Parse dependencies
        
        let file_name = jar_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid file name"))?;

        // For now, return a mock mod info
        Ok(ModInfo {
            id: file_name.to_string(),
            name: file_name.to_string(),
            version: "1.0.0".to_string(),
            loader: "fabric".to_string(),
            mc_version: "1.20.1".to_string(),
            dependencies: Vec::new(),
            mixins: Vec::new(),
            conflicts: Vec::new(),
            file_path: jar_path.to_string_lossy().to_string(),
            sha1: "mock_sha1".to_string(),
        })
    }

    /// Analyze mixin conflicts
    fn analyze_mixin_conflicts(&self, mods: &[ModInfo], issues: &mut Vec<CompatibilityIssue>, suggestions: &mut Vec<CompatibilitySuggestion>) {
        // Check for mixin conflicts
        let mut mixin_targets: HashMap<String, Vec<String>> = HashMap::new();
        
        for mod_info in mods {
            for mixin in &mod_info.mixins {
                for target in &mixin.targets {
                    mixin_targets.entry(target.clone())
                        .or_insert_with(Vec::new)
                        .push(mod_info.id.clone());
                }
            }
        }

        // Find conflicts
        for (target, mods_using) in mixin_targets {
            if mods_using.len() > 1 {
                issues.push(CompatibilityIssue {
                    id: Uuid::new_v4().to_string(),
                    severity: IssueSeverity::Warning,
                    category: IssueCategory::MixinConflict,
                    title: format!("Mixin conflict on {}", target),
                    description: format!("Multiple mods are trying to mix into {}: {}", target, mods_using.join(", ")),
                    affected_mods: mods_using,
                    suggested_fixes: vec![
                        "Check if mods are compatible".to_string(),
                        "Update to newer versions".to_string(),
                        "Remove conflicting mods".to_string(),
                    ],
                    auto_fixable: false,
                });
            }
        }
    }

    /// Analyze loader compatibility
    fn analyze_loader_compatibility(&self, mods: &[ModInfo], issues: &mut Vec<CompatibilityIssue>, suggestions: &mut Vec<CompatibilitySuggestion>) {
        let mut loaders: HashSet<String> = mods.iter().map(|m| m.loader.clone()).collect();
        
        if loaders.len() > 1 {
            issues.push(CompatibilityIssue {
                id: Uuid::new_v4().to_string(),
                severity: IssueSeverity::Error,
                category: IssueCategory::LoaderMismatch,
                title: "Multiple mod loaders detected".to_string(),
                description: format!("Found mods for different loaders: {}", loaders.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")),
                affected_mods: mods.iter().map(|m| m.id.clone()).collect(),
                suggested_fixes: vec![
                    "Use only one mod loader".to_string(),
                    "Remove incompatible mods".to_string(),
                ],
                auto_fixable: false,
            });
        }
    }

    /// Analyze version conflicts
    fn analyze_version_conflicts(&self, mods: &[ModInfo], issues: &mut Vec<CompatibilityIssue>, suggestions: &mut Vec<CompatibilitySuggestion>) {
        let mut mc_versions: HashSet<String> = mods.iter().map(|m| m.mc_version.clone()).collect();
        
        if mc_versions.len() > 1 {
            issues.push(CompatibilityIssue {
                id: Uuid::new_v4().to_string(),
                severity: IssueSeverity::Error,
                category: IssueCategory::VersionConflict,
                title: "Multiple Minecraft versions detected".to_string(),
                description: format!("Found mods for different MC versions: {}", mc_versions.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")),
                affected_mods: mods.iter().map(|m| m.id.clone()).collect(),
                suggested_fixes: vec![
                    "Use mods for the same MC version".to_string(),
                    "Update mods to compatible versions".to_string(),
                ],
                auto_fixable: false,
            });
        }
    }

    /// Analyze dependencies
    fn analyze_dependencies(&self, mods: &[ModInfo], issues: &mut Vec<CompatibilityIssue>, suggestions: &mut Vec<CompatibilitySuggestion>) {
        let mod_ids: HashSet<String> = mods.iter().map(|m| m.id.clone()).collect();
        
        for mod_info in mods {
            for dep in &mod_info.dependencies {
                if dep.required && !mod_ids.contains(&dep.mod_id) {
                    issues.push(CompatibilityIssue {
                        id: Uuid::new_v4().to_string(),
                        severity: IssueSeverity::Error,
                        category: IssueCategory::DependencyMissing,
                        title: format!("Missing dependency: {}", dep.mod_id),
                        description: format!("{} requires {} but it's not installed", mod_info.name, dep.mod_id),
                        affected_mods: vec![mod_info.id.clone()],
                        suggested_fixes: vec![
                            format!("Install {}", dep.mod_id),
                        ],
                        auto_fixable: true,
                    });
                }
            }
        }
    }

    /// Analyze duplicate mods
    fn analyze_duplicates(&self, mods: &[ModInfo], issues: &mut Vec<CompatibilityIssue>, suggestions: &mut Vec<CompatibilitySuggestion>) {
        let mut mod_names: HashMap<String, Vec<String>> = HashMap::new();
        
        for mod_info in mods {
            mod_names.entry(mod_info.name.clone())
                .or_insert_with(Vec::new)
                .push(mod_info.id.clone());
        }

        for (name, mod_ids) in mod_names {
            if mod_ids.len() > 1 {
                issues.push(CompatibilityIssue {
                    id: Uuid::new_v4().to_string(),
                    severity: IssueSeverity::Warning,
                    category: IssueCategory::DuplicateMod,
                    title: format!("Duplicate mod: {}", name),
                    description: format!("Found {} instances of {}", mod_ids.len(), name),
                    affected_mods: mod_ids,
                    suggested_fixes: vec![
                        "Remove duplicate mods".to_string(),
                        "Keep only the latest version".to_string(),
                    ],
                    auto_fixable: true,
                });
            }
        }
    }

    /// Analyze known conflicts
    fn analyze_known_conflicts(&self, mods: &[ModInfo], issues: &mut Vec<CompatibilityIssue>, suggestions: &mut Vec<CompatibilitySuggestion>) {
        let mod_ids: HashSet<String> = mods.iter().map(|m| m.id.clone()).collect();
        
        for (mod_id, conflicts) in &self.known_conflicts {
            if mod_ids.contains(mod_id) {
                for conflict in conflicts {
                    if mod_ids.contains(conflict) {
                        issues.push(CompatibilityIssue {
                            id: Uuid::new_v4().to_string(),
                            severity: IssueSeverity::Critical,
                            category: IssueCategory::ABIIncompatibility,
                            title: format!("Known conflict: {} and {}", mod_id, conflict),
                            description: format!("{} and {} are known to be incompatible", mod_id, conflict),
                            affected_mods: vec![mod_id.clone(), conflict.clone()],
                            suggested_fixes: vec![
                                format!("Remove either {} or {}", mod_id, conflict),
                                "Check for alternative mods".to_string(),
                            ],
                            auto_fixable: true,
                        });
                    }
                }
            }
        }
    }

    /// Generate compatibility summary
    fn generate_summary(&self, mods: &[ModInfo], issues: &[CompatibilityIssue]) -> CompatibilitySummary {
        let total_mods = mods.len();
        let mut compatible_mods = total_mods;
        let mut warning_mods = 0;
        let mut error_mods = 0;
        let mut critical_mods = 0;
        let mut auto_fixable_issues = 0;
        let mut manual_fixes_required = 0;

        for issue in issues {
            match issue.severity {
                IssueSeverity::Info => {},
                IssueSeverity::Warning => warning_mods += 1,
                IssueSeverity::Error => error_mods += 1,
                IssueSeverity::Critical => critical_mods += 1,
            }

            if issue.auto_fixable {
                auto_fixable_issues += 1;
            } else {
                manual_fixes_required += 1;
            }
        }

        compatible_mods -= warning_mods + error_mods + critical_mods;

        CompatibilitySummary {
            total_mods,
            compatible_mods,
            warning_mods,
            error_mods,
            critical_mods,
            auto_fixable_issues,
            manual_fixes_required,
        }
    }

    /// Determine overall compatibility status
    fn determine_overall_status(&self, issues: &[CompatibilityIssue]) -> CompatibilityStatus {
        let mut has_critical = false;
        let mut has_errors = false;
        let mut has_warnings = false;

        for issue in issues {
            match issue.severity {
                IssueSeverity::Critical => has_critical = true,
                IssueSeverity::Error => has_errors = true,
                IssueSeverity::Warning => has_warnings = true,
                IssueSeverity::Info => {},
            }
        }

        if has_critical {
            CompatibilityStatus::Critical
        } else if has_errors {
            CompatibilityStatus::Conflicts
        } else if has_warnings {
            CompatibilityStatus::Warnings
        } else {
            CompatibilityStatus::Compatible
        }
    }

    /// Load known mod conflicts
    fn load_known_conflicts() -> HashMap<String, Vec<String>> {
        let mut conflicts = HashMap::new();
        
        // Example known conflicts
        conflicts.insert("create".to_string(), vec!["valkyrien-skies".to_string()]);
        conflicts.insert("optifine".to_string(), vec!["sodium".to_string(), "iris".to_string()]);
        
        conflicts
    }

    /// Load version compatibility data
    fn load_version_compatibility() -> HashMap<String, HashMap<String, Vec<String>>> {
        // This would load from a database or config file
        HashMap::new()
    }

    /// Load loader compatibility data
    fn load_loader_compatibility() -> HashMap<String, Vec<String>> {
        // This would load from a database or config file
        HashMap::new()
    }
}

/// Auto-fix engine for compatibility issues
pub struct AutoFixEngine {
    scanner: CompatibilityScanner,
}

impl AutoFixEngine {
    pub fn new() -> Self {
        Self {
            scanner: CompatibilityScanner::new(),
        }
    }

    /// Apply auto-fixes to resolve compatibility issues
    pub async fn apply_fixes(&self, server_id: &str, mods_dir: &Path, fixes: Vec<String>) -> Result<CompatibilityReport> {
        info!("Applying auto-fixes for server: {}", server_id);
        
        // This is a simplified implementation
        // In a real implementation, you would:
        // 1. Parse the requested fixes
        // 2. Apply them to the mod files
        // 3. Update mod configurations
        // 4. Re-scan to verify fixes
        
        // For now, just return a new scan
        self.scanner.scan_server(server_id, mods_dir).await
    }
}
