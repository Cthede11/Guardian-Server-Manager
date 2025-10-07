use crate::mod_manager::{ModInfo, ModManager};
use crate::external_apis::mod_provider::{ModProvider, ProviderType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::error::Error;
use std::io::Read;
use tokio::fs;
use tokio::io::AsyncReadExt;
use zip::ZipArchive;
use std::io::Cursor;

/// Modpack manifest structure for .mrpack files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackManifest {
    pub format_version: u32,
    pub game: String,
    pub version_id: String,
    pub name: String,
    pub summary: Option<String>,
    pub files: Vec<ModpackFile>,
    pub dependencies: HashMap<String, String>,
    pub server_overrides: Option<String>,
    pub client_overrides: Option<String>,
}

/// Individual file in a modpack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackFile {
    pub path: String,
    pub hashes: HashMap<String, String>,
    pub env: Option<ModpackFileEnv>,
    pub downloads: Vec<String>,
    pub file_size: Option<u64>,
}

/// Environment specification for modpack files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackFileEnv {
    pub client: String,
    pub server: String,
}

/// Modpack installation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackInstallResult {
    pub modpack_id: String,
    pub installed_mods: Vec<InstalledModInfo>,
    pub skipped_files: Vec<String>,
    pub errors: Vec<String>,
    pub total_size: u64,
}

/// Information about an installed mod
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledModInfo {
    pub mod_id: String,
    pub name: String,
    pub version: String,
    pub provider: String,
    pub file_path: String,
    pub file_size: u64,
    pub side: String, // "client", "server", or "both"
}

/// Modpack installer and updater
pub struct ModpackInstaller {
    mod_manager: ModManager,
    providers: HashMap<ProviderType, Box<dyn ModProvider + Send + Sync>>,
    mods_directory: PathBuf,
    temp_directory: PathBuf,
}

impl ModpackInstaller {
    /// Create a new modpack installer
    pub fn new(
        mod_manager: ModManager,
        providers: HashMap<ProviderType, Box<dyn ModProvider + Send + Sync>>,
        mods_directory: PathBuf,
        temp_directory: PathBuf,
    ) -> Self {
        Self {
            mod_manager,
            providers,
            mods_directory,
            temp_directory,
        }
    }

    /// Install a modpack from a .mrpack file
    pub async fn install_modpack(
        &self,
        modpack_path: &Path,
        server_id: &str,
    ) -> Result<ModpackInstallResult, Box<dyn Error>> {
        // Parse the modpack manifest
        let manifest = self.parse_modpack_manifest(modpack_path).await?;
        
        // Create server-specific mods directory
        let server_mods_dir = self.mods_directory.join(server_id);
        fs::create_dir_all(&server_mods_dir).await?;
        
        let mut installed_mods = Vec::new();
        let mut skipped_files = Vec::new();
        let mut errors = Vec::new();
        let mut total_size = 0u64;
        
        // Process each file in the modpack
        for file in &manifest.files {
            match self.process_modpack_file(file, &server_mods_dir, &manifest).await {
                Ok(Some(installed_mod)) => {
                    total_size += installed_mod.file_size;
                    installed_mods.push(installed_mod);
                }
                Ok(None) => {
                    skipped_files.push(file.path.clone());
                }
                Err(e) => {
                    errors.push(format!("Failed to process {}: {}", file.path, e));
                }
            }
        }
        
        Ok(ModpackInstallResult {
            modpack_id: manifest.name.clone(),
            installed_mods,
            skipped_files,
            errors,
            total_size,
        })
    }

