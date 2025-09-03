use crate::error::{GuardianError, utils as error_utils};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs;
use tokio::sync::RwLock;
use tokio::time::{interval, sleep};
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use zip::ZipWriter;
use std::io::Write;

/// Backup strategy types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackupStrategy {
    /// Full backup of all data
    Full,
    /// Incremental backup of changed files only
    Incremental,
    /// Differential backup since last full backup
    Differential,
    /// Snapshot backup for instant recovery
    Snapshot,
}

/// Backup retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Keep daily backups for this many days
    pub daily_retention_days: u32,
    /// Keep weekly backups for this many weeks
    pub weekly_retention_weeks: u32,
    /// Keep monthly backups for this many months
    pub monthly_retention_months: u32,
    /// Keep yearly backups for this many years
    pub yearly_retention_years: u32,
    /// Maximum number of backups to keep (0 = unlimited)
    pub max_backups: u32,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            daily_retention_days: 7,
            weekly_retention_weeks: 4,
            monthly_retention_months: 12,
            yearly_retention_years: 5,
            max_backups: 0,
        }
    }
}

/// Backup storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Local storage path
    pub local_path: PathBuf,
    /// Remote storage configuration
    pub remote: Option<RemoteStorageConfig>,
    /// Compression level (0-9)
    pub compression_level: u8,
    /// Encryption enabled
    pub encryption_enabled: bool,
    /// Encryption key (should be stored securely)
    pub encryption_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteStorageConfig {
    /// Storage type
    pub storage_type: RemoteStorageType,
    /// Connection configuration
    pub config: HashMap<String, String>,
    /// Upload timeout
    pub upload_timeout: Duration,
    /// Download timeout
    pub download_timeout: Duration,
    /// Retry attempts
    pub retry_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RemoteStorageType {
    S3,
    AzureBlob,
    GoogleCloud,
    FTP,
    SFTP,
    WebDAV,
}

/// Backup metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    pub backup_id: String,
    pub backup_type: BackupStrategy,
    pub created_at: u64,
    pub size_bytes: u64,
    pub file_count: u32,
    pub checksum: String,
    pub parent_backup_id: Option<String>,
    pub tags: HashMap<String, String>,
    pub compression_ratio: f64,
    pub encryption_enabled: bool,
}

/// Backup result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupResult {
    pub backup_id: String,
    pub success: bool,
    pub message: String,
    pub duration_seconds: u64,
    pub size_bytes: u64,
    pub file_count: u32,
    pub error: Option<String>,
    pub metadata: BackupMetadata,
}

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Backup strategy
    pub strategy: BackupStrategy,
    /// Retention policy
    pub retention: RetentionPolicy,
    /// Storage configuration
    pub storage: StorageConfig,
    /// Backup schedule (cron expression)
    pub schedule: String,
    /// Enabled
    pub enabled: bool,
    /// Backup paths to include
    pub include_paths: Vec<PathBuf>,
    /// Paths to exclude
    pub exclude_paths: Vec<PathBuf>,
    /// Maximum backup size (0 = unlimited)
    pub max_size_bytes: u64,
    /// Parallel compression threads
    pub compression_threads: u32,
}

