use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use reqwest::Client;

/// Mod information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub minecraft_version: String,
    pub loader: String,
    pub category: String,
    pub side: String, // client, server, both
    pub download_url: Option<String>,
    pub file_size: Option<u64>,
    pub sha1: Option<String>,
    pub dependencies: Vec<ModDependency>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Mod dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependency {
    pub mod_id: String,
    pub version_range: String,
    pub required: bool,
}

/// Installed mod
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledMod {
    pub id: String,
    pub mod_info: ModInfo,
    pub server_id: String,
    pub file_path: String,
    pub installed_at: DateTime<Utc>,
    pub enabled: bool,
    pub version: String,
}

/// Mod installation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInstallationResult {
    pub success: bool,
    pub mod_id: String,
    pub message: String,
    pub installed_mod: Option<InstalledMod>,
    pub errors: Vec<String>,
}

/// Mod compatibility check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModCompatibilityResult {
    pub compatible: bool,
    pub issues: Vec<CompatibilityIssue>,
    pub warnings: Vec<String>,
}

/// Compatibility issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityIssue {
    pub mod_id: String,
    pub issue_type: String,
    pub description: String,
    pub severity: String, // error, warning, info
}

/// Mod manager for handling mod installation and management
#[derive(Clone)]
pub struct ModManager {
    /// Installed mods by server ID
    installed_mods: Arc<RwLock<HashMap<String, Vec<InstalledMod>>>>,
    /// Mod cache for downloaded mods
    mod_cache: Arc<RwLock<HashMap<String, ModInfo>>>,
    /// Base directory for mods
    mods_base_dir: PathBuf,
    /// HTTP client for downloading mods
    http_client: Client,
}

impl ModManager {
    pub fn new(mods_base_dir: PathBuf) -> Self {
        Self {
            installed_mods: Arc::new(RwLock::new(HashMap::new())),
            mod_cache: Arc::new(RwLock::new(HashMap::new())),
            mods_base_dir,
            http_client: Client::new(),
        }
    }

    /// Install a mod to a server
    pub async fn install_mod(
        &self,
        server_id: &str,
        mod_id: &str,
        version: &str,
        source: &str, // "curseforge", "modrinth", "direct"
    ) -> Result<ModInstallationResult, Box<dyn std::error::Error>> {
        let mut result = ModInstallationResult {
            success: false,
            mod_id: mod_id.to_string(),
            message: String::new(),
            installed_mod: None,
            errors: Vec::new(),
        };

        // Get mod information
        let mod_info = match self.get_mod_info(mod_id, version, source).await {
            Ok(info) => info,
            Err(e) => {
                result.errors.push(format!("Failed to get mod info: {}", e));
                return Ok(result);
            }
        };

        // Check compatibility
        let compatibility = self.check_mod_compatibility(server_id, &mod_info).await?;
        if !compatibility.compatible {
            result.errors.extend(compatibility.issues.into_iter().map(|i| i.description));
            result.message = "Mod is not compatible with server".to_string();
            return Ok(result);
        }

        // Download mod file
        let mod_file_path = match self.download_mod(&mod_info).await {
            Ok(path) => path,
            Err(e) => {
                result.errors.push(format!("Failed to download mod: {}", e));
                return Ok(result);
            }
        };

        // Install mod to server
        let server_mods_dir = self.mods_base_dir.join(server_id).join("mods");
        tokio::fs::create_dir_all(&server_mods_dir).await?;

        let installed_file_path = server_mods_dir.join(format!("{}-{}.jar", mod_id, version));
        tokio::fs::copy(&mod_file_path, &installed_file_path).await?;

        // Create installed mod record
        let installed_mod = InstalledMod {
            id: Uuid::new_v4().to_string(),
            mod_info: mod_info.clone(),
            server_id: server_id.to_string(),
            file_path: installed_file_path.to_string_lossy().to_string(),
            installed_at: Utc::now(),
            enabled: true,
            version: version.to_string(),
        };

        // Update installed mods
        {
            let mut installed = self.installed_mods.write().await;
            let server_mods = installed.entry(server_id.to_string()).or_insert_with(Vec::new);
            server_mods.push(installed_mod.clone());
        }

        result.success = true;
        result.message = "Mod installed successfully".to_string();
        result.installed_mod = Some(installed_mod);

        Ok(result)
    }

    /// Uninstall a mod from a server
    pub async fn uninstall_mod(
        &self,
        server_id: &str,
        mod_id: &str,
    ) -> Result<ModInstallationResult, Box<dyn std::error::Error>> {
        let mut result = ModInstallationResult {
            success: false,
            mod_id: mod_id.to_string(),
            message: String::new(),
            installed_mod: None,
            errors: Vec::new(),
        };

        let mut installed = self.installed_mods.write().await;
        if let Some(server_mods) = installed.get_mut(server_id) {
            if let Some(index) = server_mods.iter().position(|m| m.mod_info.id == mod_id) {
                let installed_mod = server_mods.remove(index);
                
                // Remove mod file
                if let Err(e) = tokio::fs::remove_file(&installed_mod.file_path).await {
                    result.errors.push(format!("Failed to remove mod file: {}", e));
                }

                result.success = true;
                result.message = "Mod uninstalled successfully".to_string();
                result.installed_mod = Some(installed_mod);
            } else {
                result.errors.push("Mod not found in server".to_string());
            }
        } else {
            result.errors.push("Server not found".to_string());
        }

        Ok(result)
    }

