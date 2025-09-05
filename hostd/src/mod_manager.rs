use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::{info, warn, error, debug};
use sha1::{Sha1, Digest};

use crate::external_apis::{ModrinthApiClient, CurseForgeApiClient};
use crate::database::{DatabaseManager, ModInfo, ModVersion, ModDependency, ModConflict};
use crate::compatibility_engine::{CompatibilityIssue, CompatibilityReport};
use crate::version_manager::ModLoader;

/// Mod manager service for handling mod operations
#[derive(Clone)]
pub struct ModManager {
    modrinth_client: ModrinthApiClient,
    curseforge_client: CurseForgeApiClient,
    database: DatabaseManager,
    download_dir: String,
}

/// Mod download result
#[derive(Debug, Clone)]
pub struct ModDownloadResult {
    pub mod_info: ModInfo,
    pub file_path: String,
    pub file_size: u64,
    pub sha256: String,
}

/// Mod search result from external APIs
#[derive(Debug, Clone)]
pub struct ExternalModSearchResult {
    pub mods: Vec<ModInfo>,
    pub total: u32,
    pub source: String,
}

impl ModManager {
    pub fn new(database: DatabaseManager, download_dir: String, curseforge_api_key: Option<String>) -> Self {
        let api_key = curseforge_api_key.unwrap_or_else(|| "default-key".to_string());
        Self {
            modrinth_client: ModrinthApiClient::new(),
            curseforge_client: CurseForgeApiClient::new(api_key),
            database,
            download_dir,
        }
    }