/// Backup manager for the Guardian Platform
pub struct BackupManager {
    config: BackupConfig,
    backups: Arc<RwLock<HashMap<String, BackupMetadata>>>,
    is_running: Arc<RwLock<bool>>,
    scheduler: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl BackupManager {
    pub fn new(config: BackupConfig) -> Self {
        Self {
            config,
            backups: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
            scheduler: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the backup scheduler
    pub async fn start(&self) -> Result<(), GuardianError> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(error_utils::internal_error(
                "backup_manager",
                "start",
                "Backup manager is already running",
            ));
        }
        *is_running = true;
        drop(is_running);

        info!("Starting backup manager with strategy: {:?}", self.config.strategy);

        // Create backup directory if it doesn't exist
        fs::create_dir_all(&self.config.storage.local_path)
            .await
            .map_err(|e| error_utils::storage_error(
                "create_directory",
                Some(&self.config.storage.local_path.to_string_lossy()),
                &format!("Failed to create backup directory: {}", e),
                false,
            ))?;

        // Start scheduler
        let config = self.config.clone();
        let backups = self.backups.clone();
        let is_running = self.is_running.clone();

        let scheduler_handle = tokio::spawn(async move {
            Self::run_scheduler(config, backups, is_running).await;
        });

        {
            let mut scheduler = self.scheduler.write().await;
            *scheduler = Some(scheduler_handle);
        }

        Ok(())
    }

    /// Stop the backup scheduler
    pub async fn stop(&self) {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        drop(is_running);

        // Wait for scheduler to finish
        let mut scheduler = self.scheduler.write().await;
        if let Some(handle) = scheduler.take() {
            handle.abort();
        }

        info!("Backup manager stopped");
    }

    /// Run backup scheduler
    async fn run_scheduler(
        config: BackupConfig,
        backups: Arc<RwLock<HashMap<String, BackupMetadata>>>,
        is_running: Arc<RwLock<bool>>,
    ) {
        let mut interval_timer = interval(Duration::from_secs(60)); // Check every minute

        while *is_running.read().await {
            interval_timer.tick().await;

            if !config.enabled {
                continue;
            }

            // Check if it's time for a backup based on schedule
            if Self::should_run_backup(&config.schedule).await {
                let manager = BackupManager {
                    config: config.clone(),
                    backups: backups.clone(),
                    is_running: is_running.clone(),
                    scheduler: Arc::new(RwLock::new(None)),
                };

                match manager.create_backup().await {
                    Ok(result) => {
                        info!("Scheduled backup completed: {}", result.backup_id);
                    }
                    Err(e) => {
                        error!("Scheduled backup failed: {}", e);
                    }
                }
            }
        }
    }

    /// Check if backup should run based on schedule
    async fn should_run_backup(schedule: &str) -> bool {
        // This is a simplified implementation
        // In a real implementation, you would use a proper cron parser
        // For now, we'll run backups every hour
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        now % 3600 == 0 // Every hour
    }

    /// Create a backup
    pub async fn create_backup(&self) -> Result<BackupResult, GuardianError> {
        let backup_id = Uuid::new_v4().to_string();
        let start_time = SystemTime::now();

        info!("Starting backup: {}", backup_id);

        // Determine backup strategy
        let backup_strategy = self.determine_backup_strategy().await?;

        // Create backup
        let result = match backup_strategy {
            BackupStrategy::Full => self.create_full_backup(&backup_id).await,
            BackupStrategy::Incremental => self.create_incremental_backup(&backup_id).await,
            BackupStrategy::Differential => self.create_differential_backup(&backup_id).await,
            BackupStrategy::Snapshot => self.create_snapshot_backup(&backup_id).await,
        };

        let duration = start_time.elapsed().unwrap_or_default().as_secs();

        match result {
            Ok(metadata) => {
                // Store backup metadata
                {
                    let mut backups = self.backups.write().await;
                    backups.insert(backup_id.clone(), metadata.clone());
                }

                // Upload to remote storage if configured
                if let Some(remote_config) = &self.config.storage.remote {
                    self.upload_to_remote(&backup_id, &metadata).await?;
                }

                // Clean up old backups
                self.cleanup_old_backups().await?;

                info!("Backup completed successfully: {}", backup_id);

                Ok(BackupResult {
                    backup_id,
                    success: true,
                    message: "Backup completed successfully".to_string(),
                    duration_seconds: duration,
                    size_bytes: metadata.size_bytes,
                    file_count: metadata.file_count,
                    error: None,
                    metadata,
                })
            }
            Err(e) => {
                error!("Backup failed: {}", e);
                Ok(BackupResult {
                    backup_id,
                    success: false,
                    message: format!("Backup failed: {}", e),
                    duration_seconds: duration,
                    size_bytes: 0,
                    file_count: 0,
                    error: Some(e.to_string()),
                    metadata: BackupMetadata {
                        backup_id: backup_id.clone(),
                        backup_type: backup_strategy,
                        created_at: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                        size_bytes: 0,
                        file_count: 0,
                        checksum: String::new(),
                        parent_backup_id: None,
                        tags: HashMap::new(),
                        compression_ratio: 0.0,
                        encryption_enabled: self.config.storage.encryption_enabled,
                    },
                })
            }
        }
    }

    /// Determine which backup strategy to use
    async fn determine_backup_strategy(&self) -> Result<BackupStrategy, GuardianError> {
        let backups = self.backups.read().await;
        
        // Check if we need a full backup
        let last_full_backup = backups.values()
            .filter(|b| b.backup_type == BackupStrategy::Full)
            .max_by_key(|b| b.created_at);

        match last_full_backup {
            Some(last_full) => {
                let days_since_full = (SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() - last_full.created_at) / 86400;

                if days_since_full >= 7 {
                    // Full backup if it's been more than a week
                    Ok(BackupStrategy::Full)
                } else {
                    // Use configured strategy for regular backups
                    Ok(self.config.strategy.clone())
                }
            }
            None => {
                // First backup should always be full
                Ok(BackupStrategy::Full)
            }
        }
    }

    /// Create a full backup
    async fn create_full_backup(&self, backup_id: &str) -> Result<BackupMetadata, GuardianError> {
        let backup_path = self.config.storage.local_path.join(format!("{}.zip", backup_id));
        
        // Create backup archive
        let file = fs::File::create(&backup_path).await
            .map_err(|e| error_utils::storage_error(
                "create_file",
                Some(&backup_path.to_string_lossy()),
                &format!("Failed to create backup file: {}", e),
                false,
            ))?;

        let mut zip = ZipWriter::new(file);
        let mut file_count = 0;
        let mut total_size = 0;

        // Add files to archive
        for path in &self.config.include_paths {
            if path.exists() {
                self.add_directory_to_zip(&mut zip, path, &mut file_count, &mut total_size).await?;
            }
        }

        zip.finish()
            .map_err(|e| error_utils::storage_error(
                "finish_zip",
                None,
                &format!("Failed to finish zip archive: {}", e),
                false,
            ))?;

        // Calculate checksum
        let checksum = self.calculate_checksum(&backup_path).await?;

        // Get final file size
        let metadata = fs::metadata(&backup_path).await
            .map_err(|e| error_utils::storage_error(
                "get_metadata",
                Some(&backup_path.to_string_lossy()),
                &format!("Failed to get file metadata: {}", e),
                false,
            ))?;

        Ok(BackupMetadata {
            backup_id: backup_id.to_string(),
            backup_type: BackupStrategy::Full,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            size_bytes: metadata.len(),
            file_count,
            checksum,
            parent_backup_id: None,
            tags: HashMap::new(),
            compression_ratio: if total_size > 0 {
                metadata.len() as f64 / total_size as f64
            } else {
                1.0
            },
            encryption_enabled: self.config.storage.encryption_enabled,
        })
    }

    /// Create an incremental backup
    async fn create_incremental_backup(&self, backup_id: &str) -> Result<BackupMetadata, GuardianError> {
        // Find the last backup to compare against
        let last_backup = {
            let backups = self.backups.read().await;
            backups.values()
                .max_by_key(|b| b.created_at)
                .cloned()
        };

        match last_backup {
            Some(last) => {
                // Create incremental backup based on last backup
                self.create_backup_since(&backup_id, &last).await
            }
            None => {
                // No previous backup, create full backup
                self.create_full_backup(backup_id).await
            }
        }
    }

    /// Create a differential backup
    async fn create_differential_backup(&self, backup_id: &str) -> Result<BackupMetadata, GuardianError> {
        // Find the last full backup
        let last_full_backup = {
            let backups = self.backups.read().await;
            backups.values()
                .filter(|b| b.backup_type == BackupStrategy::Full)
                .max_by_key(|b| b.created_at)
                .cloned()
        };

        match last_full_backup {
            Some(last_full) => {
                // Create differential backup based on last full backup
                self.create_backup_since(&backup_id, &last_full).await
            }
            None => {
                // No full backup exists, create full backup
                self.create_full_backup(backup_id).await
            }
        }
    }

    /// Create a snapshot backup
    async fn create_snapshot_backup(&self, backup_id: &str) -> Result<BackupMetadata, GuardianError> {
        // Snapshot backups are typically filesystem-level snapshots
        // For now, we'll create a full backup as a snapshot
        self.create_full_backup(backup_id).await
    }

    /// Create backup since a specific backup
    async fn create_backup_since(&self, backup_id: &str, since_backup: &BackupMetadata) -> Result<BackupMetadata, GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would track file modification times
        // and only backup files that have changed since the reference backup
        
        // For now, create a full backup
        self.create_full_backup(backup_id).await
    }