    /// Enable/disable a mod
    pub async fn toggle_mod(
        &self,
        server_id: &str,
        mod_id: &str,
        enabled: bool,
    ) -> Result<ModInstallationResult, Box<dyn std::error::Error>> {
        let mut result = ModInstallationResult {
            success: false,
            mod_id: mod_id.to_string(),
            message: String::new(),
            installed_mod: None,
            errors: Vec::new(),
        };

        let mut installed = self.installed_mods.write().await;
        if let Some(server_mods) = installed.get_mut(server_id) {
            if let Some(installed_mod) = server_mods.iter_mut().find(|m| m.mod_info.id == mod_id) {
                installed_mod.enabled = enabled;
                
                result.success = true;
                result.message = if enabled { "Mod enabled" } else { "Mod disabled" }.to_string();
                result.installed_mod = Some(installed_mod.clone());
            } else {
                result.errors.push("Mod not found in server".to_string());
            }
        } else {
            result.errors.push("Server not found".to_string());
        }

        Ok(result)
    }

    /// Get installed mods for a server
    pub async fn get_installed_mods(&self, server_id: &str) -> Result<Vec<InstalledMod>, Box<dyn std::error::Error>> {
        let installed = self.installed_mods.read().await;
        Ok(installed.get(server_id).cloned().unwrap_or_default())
    }

    /// Get mod information
    async fn get_mod_info(
        &self,
        mod_id: &str,
        version: &str,
        source: &str,
    ) -> Result<ModInfo, Box<dyn std::error::Error>> {
        // Check cache first
        {
            let cache = self.mod_cache.read().await;
            if let Some(info) = cache.get(mod_id) {
                return Ok(info.clone());
            }
        }

        // Fetch from source
        let mod_info = match source {
            "curseforge" => self.fetch_from_curseforge(mod_id, version).await?,
            "modrinth" => self.fetch_from_modrinth(mod_id, version).await?,
            "direct" => self.fetch_direct(mod_id, version).await?,
            _ => return Err("Unsupported mod source".into()),
        };

        // Cache the result
        {
            let mut cache = self.mod_cache.write().await;
            cache.insert(mod_id.to_string(), mod_info.clone());
        }

        Ok(mod_info)
    }