    /// Search for mods across all sources
    pub async fn search_mods(
        &self,
        query: &str,
        minecraft_version: Option<&str>,
        loader: Option<&str>,
        category: Option<&str>,
        source: Option<&str>,
        limit: u32,
    ) -> Result<Vec<ExternalModSearchResult>> {
        let mut results = Vec::new();

        // Search Modrinth
        if source.is_none() || source == Some("modrinth") {
            match self.search_modrinth(query, minecraft_version, loader, category, limit).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    error!("Failed to search Modrinth: {}", e);
                }
            }
        }

        // Search CurseForge
        if source.is_none() || source == Some("curseforge") {
            match self.search_curseforge(query, minecraft_version, loader, category, limit).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    error!("Failed to search CurseForge: {}", e);
                }
            }
        }

        Ok(results)
    }

    /// Search Modrinth
    async fn search_modrinth(
        &self,
        query: &str,
        minecraft_version: Option<&str>,
        loader: Option<&str>,
        category: Option<&str>,
        limit: u32,
    ) -> Result<ExternalModSearchResult> {
        let search_response = self.modrinth_client
            .search_mods(query, minecraft_version, loader, category, Some(limit), None)
            .await?;

        let mods: Vec<ModInfo> = search_response
            .hits
            .into_iter()
            .map(|project| self.modrinth_client.convert_to_mod_info(&project))
            .collect();

        Ok(ExternalModSearchResult {
            mods,
            total: search_response.total_hits,
            source: "modrinth".to_string(),
        })
    }

    /// Search CurseForge
    async fn search_curseforge(
        &self,
        query: &str,
        minecraft_version: Option<&str>,
        loader: Option<&str>,
        category: Option<&str>,
        limit: u32,
    ) -> Result<ExternalModSearchResult> {
        // Map loader to CurseForge mod loader type
        let mod_loader_type = match loader {
            Some("forge") | Some("neoforge") => Some(1), // Forge
            Some("fabric") | Some("quilt") => Some(4),   // Fabric
            _ => None,
        };

        let search_response = self.curseforge_client
            .search_mods(
                432, // Minecraft game ID
                Some(query),
                None, // category_id
                minecraft_version,
                mod_loader_type,
                Some(2), // Sort by popularity
                Some("desc"),
                Some(limit),
                None,
            )
            .await?;

        let mods: Vec<ModInfo> = search_response
            .data
            .into_iter()
            .map(|project| self.curseforge_client.convert_to_mod_info(&project))
            .collect();

        Ok(ExternalModSearchResult {
            mods,
            total: search_response.pagination.total_count,
            source: "curseforge".to_string(),
        })
    }

    /// Download and install a mod
    pub async fn download_mod(
        &self,
        mod_id: &str,
        version: Option<&str>,
        minecraft_version: Option<&str>,
        loader: Option<&str>,
    ) -> Result<ModDownloadResult> {
        // Determine source from mod_id format or try both
        let source = if mod_id.starts_with("modrinth-") {
            "modrinth"
        } else if mod_id.parse::<u32>().is_ok() {
            "curseforge"
        } else {
            // Try to find the mod in our database first
            if let Ok(Some(existing_mod)) = self.database.get_mod(mod_id).await {
                return self.download_existing_mod(&existing_mod, version, minecraft_version, loader).await;
            }
            "unknown"
        };

        match source {
            "modrinth" => self.download_modrinth_mod(mod_id, version, minecraft_version, loader).await,
            "curseforge" => self.download_curseforge_mod(mod_id, version, minecraft_version, loader).await,
            _ => Err(anyhow::anyhow!("Unknown mod source for ID: {}", mod_id)),
        }
    }

    /// Download mod from Modrinth
    async fn download_modrinth_mod(
        &self,
        mod_id: &str,
        version: Option<&str>,
        minecraft_version: Option<&str>,
        loader: Option<&str>,
    ) -> Result<ModDownloadResult> {
        // Get project details
        let project = self.modrinth_client.get_project(mod_id).await?;
        let mod_info = self.modrinth_client.convert_to_mod_info(&project);

        // Get versions
        let versions = self.modrinth_client
            .get_project_versions(
                mod_id,
                minecraft_version.map(|v| vec![v]),
                loader.map(|l| vec![l]),
            )
            .await?;

        if versions.is_empty() {
            return Err(anyhow::anyhow!("No compatible versions found for mod: {}", mod_id));
        }

        // Select version
        let selected_version = if let Some(version) = version {
            versions.iter().find(|v| v.version_number == version)
                .ok_or_else(|| anyhow::anyhow!("Version {} not found", version))?
        } else {
            &versions[0] // Use latest version
        };

        // Get primary file
        let primary_file = selected_version
            .files
            .iter()
            .find(|f| f.primary)
            .ok_or_else(|| anyhow::anyhow!("No primary file found for version"))?;

        // Download file
        let file_path = self.download_file(
            &primary_file.url,
            &primary_file.filename,
            &primary_file.hashes,
        ).await?;

        // Store mod info in database
        self.store_mod_info(&mod_info, selected_version, &file_path).await?;

        Ok(ModDownloadResult {
            mod_info,
            file_path,
            file_size: primary_file.size,
            sha256: primary_file.hashes.get("sha512").unwrap_or(&primary_file.hashes.get("sha1").unwrap_or(&"".to_string())).clone(),
        })
    }

    /// Download mod from CurseForge
    async fn download_curseforge_mod(
        &self,
        mod_id: &str,
        version: Option<&str>,
        minecraft_version: Option<&str>,
        loader: Option<&str>,
    ) -> Result<ModDownloadResult> {
        let project_id = mod_id.parse::<u32>()?;

        // Get project details
        let project = self.curseforge_client.get_project(project_id).await?;
        let mod_info = self.curseforge_client.convert_to_mod_info(&project);

        // Get files
        let mod_loader_type = match loader {
            Some("forge") | Some("neoforge") => Some(1),
            Some("fabric") | Some("quilt") => Some(4),
            _ => None,
        };

        let files = self.curseforge_client
            .get_project_files(project_id, minecraft_version, mod_loader_type, None, None, Some(10))
            .await?;

        if files.is_empty() {
            return Err(anyhow::anyhow!("No compatible files found for mod: {}", mod_id));
        }

        // Select file
        let selected_file = if let Some(version) = version {
            files.iter().find(|f| f.display_name == version)
                .ok_or_else(|| anyhow::anyhow!("Version {} not found", version))?
        } else {
            &files[0] // Use latest file
        };

        // Download file
        let file_path = self.download_file(
            &selected_file.download_url,
            &selected_file.file_name,
            &selected_file.hashes.iter().map(|h| (h.algo.to_string(), h.value.clone())).collect(),
        ).await?;

        // Store mod info in database
        self.store_mod_info(&mod_info, selected_file, &file_path).await?;

        Ok(ModDownloadResult {
            mod_info,
            file_path,
            file_size: selected_file.file_length,
            sha256: selected_file.hashes.iter().find(|h| h.algo == 1).map(|h| h.value.clone()).unwrap_or_default(),
        })
    }

    /// Download an existing mod from database
    async fn download_existing_mod(
        &self,
        mod_info: &ModInfo,
        version: Option<&str>,
        minecraft_version: Option<&str>,
        loader: Option<&str>,
    ) -> Result<ModDownloadResult> {
        // Get mod versions from database
        let versions = self.database.get_mod_versions(&mod_info.id).await?;
        
        if versions.is_empty() {
            return Err(anyhow::anyhow!("No versions found for mod: {}", mod_info.id));
        }

        // Select version
        let selected_version = if let Some(version) = version {
            versions.iter().find(|v| v.version == version)
                .ok_or_else(|| anyhow::anyhow!("Version {} not found", version))?
        } else {
            &versions[0] // Use latest version
        };

        // Check if file already exists
        let file_path = format!("{}/{}_{}.jar", self.download_dir, mod_info.id, selected_version.version);
        
        if Path::new(&file_path).exists() {
            info!("Mod file already exists: {}", file_path);
            return Ok(ModDownloadResult {
                mod_info: mod_info.clone(),
                file_path,
                file_size: selected_version.file_size,
                sha256: selected_version.sha256.clone(),
            });
        }

        // Download file
        let downloaded_path = self.download_file(
            &selected_version.download_url,
            &format!("{}_{}.jar", mod_info.id, selected_version.version),
            &HashMap::new(), // We'll verify with SHA256
        ).await?;

        Ok(ModDownloadResult {
            mod_info: mod_info.clone(),
            file_path: downloaded_path,
            file_size: selected_version.file_size,
            sha256: selected_version.sha256.clone(),
        })
    }

    /// Download a file from URL
    async fn download_file(
        &self,
        url: &str,
        filename: &str,
        hashes: &HashMap<String, String>,
    ) -> Result<String> {
        info!("Downloading file: {} from {}", filename, url);

        let response = reqwest::get(url).await?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to download file: {}", response.status()));
        }

        let content = response.bytes().await?;
        
        // Verify hash if provided
        if let Some(expected_sha1) = hashes.get("1") {
            let actual_sha1 = format!("{:x}", sha1::Sha1::digest(&content));
            if actual_sha1 != *expected_sha1 {
                return Err(anyhow::anyhow!("SHA1 hash mismatch for file: {}", filename));
            }
        }

        // Create download directory if it doesn't exist
        fs::create_dir_all(&self.download_dir).await?;

        // Save file
        let file_path = format!("{}/{}", self.download_dir, filename);
        fs::write(&file_path, content).await?;

        info!("Downloaded file: {}", file_path);
        Ok(file_path)
    }

    /// Store mod information in database
    async fn store_mod_info(
        &self,
        mod_info: &ModInfo,
        version_data: &dyn std::fmt::Debug, // Generic version data
        file_path: &str,
    ) -> Result<()> {
        // Store mod info
        // Note: This is a simplified version. In practice, you'd need to handle
        // the different version data types from Modrinth and CurseForge
        
        info!("Storing mod info for: {}", mod_info.name);
        // TODO: Implement proper database storage
        Ok(())
    }

    /// Sync mods from external sources
    pub async fn sync_mods_from_external_sources(&self) -> Result<()> {
        info!("Starting mod sync from external sources...");

        // Sync popular mods from Modrinth
        match self.search_modrinth("", None, None, None, 100).await {
            Ok(result) => {
                info!("Synced {} mods from Modrinth", result.mods.len());
                for mod_info in result.mods {
                    if let Err(e) = self.store_mod_info(&mod_info, &(), "").await {
                        error!("Failed to store mod {}: {}", mod_info.name, e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to sync from Modrinth: {}", e);
            }
        }

        // Sync popular mods from CurseForge
        match self.search_curseforge("", None, None, None, 100).await {
            Ok(result) => {
                info!("Synced {} mods from CurseForge", result.mods.len());
                for mod_info in result.mods {
                    if let Err(e) = self.store_mod_info(&mod_info, &(), "").await {
                        error!("Failed to store mod {}: {}", mod_info.name, e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to sync from CurseForge: {}", e);
            }
        }

        info!("Mod sync completed");
        Ok(())
    }

    /// Get mod compatibility report
    pub async fn check_mod_compatibility(
        &self,
        mod_ids: &[String],
        minecraft_version: &str,
        loader: &str,
    ) -> Result<CompatibilityReport> {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Check each mod for compatibility
        for mod_id in mod_ids {
            if let Ok(Some(mod_info)) = self.database.get_mod(mod_id).await {
                // Check Minecraft version compatibility
                if !self.check_minecraft_version_compatibility(&mod_info, minecraft_version).await? {
                    issues.push(CompatibilityIssue::VersionMismatch {
                        mod_id: mod_id.clone(),
                        required_version: minecraft_version.to_string(),
                        available_versions: vec![],
                    });
                }

                // Check loader compatibility
                if !self.check_loader_compatibility(&mod_info, loader).await? {
                    issues.push(CompatibilityIssue::LoaderIncompatibility {
                        mod_id: mod_id.clone(),
                        required_loader: ModLoader::Forge { version: "1.20.1".to_string() }, // Placeholder
                        current_loader: ModLoader::Fabric { version: loader.to_string() }, // Placeholder
                    });
                }
            }
        }

        // Check for mod conflicts
        let conflicts = self.check_mod_conflicts(mod_ids).await?;
        issues.extend(conflicts);

        Ok(CompatibilityReport {
            is_compatible: issues.is_empty(),
            issues,
            warnings,
            recommendations: vec![],
        })
    }

    /// Check Minecraft version compatibility
    async fn check_minecraft_version_compatibility(
        &self,
        mod_info: &ModInfo,
        minecraft_version: &str,
    ) -> Result<bool> {
        // This would check the mod's supported versions
        // For now, return true as a placeholder
        Ok(true)
    }

    /// Check loader compatibility
    async fn check_loader_compatibility(
        &self,
        mod_info: &ModInfo,
        loader: &str,
    ) -> Result<bool> {
        // This would check the mod's supported loaders
        // For now, return true as a placeholder
        Ok(true)
    }

    /// Check for mod conflicts
    async fn check_mod_conflicts(&self, mod_ids: &[String]) -> Result<Vec<CompatibilityIssue>> {
        // This would check for known conflicts between mods
        // For now, return empty as a placeholder
        Ok(Vec::new())
    }
}