    /// Add directory to zip archive
    async fn add_directory_to_zip(
        &self,
        zip: &mut ZipWriter<fs::File>,
        path: &Path,
        file_count: &mut u32,
        total_size: &mut u64,
    ) -> Result<(), GuardianError> {
        if path.is_file() {
            self.add_file_to_zip(zip, path, file_count, total_size).await?;
        } else if path.is_dir() {
            let mut entries = fs::read_dir(path).await
                .map_err(|e| error_utils::storage_error(
                    "read_directory",
                    Some(&path.to_string_lossy()),
                    &format!("Failed to read directory: {}", e),
                    false,
                ))?;

            while let Some(entry) = entries.next_entry().await
                .map_err(|e| error_utils::storage_error(
                    "read_directory_entry",
                    Some(&path.to_string_lossy()),
                    &format!("Failed to read directory entry: {}", e),
                    false,
                ))? {
                let entry_path = entry.path();
                
                // Check if path should be excluded
                if self.should_exclude_path(&entry_path) {
                    continue;
                }

                self.add_directory_to_zip(zip, &entry_path, file_count, total_size).await?;
            }
        }

        Ok(())
    }

    /// Add file to zip archive
    async fn add_file_to_zip(
        &self,
        zip: &mut ZipWriter<fs::File>,
        path: &Path,
        file_count: &mut u32,
        total_size: &mut u64,
    ) -> Result<(), GuardianError> {
        let file_name = path.to_string_lossy();
        let file_data = fs::read(path).await
            .map_err(|e| error_utils::storage_error(
                "read_file",
                Some(&file_name),
                &format!("Failed to read file: {}", e),
                false,
            ))?;

        zip.start_file(file_name, zip::write::FileOptions::default())
            .map_err(|e| error_utils::storage_error(
                "start_zip_file",
                Some(&file_name),
                &format!("Failed to start zip file: {}", e),
                false,
            ))?;

        zip.write_all(&file_data)
            .map_err(|e| error_utils::storage_error(
                "write_zip_file",
                Some(&file_name),
                &format!("Failed to write zip file: {}", e),
                false,
            ))?;

        *file_count += 1;
        *total_size += file_data.len() as u64;

        Ok(())
    }

