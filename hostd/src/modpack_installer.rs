use crate::mod_manager::{ModInfo, ModManager};
use crate::external_apis::mod_provider::{ModProvider, ProviderType};
use crate::security::{PathSanitizer, SecureExtractor};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::error::Error;
use std::io::Read;
use tokio::fs;
use tokio::io::AsyncReadExt;
use zip::ZipArchive;
use std::io::Cursor;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use std::sync::Arc;
use futures::StreamExt;

/// Modpack manifest structure for .mrpack files (Modrinth)
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

/// CurseForge modpack manifest structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeManifest {
    pub manifest_type: String,
    pub manifest_version: u32,
    pub name: String,
    pub version: String,
    pub author: String,
    pub files: Vec<CurseForgeFile>,
    pub minecraft: CurseForgeMinecraft,
    pub overrides: Option<String>,
}

/// CurseForge file entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeFile {
    pub project_id: u32,
    pub file_id: u32,
    pub required: bool,
}

/// CurseForge minecraft version info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeMinecraft {
    pub version: String,
    pub mod_loaders: Vec<CurseForgeModLoader>,
}

/// CurseForge mod loader info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeModLoader {
    pub id: String,
    pub primary: bool,
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

/// Unified modpack manifest that can represent both Modrinth and CurseForge formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedModpackManifest {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub summary: Option<String>,
    pub minecraft_version: String,
    pub loader: String,
    pub files: Vec<UnifiedModpackFile>,
    pub dependencies: HashMap<String, String>,
    pub server_overrides: Option<String>,
    pub client_overrides: Option<String>,
    pub provider: String, // "modrinth" or "curseforge"
}

/// Unified modpack file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedModpackFile {
    pub path: String,
    pub hashes: HashMap<String, String>,
    pub env: Option<ModpackFileEnv>,
    pub downloads: Vec<String>,
    pub file_size: Option<u64>,
    pub project_id: Option<String>,
    pub file_id: Option<String>,
    pub required: bool,
}

/// Modpack installer and updater
pub struct ModpackInstaller {
    mod_manager: ModManager,
    providers: HashMap<ProviderType, Box<dyn ModProvider + Send + Sync>>,
    mods_directory: PathBuf,
    temp_directory: PathBuf,
    secure_extractor: SecureExtractor,
}

impl ModpackInstaller {
    /// Create a new modpack installer
    pub fn new(
        mod_manager: ModManager,
        providers: HashMap<ProviderType, Box<dyn ModProvider + Send + Sync>>,
        mods_directory: PathBuf,
        temp_directory: PathBuf,
    ) -> Self {
        let secure_extractor = SecureExtractor::new(mods_directory.clone());
        Self {
            mod_manager,
            providers,
            mods_directory,
            temp_directory,
            secure_extractor,
        }
    }