    /// Fetch mod info from CurseForge
    async fn fetch_from_curseforge(
        &self,
        mod_id: &str,
        version: &str,
    ) -> Result<ModInfo, Box<dyn std::error::Error>> {
        // In a real implementation, this would call the CurseForge API
        // For now, return placeholder data
        Ok(ModInfo {
            id: mod_id.to_string(),
            name: format!("Mod {}", mod_id),
            description: "A placeholder mod description".to_string(),
            author: "Unknown".to_string(),
            version: version.to_string(),
            minecraft_version: "1.20.1".to_string(),
            loader: "forge".to_string(),
            category: "misc".to_string(),
            side: "both".to_string(),
            download_url: Some(format!("https://example.com/mods/{}/{}.jar", mod_id, version)),
            file_size: Some(1024 * 1024), // 1MB
            sha1: Some("placeholder_sha1".to_string()),
            dependencies: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Fetch mod info from Modrinth
    async fn fetch_from_modrinth(
        &self,
        mod_id: &str,
        version: &str,
    ) -> Result<ModInfo, Box<dyn std::error::Error>> {
        // In a real implementation, this would call the Modrinth API
        // For now, return placeholder data
        Ok(ModInfo {
            id: mod_id.to_string(),
            name: format!("Mod {}", mod_id),
            description: "A placeholder mod description".to_string(),
            author: "Unknown".to_string(),
            version: version.to_string(),
            minecraft_version: "1.20.1".to_string(),
            loader: "fabric".to_string(),
            category: "misc".to_string(),
            side: "both".to_string(),
            download_url: Some(format!("https://example.com/mods/{}/{}.jar", mod_id, version)),
            file_size: Some(1024 * 1024), // 1MB
            sha1: Some("placeholder_sha1".to_string()),
            dependencies: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Fetch mod info directly
    async fn fetch_direct(
        &self,
        mod_id: &str,
        version: &str,
    ) -> Result<ModInfo, Box<dyn std::error::Error>> {
        // In a real implementation, this would parse mod metadata
        // For now, return placeholder data
        Ok(ModInfo {
            id: mod_id.to_string(),
            name: format!("Mod {}", mod_id),
            description: "A placeholder mod description".to_string(),
            author: "Unknown".to_string(),
            version: version.to_string(),
            minecraft_version: "1.20.1".to_string(),
            loader: "forge".to_string(),
            category: "misc".to_string(),
            side: "both".to_string(),
            download_url: Some(format!("https://example.com/mods/{}/{}.jar", mod_id, version)),
            file_size: Some(1024 * 1024), // 1MB
            sha1: Some("placeholder_sha1".to_string()),
            dependencies: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Download mod file
    async fn download_mod(&self, mod_info: &ModInfo) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let download_url = mod_info.download_url.as_ref()
            .ok_or("No download URL available")?;

        let response = self.http_client.get(download_url).send().await?;
        if !response.status().is_success() {
            return Err("Failed to download mod file".into());
        }

        let mod_data = response.bytes().await?;
        let temp_path = self.mods_base_dir.join("temp").join(format!("{}.jar", mod_info.id));
        tokio::fs::create_dir_all(temp_path.parent().unwrap()).await?;
        tokio::fs::write(&temp_path, mod_data).await?;

        Ok(temp_path)
    }

    /// Check mod compatibility
    async fn check_mod_compatibility(
        &self,
        server_id: &str,
        mod_info: &ModInfo,
    ) -> Result<ModCompatibilityResult, Box<dyn std::error::Error>> {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Check if mod is already installed
        let installed_mods = self.get_installed_mods(server_id).await?;
        if installed_mods.iter().any(|m| m.mod_info.id == mod_info.id) {
            issues.push(CompatibilityIssue {
                mod_id: mod_info.id.clone(),
                issue_type: "duplicate".to_string(),
                description: "Mod is already installed".to_string(),
                severity: "error".to_string(),
            });
        }

        // Check loader compatibility
        // In a real implementation, this would check server loader type
        if mod_info.loader != "forge" && mod_info.loader != "fabric" {
            issues.push(CompatibilityIssue {
                mod_id: mod_info.id.clone(),
                issue_type: "loader".to_string(),
                description: "Unsupported mod loader".to_string(),
                severity: "error".to_string(),
            });
        }

        // Check Minecraft version compatibility
        // In a real implementation, this would check server Minecraft version
        if mod_info.minecraft_version != "1.20.1" {
            warnings.push("Mod may not be compatible with server Minecraft version".to_string());
        }

        Ok(ModCompatibilityResult {
            compatible: issues.is_empty(),
            issues,
            warnings,
        })
    }

    /// Update mod cache
    pub async fn update_mod_cache(&self) -> Result<(), Box<dyn std::error::Error>> {
        // In a real implementation, this would refresh mod information from sources
        // For now, just clear the cache
        let mut cache = self.mod_cache.write().await;
        cache.clear();
        Ok(())
    }

    /// Get mod statistics
    pub async fn get_mod_stats(&self, server_id: &str) -> Result<ModStats, Box<dyn std::error::Error>> {
        let installed_mods = self.get_installed_mods(server_id).await?;
        
        let total_mods = installed_mods.len();
        let enabled_mods = installed_mods.iter().filter(|m| m.enabled).count();
        let disabled_mods = total_mods - enabled_mods;
        
        let total_size: u64 = installed_mods.iter()
            .map(|m| m.mod_info.file_size.unwrap_or(0))
            .sum();

        Ok(ModStats {
            total_mods,
            enabled_mods,
            disabled_mods,
            total_size_mb: total_size / (1024 * 1024),
            last_updated: Utc::now(),
        })
    }

    /// Search for mods
    pub async fn search_mods(
        &self,
        query: &str,
        minecraft_version: Option<&str>,
        loader: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<ModInfo>, Box<dyn std::error::Error>> {
        // TODO: Implement actual mod search
        Ok(vec![])
    }

    /// Download a mod (public version)
    pub async fn download_mod_public(&self, mod_info: &ModInfo) -> Result<PathBuf, Box<dyn std::error::Error>> {
        self.download_mod(mod_info).await
    }

    /// Sync mods from external sources
    pub async fn sync_mods_from_external_sources(&self) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement actual sync
        Ok(())
    }

    /// Apply install operation
    pub async fn apply_install_operation(
        &self,
        mod_id: &str,
        version: &str,
        provider: &str,
        file_path: &str,
        server_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement actual install operation
        Ok(())
    }

    /// Apply update operation
    pub async fn apply_update_operation(
        &self,
        mod_id: &str,
        from_version: &str,
        to_version: &str,
        provider: &str,
        file_path: &str,
        server_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement actual update operation
        Ok(())
    }

    /// Apply remove operation
    pub async fn apply_remove_operation(
        &self,
        mod_id: &str,
        file_path: &str,
        server_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement actual remove operation
        Ok(())
    }

    /// Apply enable operation
    pub async fn apply_enable_operation(
        &self,
        mod_id: &str,
        server_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement actual enable operation
        Ok(())
    }

    /// Apply disable operation
    pub async fn apply_disable_operation(
        &self,
        mod_id: &str,
        server_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement actual disable operation
        Ok(())
    }
}

/// Mod statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModStats {
    pub total_mods: usize,
    pub enabled_mods: usize,
    pub disabled_mods: usize,
    pub total_size_mb: u64,
    pub last_updated: DateTime<Utc>,
}

impl Default for ModManager {
    fn default() -> Self {
        Self::new(PathBuf::from("./mods"))
    }
}