    /// Check if path should be excluded
    fn should_exclude_path(&self, path: &Path) -> bool {
        for exclude_path in &self.config.exclude_paths {
            if path.starts_with(exclude_path) {
                return true;
            }
        }
        false
    }

    /// Calculate file checksum
    async fn calculate_checksum(&self, path: &Path) -> Result<String, GuardianError> {
        use sha2::{Sha256, Digest};
        
        let file_data = fs::read(path).await
            .map_err(|e| error_utils::storage_error(
                "read_file",
                Some(&path.to_string_lossy()),
                &format!("Failed to read file for checksum: {}", e),
                false,
            ))?;

        let mut hasher = Sha256::new();
        hasher.update(&file_data);
        let result = hasher.finalize();
        
        Ok(format!("{:x}", result))
    }

    /// Upload backup to remote storage
    async fn upload_to_remote(&self, backup_id: &str, metadata: &BackupMetadata) -> Result<(), GuardianError> {
        let remote_config = self.config.storage.remote.as_ref()
            .ok_or_else(|| error_utils::config_error(
                "remote_storage",
                "Remote storage not configured",
                None,
            ))?;

        let local_path = self.config.storage.local_path.join(format!("{}.zip", backup_id));
        
        match remote_config.storage_type {
            RemoteStorageType::S3 => {
                self.upload_to_s3(&local_path, backup_id, remote_config).await
            }
            RemoteStorageType::AzureBlob => {
                self.upload_to_azure_blob(&local_path, backup_id, remote_config).await
            }
            RemoteStorageType::GoogleCloud => {
                self.upload_to_google_cloud(&local_path, backup_id, remote_config).await
            }
            RemoteStorageType::FTP => {
                self.upload_to_ftp(&local_path, backup_id, remote_config).await
            }
            RemoteStorageType::SFTP => {
                self.upload_to_sftp(&local_path, backup_id, remote_config).await
            }
            RemoteStorageType::WebDAV => {
                self.upload_to_webdav(&local_path, backup_id, remote_config).await
            }
        }
    }