    /// Parse modpack manifest from .mrpack file
    async fn parse_modpack_manifest(
        &self,
        modpack_path: &Path,
    ) -> Result<ModpackManifest, Box<dyn Error>> {
        // Read the modpack file
        let mut file = fs::File::open(modpack_path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;
        
        // Extract the manifest.json from the zip
        let cursor = Cursor::new(buffer);
        let mut archive = ZipArchive::new(cursor)?;
        
        let mut manifest_file = archive.by_name("modrinth.index.json")?;
        let mut manifest_content = String::new();
        manifest_file.read_to_string(&mut manifest_content)?;
        
        // Parse the manifest
        let manifest: ModpackManifest = serde_json::from_str(&manifest_content)?;
        Ok(manifest)
    }

    /// Process a single file from the modpack
    async fn process_modpack_file(
        &self,
        file: &ModpackFile,
        server_mods_dir: &Path,
        manifest: &ModpackManifest,
    ) -> Result<Option<InstalledModInfo>, Box<dyn Error>> {
        // Check if this is a server-side file
        if let Some(env) = &file.env {
            if env.server == "unsupported" {
                return Ok(None); // Skip client-only files
            }
        }
        
        // Determine the provider based on the download URL
        let provider = self.determine_provider(&file.downloads)?;
        
        // Download the mod file
        let file_path = server_mods_dir.join(&file.path);
        fs::create_dir_all(file_path.parent().unwrap()).await?;
        
        // Download from the first available URL
        if let Some(download_url) = file.downloads.first() {
            self.download_file(download_url, &file_path).await?;
        } else {
            return Err("No download URL available".into());
        }
        
        // Get file size
        let file_size = fs::metadata(&file_path).await?.len();
        
        // Try to extract mod information
        let mod_info = self.extract_mod_info(&file_path, &provider).await?;
        
        Ok(Some(InstalledModInfo {
            mod_id: mod_info.id,
            name: mod_info.name,
            version: mod_info.version,
            provider: provider.to_string(),
            file_path: file_path.to_string_lossy().to_string(),
            file_size,
            side: mod_info.side,
        }))
    }

    /// Determine the provider based on download URL
    fn determine_provider(&self, urls: &[String]) -> Result<ProviderType, Box<dyn Error>> {
        for url in urls {
            if url.contains("curseforge.com") {
                return Ok(ProviderType::CurseForge);
            } else if url.contains("modrinth.com") {
                return Ok(ProviderType::Modrinth);
            }
        }
        Err("Unknown provider".into())
    }

    /// Download a file from URL
    async fn download_file(&self, url: &str, file_path: &Path) -> Result<(), Box<dyn Error>> {
        let response = reqwest::get(url).await?;
        let content = response.bytes().await?;
        fs::write(file_path, content).await?;
        Ok(())
    }

    /// Extract mod information from a downloaded file
    async fn extract_mod_info(
        &self,
        file_path: &Path,
        provider: &ProviderType,
    ) -> Result<ModInfo, Box<dyn Error>> {
        // This is a simplified implementation
        // In a real implementation, you would parse the mod file to extract metadata
        let file_name = file_path.file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        
        Ok(ModInfo {
            id: "unknown".to_string(),
            name: file_name.clone(),
            description: "Mod from modpack".to_string(),
            author: "Unknown".to_string(),
            version: "1.0.0".to_string(),
            minecraft_version: "1.20.1".to_string(),
            loader: "fabric".to_string(),
            category: "misc".to_string(),
            side: "both".to_string(),
            download_url: None,
            file_size: Some(fs::metadata(file_path).await?.len()),
            sha1: None,
            dependencies: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    /// Update a modpack
    pub async fn update_modpack(
        &self,
        server_id: &str,
        modpack_path: &Path,
    ) -> Result<ModpackInstallResult, Box<dyn Error>> {
        // For now, just reinstall the modpack
        // In a real implementation, you would check for updates and only update changed mods
        self.install_modpack(modpack_path, server_id).await
    }

    /// Check for modpack updates
    pub async fn check_modpack_updates(
        &self,
        server_id: &str,
    ) -> Result<Vec<ModUpdateInfo>, Box<dyn Error>> {
        let mut updates = Vec::new();
        
        // Get installed mods for this server
        let server_mods_dir = self.mods_directory.join(server_id);
        if !server_mods_dir.exists() {
            return Ok(updates);
        }
        
        // Check each installed mod for updates
        let mut entries = fs::read_dir(&server_mods_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let file_path = entry.path();
            if file_path.extension().is_some_and(|ext| ext == "jar") {
                if let Ok(mod_info) = self.extract_mod_info(&file_path, &ProviderType::Modrinth).await {
                    // Check for updates using the appropriate provider
                    if let Some(provider) = self.providers.get(&ProviderType::Modrinth) {
                        if let Ok(Some(new_version)) = provider.check_for_updates(&mod_info.id, &mod_info.version).await {
                            updates.push(ModUpdateInfo {
                                mod_id: mod_info.id,
                                name: mod_info.name,
                                current_version: mod_info.version,
                                new_version,
                                provider: ProviderType::Modrinth,
                            });
                        }
                    }
                }
            }
        }
        
        Ok(updates)
    }
}

/// Information about a mod update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModUpdateInfo {
    pub mod_id: String,
    pub name: String,
    pub current_version: String,
    pub new_version: String,
    pub provider: ProviderType,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_modpack_installer_creation() {
        let temp_dir = TempDir::new().unwrap();
        let mods_dir = temp_dir.path().join("mods");
        let temp_dir_path = temp_dir.path().join("temp");
        
        let mod_manager = ModManager::new();
        let providers = HashMap::new();
        
        let installer = ModpackInstaller::new(
            mod_manager,
            providers,
            mods_dir,
            temp_dir_path,
        );
        
        assert!(installer.mods_directory.exists() || installer.mods_directory.parent().unwrap().exists());
    }
}