    /// Install a modpack from a .mrpack or .zip file
    pub async fn install_modpack(
        &self,
        modpack_path: &Path,
        server_id: &str,
    ) -> Result<ModpackInstallResult, Box<dyn Error>> {
        // Parse the modpack manifest (detects format automatically)
        let manifest = self.parse_unified_modpack_manifest(modpack_path).await?;
        
        // Create server-specific mods directory
        let server_mods_dir = self.mods_directory.join(server_id);
        fs::create_dir_all(&server_mods_dir).await?;
        
        let mut installed_mods = Vec::new();
        let mut skipped_files = Vec::new();
        let mut errors = Vec::new();
        let mut total_size = 0u64;
        
        // Process each file in the modpack
        for file in &manifest.files {
            match self.process_unified_modpack_file(file, &server_mods_dir, &manifest).await {
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

    /// Install a modpack with parallel downloads and progress events
    pub async fn install_modpack_with_progress(
        &self,
        modpack_path: &Path,
        server_id: &str,
        progress_sender: mpsc::UnboundedSender<ProgressEvent>,
    ) -> Result<ModpackInstallResult, Box<dyn Error>> {
        // Parse the modpack manifest
        let manifest = self.parse_unified_modpack_manifest(modpack_path).await?;
        
        // Create server-specific mods directory
        let server_mods_dir = self.mods_directory.join(server_id);
        fs::create_dir_all(&server_mods_dir).await?;
        
        // Prepare files for parallel download
        let mut download_files = Vec::new();
        let mut skipped_files = Vec::new();
        
        for file in &manifest.files {
            // Check if this is a server-side file
            if let Some(env) = &file.env {
                if env.server == "unsupported" {
                    skipped_files.push(file.path.clone());
                    continue;
                }
            }
            
            // Sanitize the file path
            if !self.secure_extractor.sanitizer().is_safe_path(&file.path) {
                tracing::warn!("Skipping unsafe file path: {}", file.path);
                skipped_files.push(file.path.clone());
                continue;
            }
            
            download_files.push((file.path.clone(), file.downloads.clone()));
        }
        
        // Download files in parallel
        let downloader = ParallelDownloader::new(4) // 4 concurrent downloads
            .with_progress_sender(progress_sender.clone());
        
        let download_results = downloader.download_files(download_files).await.map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())) as Box<dyn Error>)?;
        
        // Process downloaded files
        let mut installed_mods = Vec::new();
        let mut errors = Vec::new();
        let mut total_size = 0u64;
        let mut completed = 0;
        
        for result in download_results {
            if !result.success {
                if let Some(error) = result.error {
                    errors.push(format!("Failed to download {}: {}", result.file_path, error));
                }
                continue;
            }
            
            // Find the original file info
            let file_info = manifest.files.iter()
                .find(|f| f.path == result.file_path)
                .ok_or("File info not found")?;
            
            // Verify hash if provided
            if let Some(sha1) = file_info.hashes.get("sha1") {
                if !self.verify_hash(&result.content, sha1, "sha1") {
                    let _ = progress_sender.send(ProgressEvent::HashVerification {
                        file_path: result.file_path.clone(),
                        success: false,
                    });
                    errors.push(format!("Hash verification failed for {}", result.file_path));
                    continue;
                }
                let _ = progress_sender.send(ProgressEvent::HashVerification {
                    file_path: result.file_path.clone(),
                    success: true,
                });
            }
            
            // Extract the file securely
            match self.secure_extractor.extract_file(&result.file_path, &result.content).await {
                Ok(file_path) => {
                    // Determine provider
                    let provider = if !manifest.provider.is_empty() {
                        match manifest.provider.as_str() {
                            "modrinth" => ProviderType::Modrinth,
                            "curseforge" => ProviderType::CurseForge,
                            _ => self.determine_provider_from_urls(&file_info.downloads)?,
                        }
                    } else {
                        self.determine_provider_from_urls(&file_info.downloads)?
                    };
                    
                    // Extract mod information
                    let mod_info = self.extract_mod_info(&file_path, &provider).await?;
                    
                    installed_mods.push(InstalledModInfo {
                        mod_id: mod_info.id,
                        name: mod_info.name,
                        version: mod_info.version,
                        provider: provider.to_string(),
                        file_path: file_path.to_string_lossy().to_string(),
                        file_size: result.size,
                        side: mod_info.side,
                    });
                    
                    total_size += result.size;
                    completed += 1;
                    
                    let _ = progress_sender.send(ProgressEvent::OverallProgress {
                        completed,
                        total: manifest.files.len(),
                    });
                }
                Err(e) => {
                    errors.push(format!("Failed to extract {}: {}", result.file_path, e));
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

    /// Parse unified modpack manifest (detects format automatically)
    async fn parse_unified_modpack_manifest(
        &self,
        modpack_path: &Path,
    ) -> Result<UnifiedModpackManifest, Box<dyn Error>> {
        // Read the modpack file
        let mut file = fs::File::open(modpack_path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;
        
        // Extract the manifest from the zip
        let cursor = Cursor::new(buffer);
        let mut archive = ZipArchive::new(cursor)?;
        
        // Try to parse as Modrinth first
        if let Ok(mut manifest_file) = archive.by_name("modrinth.index.json") {
            let mut manifest_content = String::new();
            manifest_file.read_to_string(&mut manifest_content)?;
            let modrinth_manifest: ModpackManifest = serde_json::from_str(&manifest_content)?;
            return Ok(self.convert_modrinth_to_unified(modrinth_manifest));
        }
        
        // Try to parse as CurseForge
        if let Ok(mut manifest_file) = archive.by_name("manifest.json") {
            let mut manifest_content = String::new();
            manifest_file.read_to_string(&mut manifest_content)?;
            let curseforge_manifest: CurseForgeManifest = serde_json::from_str(&manifest_content)?;
            return Ok(self.convert_curseforge_to_unified(curseforge_manifest).await?);
        }
        
        Err("No supported manifest found in modpack".into())
    }

    /// Convert Modrinth manifest to unified format
    fn convert_modrinth_to_unified(&self, manifest: ModpackManifest) -> UnifiedModpackManifest {
        UnifiedModpackManifest {
            name: manifest.name,
            version: manifest.version_id,
            author: None,
            summary: manifest.summary,
            minecraft_version: manifest.dependencies.get("minecraft").cloned().unwrap_or_default(),
            loader: manifest.dependencies.get("forge").or_else(|| manifest.dependencies.get("fabric"))
                .cloned().unwrap_or_else(|| "forge".to_string()),
            files: manifest.files.into_iter().map(|f| UnifiedModpackFile {
                path: f.path,
                hashes: f.hashes,
                env: f.env,
                downloads: f.downloads,
                file_size: f.file_size,
                project_id: None,
                file_id: None,
                required: true,
            }).collect(),
            dependencies: manifest.dependencies,
            server_overrides: manifest.server_overrides,
            client_overrides: manifest.client_overrides,
            provider: "modrinth".to_string(),
        }
    }

    /// Convert CurseForge manifest to unified format
    async fn convert_curseforge_to_unified(&self, manifest: CurseForgeManifest) -> Result<UnifiedModpackManifest, Box<dyn Error>> {
        // Get the primary mod loader
        let loader = manifest.minecraft.mod_loaders.iter()
            .find(|ml| ml.primary)
            .map(|ml| ml.id.clone())
            .unwrap_or_else(|| "forge".to_string());

        // Convert files - we'll need to fetch file details from CurseForge API
        let mut unified_files = Vec::new();
        for file in manifest.files {
            // For now, create a placeholder file entry
            // In a real implementation, you would fetch file details from CurseForge API
            unified_files.push(UnifiedModpackFile {
                path: format!("mods/{}.jar", file.project_id),
                hashes: HashMap::new(),
                env: Some(ModpackFileEnv {
                    client: "required".to_string(),
                    server: "required".to_string(),
                }),
                downloads: vec![format!("https://www.curseforge.com/minecraft/mc-mods/{}/files/{}", 
                    file.project_id, file.file_id)],
                file_size: None,
                project_id: Some(file.project_id.to_string()),
                file_id: Some(file.file_id.to_string()),
                required: file.required,
            });
        }

        Ok(UnifiedModpackManifest {
            name: manifest.name,
            version: manifest.version,
            author: Some(manifest.author),
            summary: None,
            minecraft_version: manifest.minecraft.version,
            loader,
            files: unified_files,
            dependencies: HashMap::new(),
            server_overrides: manifest.overrides,
            client_overrides: None,
            provider: "curseforge".to_string(),
        })
    }

    /// Process a single file from the unified modpack
    async fn process_unified_modpack_file(
        &self,
        file: &UnifiedModpackFile,
        server_mods_dir: &Path,
        manifest: &UnifiedModpackManifest,
    ) -> Result<Option<InstalledModInfo>, Box<dyn Error>> {
        // Check if this is a server-side file
        if let Some(env) = &file.env {
            if env.server == "unsupported" {
                return Ok(None); // Skip client-only files
            }
        }
        
        // Sanitize the file path
        if !self.secure_extractor.sanitizer().is_safe_path(&file.path) {
            tracing::warn!("Skipping unsafe file path: {}", file.path);
            return Ok(None);
        }
        
        // Determine the provider based on the manifest provider or download URL
        let provider = if !manifest.provider.is_empty() {
            match manifest.provider.as_str() {
                "modrinth" => ProviderType::Modrinth,
                "curseforge" => ProviderType::CurseForge,
                _ => self.determine_provider_from_urls(&file.downloads)?,
            }
        } else {
            self.determine_provider_from_urls(&file.downloads)?
        };
        
        // Download the file content first
        let content = if let Some(download_url) = file.downloads.first() {
            self.download_file_content(download_url).await?
        } else {
            return Err("No download URL available".into());
        };
        
        // Verify hash if provided
        if let Some(sha1) = file.hashes.get("sha1") {
            if !self.verify_hash(&content, sha1, "sha1") {
                return Err(format!("Hash verification failed for {}", file.path).into());
            }
        }
        
        // Extract the file securely
        let file_path = self.secure_extractor.extract_file(&file.path, &content).await?;
        
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

    /// Process a single file from the modpack (legacy method)
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
        let provider = self.determine_provider_from_urls(&file.downloads)?;
        
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
    fn determine_provider_from_urls(&self, urls: &[String]) -> Result<ProviderType, Box<dyn Error>> {
        for url in urls {
            if url.contains("curseforge.com") {
                return Ok(ProviderType::CurseForge);
            } else if url.contains("modrinth.com") {
                return Ok(ProviderType::Modrinth);
            }
        }
        Err("Unknown provider".into())
    }

    /// Determine the provider based on download URL (legacy method)
    fn determine_provider(&self, urls: &[String]) -> Result<ProviderType, Box<dyn Error>> {
        self.determine_provider_from_urls(urls)
    }

    /// Download a file from URL
    async fn download_file(&self, url: &str, file_path: &Path) -> Result<(), Box<dyn Error>> {
        let response = reqwest::get(url).await?;
        let content = response.bytes().await?;
        fs::write(file_path, content).await?;
        Ok(())
    }

    /// Download file content from URL
    async fn download_file_content(&self, url: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let response = reqwest::get(url).await?;
        let content = response.bytes().await?;
        Ok(content.to_vec())
    }

    /// Verify file hash
    fn verify_hash(&self, content: &[u8], expected_hash: &str, algorithm: &str) -> bool {
        match algorithm {
            "sha1" => {
                use sha1::{Sha1, Digest};
                let mut hasher = Sha1::new();
                hasher.update(content);
                let result = hasher.finalize();
                format!("{:x}", result) == expected_hash.to_lowercase()
            }
            "sha512" => {
                use sha2::{Sha512, Digest};
                let mut hasher = Sha512::new();
                hasher.update(content);
                let result = hasher.finalize();
                format!("{:x}", result) == expected_hash.to_lowercase()
            }
            _ => {
                tracing::warn!("Unsupported hash algorithm: {}", algorithm);
                false
            }
        }
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

/// Progress event for modpack installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgressEvent {
    FileStarted { file_path: String },
    FileCompleted { file_path: String, size: u64 },
    FileFailed { file_path: String, error: String },
    DownloadProgress { file_path: String, bytes_downloaded: u64, total_bytes: Option<u64> },
    HashVerification { file_path: String, success: bool },
    MirrorFallback { file_path: String, mirror_url: String },
    OverallProgress { completed: usize, total: usize },
}

/// Download result for a single file
#[derive(Debug, Clone)]
pub struct DownloadResult {
    pub file_path: String,
    pub content: Vec<u8>,
    pub success: bool,
    pub error: Option<String>,
    pub size: u64,
}

/// Parallel download manager
pub struct ParallelDownloader {
    max_concurrent: usize,
    progress_sender: Option<mpsc::UnboundedSender<ProgressEvent>>,
}

impl ParallelDownloader {
    /// Create a new parallel downloader
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            max_concurrent,
            progress_sender: None,
        }
    }

    /// Set progress event sender
    pub fn with_progress_sender(mut self, sender: mpsc::UnboundedSender<ProgressEvent>) -> Self {
        self.progress_sender = Some(sender);
        self
    }

    /// Download multiple files in parallel
    pub async fn download_files(
        &self,
        files: Vec<(String, Vec<String>)>, // (file_path, urls)
    ) -> Result<Vec<DownloadResult>, Box<dyn Error + Send + Sync>> {
        let mut results = Vec::new();
        let mut join_set = JoinSet::new();
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_concurrent));

        // Start downloads
        for (file_path, urls) in files {
            let semaphore = semaphore.clone();
            let progress_sender = self.progress_sender.clone();
            
            join_set.spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                
                if let Some(ref sender) = progress_sender {
                    let _ = sender.send(ProgressEvent::FileStarted { 
                        file_path: file_path.clone() 
                    });
                }

                let result = Self::download_file_with_fallbacks(&file_path, &urls, &progress_sender).await;
                
                if let Some(ref sender) = progress_sender {
                    match &result {
                        Ok(download_result) => {
                            let _ = sender.send(ProgressEvent::FileCompleted { 
                                file_path: file_path.clone(),
                                size: download_result.size,
                            });
                        }
                        Err(e) => {
                            let _ = sender.send(ProgressEvent::FileFailed { 
                                file_path: file_path.clone(),
                                error: e.to_string(),
                            });
                        }
                    }
                }

                result
            });
        }

        // Collect results
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok(download_result)) => results.push(download_result),
                Ok(Err(e)) => {
                    tracing::error!("Download failed: {}", e);
                    // Add failed result
                    results.push(DownloadResult {
                        file_path: "unknown".to_string(),
                        content: Vec::new(),
                        success: false,
                        error: Some(e.to_string()),
                        size: 0,
                    });
                }
                Err(e) => {
                    tracing::error!("Task join error: {}", e);
                }
            }
        }

        Ok(results)
    }

    /// Download a single file with fallback mirrors
    async fn download_file_with_fallbacks(
        file_path: &str,
        urls: &[String],
        progress_sender: &Option<mpsc::UnboundedSender<ProgressEvent>>,
    ) -> Result<DownloadResult, Box<dyn Error + Send + Sync>> {
        let mut last_error: Option<String> = None;

        for (i, url) in urls.iter().enumerate() {
            if i > 0 {
                if let Some(ref sender) = progress_sender {
                    let _ = sender.send(ProgressEvent::MirrorFallback { 
                        file_path: file_path.to_string(),
                        mirror_url: url.clone(),
                    });
                }
            }

            match Self::download_single_file(url, progress_sender).await {
                Ok(content) => {
                    let size = content.len() as u64;
                    return Ok(DownloadResult {
                        file_path: file_path.to_string(),
                        content,
                        success: true,
                        error: None,
                        size,
                    });
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                    tracing::warn!("Failed to download from {}: {}", url, last_error.as_ref().unwrap());
                }
            }
        }

        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("All download attempts failed: {:?}", last_error))) as Box<dyn Error + Send + Sync>)
    }

    /// Download a single file
    async fn download_single_file(
        url: &str,
        progress_sender: &Option<mpsc::UnboundedSender<ProgressEvent>>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let response = reqwest::get(url).await?;
        
        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()).into());
        }

        let total_bytes = response.content_length();
        let content = response.bytes().await?;
        
        if let Some(ref sender) = progress_sender {
            let _ = sender.send(ProgressEvent::DownloadProgress {
                file_path: url.to_string(),
                bytes_downloaded: content.len() as u64,
                total_bytes,
            });
        }

        Ok(content.to_vec())
    }
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