    /// Upload to S3
    async fn upload_to_s3(&self, local_path: &Path, backup_id: &str, config: &RemoteStorageConfig) -> Result<(), GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would use the AWS SDK
        info!("Uploading backup {} to S3", backup_id);
        Ok(())
    }

    /// Upload to Azure Blob Storage
    async fn upload_to_azure_blob(&self, local_path: &Path, backup_id: &str, config: &RemoteStorageConfig) -> Result<(), GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would use the Azure SDK
        info!("Uploading backup {} to Azure Blob Storage", backup_id);
        Ok(())
    }

    /// Upload to Google Cloud Storage
    async fn upload_to_google_cloud(&self, local_path: &Path, backup_id: &str, config: &RemoteStorageConfig) -> Result<(), GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would use the Google Cloud SDK
        info!("Uploading backup {} to Google Cloud Storage", backup_id);
        Ok(())
    }

    /// Upload to FTP
    async fn upload_to_ftp(&self, local_path: &Path, backup_id: &str, config: &RemoteStorageConfig) -> Result<(), GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would use an FTP client
        info!("Uploading backup {} to FTP", backup_id);
        Ok(())
    }

    /// Upload to SFTP
    async fn upload_to_sftp(&self, local_path: &Path, backup_id: &str, config: &RemoteStorageConfig) -> Result<(), GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would use an SFTP client
        info!("Uploading backup {} to SFTP", backup_id);
        Ok(())
    }

    /// Upload to WebDAV
    async fn upload_to_webdav(&self, local_path: &Path, backup_id: &str, config: &RemoteStorageConfig) -> Result<(), GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would use a WebDAV client
        info!("Uploading backup {} to WebDAV", backup_id);
        Ok(())
    }

    /// Clean up old backups based on retention policy
    async fn cleanup_old_backups(&self) -> Result<(), GuardianError> {
        let mut backups = self.backups.write().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut to_remove = Vec::new();

        for (backup_id, metadata) in backups.iter() {
            if self.should_remove_backup(metadata, now) {
                to_remove.push(backup_id.clone());
            }
        }

        for backup_id in to_remove {
            if let Some(metadata) = backups.remove(&backup_id) {
                // Remove local backup file
                let backup_path = self.config.storage.local_path.join(format!("{}.zip", backup_id));
                if let Err(e) = fs::remove_file(&backup_path).await {
                    warn!("Failed to remove backup file {}: {}", backup_path.display(), e);
                }

                // Remove from remote storage if configured
                if let Some(remote_config) = &self.config.storage.remote {
                    if let Err(e) = self.remove_from_remote(&backup_id, remote_config).await {
                        warn!("Failed to remove backup from remote storage {}: {}", backup_id, e);
                    }
                }

                info!("Removed old backup: {}", backup_id);
            }
        }

        Ok(())
    }

    /// Check if backup should be removed based on retention policy
    fn should_remove_backup(&self, metadata: &BackupMetadata, now: u64) -> bool {
        let age_seconds = now - metadata.created_at;
        let age_days = age_seconds / 86400;

        match metadata.backup_type {
            BackupStrategy::Full => {
                // Keep full backups based on yearly retention
                age_days > (self.config.retention.yearly_retention_years * 365) as u64
            }
            BackupStrategy::Incremental | BackupStrategy::Differential => {
                // Keep incremental/differential backups based on daily retention
                age_days > self.config.retention.daily_retention_days as u64
            }
            BackupStrategy::Snapshot => {
                // Keep snapshots for a shorter period
                age_days > 1
            }
        }
    }

    /// Remove backup from remote storage
    async fn remove_from_remote(&self, backup_id: &str, config: &RemoteStorageConfig) -> Result<(), GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would use the appropriate SDK
        info!("Removing backup {} from remote storage", backup_id);
        Ok(())
    }

    /// Restore from backup
    pub async fn restore_from_backup(&self, backup_id: &str, target_path: &Path) -> Result<(), GuardianError> {
        let metadata = {
            let backups = self.backups.read().await;
            backups.get(backup_id).cloned()
                .ok_or_else(|| error_utils::resource_error(
                    crate::error::ResourceErrorKind::NotFound,
                    "backup",
                    backup_id,
                    "Backup not found",
                ))?
        };

        info!("Starting restore from backup: {}", backup_id);

        // Download from remote storage if not available locally
        let backup_path = self.config.storage.local_path.join(format!("{}.zip", backup_id));
        if !backup_path.exists() {
            if let Some(remote_config) = &self.config.storage.remote {
                self.download_from_remote(backup_id, &backup_path, remote_config).await?;
            } else {
                return Err(error_utils::resource_error(
                    crate::error::ResourceErrorKind::NotFound,
                    "backup_file",
                    &backup_path.to_string_lossy(),
                    "Backup file not found locally and no remote storage configured",
                ));
            }
        }

        // Verify backup integrity
        let checksum = self.calculate_checksum(&backup_path).await?;
        if checksum != metadata.checksum {
            return Err(error_utils::resource_error(
                crate::error::ResourceErrorKind::Corrupted,
                "backup",
                backup_id,
                "Backup checksum verification failed",
            ));
        }

        // Extract backup
        self.extract_backup(&backup_path, target_path).await?;

        info!("Restore completed successfully from backup: {}", backup_id);
        Ok(())
    }

    /// Download backup from remote storage
    async fn download_from_remote(&self, backup_id: &str, local_path: &Path, config: &RemoteStorageConfig) -> Result<(), GuardianError> {
        // This is a simplified implementation
        // In a real implementation, you would use the appropriate SDK
        info!("Downloading backup {} from remote storage", backup_id);
        Ok(())
    }

    /// Extract backup archive
    async fn extract_backup(&self, backup_path: &Path, target_path: &Path) -> Result<(), GuardianError> {
        use zip::ZipArchive;
        use std::io::Read;

        let file = std::fs::File::open(backup_path)
            .map_err(|e| error_utils::storage_error(
                "open_file",
                Some(&backup_path.to_string_lossy()),
                &format!("Failed to open backup file: {}", e),
                false,
            ))?;

        let mut archive = ZipArchive::new(file)
            .map_err(|e| error_utils::storage_error(
                "open_zip",
                Some(&backup_path.to_string_lossy()),
                &format!("Failed to open zip archive: {}", e),
                false,
            ))?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| error_utils::storage_error(
                    "read_zip_entry",
                    None,
                    &format!("Failed to read zip entry: {}", e),
                    false,
                ))?;

            let outpath = target_path.join(file.name());
            
            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath).await
                    .map_err(|e| error_utils::storage_error(
                        "create_directory",
                        Some(&outpath.to_string_lossy()),
                        &format!("Failed to create directory: {}", e),
                        false,
                    ))?;
            } else {
                if let Some(p) = outpath.parent() {
                    fs::create_dir_all(p).await
                        .map_err(|e| error_utils::storage_error(
                            "create_directory",
                            Some(&p.to_string_lossy()),
                            &format!("Failed to create parent directory: {}", e),
                            false,
                        ))?;
                }

                let mut outfile = fs::File::create(&outpath).await
                    .map_err(|e| error_utils::storage_error(
                        "create_file",
                        Some(&outpath.to_string_lossy()),
                        &format!("Failed to create file: {}", e),
                        false,
                    ))?;

                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)
                    .map_err(|e| error_utils::storage_error(
                        "read_file",
                        None,
                        &format!("Failed to read file data: {}", e),
                        false,
                    ))?;

                tokio::io::AsyncWriteExt::write_all(&mut outfile, &buffer).await
                    .map_err(|e| error_utils::storage_error(
                        "write_file",
                        Some(&outpath.to_string_lossy()),
                        &format!("Failed to write file: {}", e),
                        false,
                    ))?;
            }
        }

        Ok(())
    }

    /// List available backups
    pub async fn list_backups(&self) -> Vec<BackupMetadata> {
        let backups = self.backups.read().await;
        backups.values().cloned().collect()
    }

    /// Get backup metadata
    pub async fn get_backup_metadata(&self, backup_id: &str) -> Option<BackupMetadata> {
        let backups = self.backups.read().await;
        backups.get(backup_id).cloned()
    }

    /// Delete backup
    pub async fn delete_backup(&self, backup_id: &str) -> Result<(), GuardianError> {
        let metadata = {
            let mut backups = self.backups.write().await;
            backups.remove(backup_id)
        };

        if let Some(metadata) = metadata {
            // Remove local backup file
            let backup_path = self.config.storage.local_path.join(format!("{}.zip", backup_id));
            if let Err(e) = fs::remove_file(&backup_path).await {
                warn!("Failed to remove backup file {}: {}", backup_path.display(), e);
            }

            // Remove from remote storage if configured
            if let Some(remote_config) = &self.config.storage.remote {
                if let Err(e) = self.remove_from_remote(backup_id, remote_config).await {
                    warn!("Failed to remove backup from remote storage {}: {}", backup_id, e);
                }
            }

            info!("Deleted backup: {}", backup_id);
        }

        Ok(())
    }
}
