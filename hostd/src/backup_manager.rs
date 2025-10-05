use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::fs;
use tokio::fs as async_fs;
use zip::ZipWriter;
use std::io::Write;

/// Backup information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub description: Option<String>,
    pub size: u64,
    pub created_at: DateTime<Utc>,
    pub status: BackupStatus,
    pub backup_type: BackupType,
    pub compression: CompressionType,
    pub includes: BackupIncludes,
    pub metadata: Option<serde_json::Value>,
}

/// Backup status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackupStatus {
    Creating,
    Completed,
    Failed,
    Restoring,
}

/// Backup type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackupType {
    Manual,
    Scheduled,
    Automatic,
}

/// Compression type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompressionType {
    None,
    Gzip,
    Zip,
}

/// What to include in backup
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BackupIncludes {
    pub world: bool,
    pub mods: bool,
    pub config: bool,
    pub logs: bool,
    pub server_properties: bool,
    pub whitelist: bool,
    pub ops: bool,
    pub banned_players: bool,
    pub banned_ips: bool,
}

/// Backup creation request
#[derive(Debug, Deserialize)]
pub struct CreateBackupRequest {
    pub name: String,
    pub description: Option<String>,
    pub backup_type: BackupType,
    pub compression: CompressionType,
    pub includes: BackupIncludes,
    pub metadata: Option<serde_json::Value>,
}

/// Backup restore request
#[derive(Debug, Deserialize)]
pub struct RestoreBackupRequest {
    pub backup_id: String,
    pub restore_world: bool,
    pub restore_mods: bool,
    pub restore_config: bool,
    pub restore_logs: bool,
    pub create_backup: bool,
}

/// Backup schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSchedule {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub enabled: bool,
    pub cron_expression: String,
    pub retention_days: u32,
    pub compression: CompressionType,
    pub includes: BackupIncludes,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Backup statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupStats {
    pub total_backups: u32,
    pub total_size: u64,
    pub oldest_backup: Option<DateTime<Utc>>,
    pub newest_backup: Option<DateTime<Utc>>,
    pub average_size: u64,
    pub success_rate: f64,
    pub last_backup: Option<DateTime<Utc>>,
}

/// Backup manager for handling backup and restore operations
pub struct BackupManager {
    /// Backups by server ID
    backups: Arc<RwLock<HashMap<String, Vec<BackupInfo>>>>,
    /// Backup schedules by server ID
    schedules: Arc<RwLock<HashMap<String, Vec<BackupSchedule>>>>,
    /// Base directory for backups
    backups_base_dir: PathBuf,
    /// Base directory for servers
    servers_base_dir: PathBuf,
}

impl BackupManager {
    pub fn new(backups_base_dir: PathBuf, servers_base_dir: PathBuf) -> Self {
        Self {
            backups: Arc::new(RwLock::new(HashMap::new())),
            schedules: Arc::new(RwLock::new(HashMap::new())),
            backups_base_dir,
            servers_base_dir,
        }
    }

    /// Create a backup for a server
    pub async fn create_backup(
        &self,
        server_id: &str,
        request: CreateBackupRequest,
    ) -> Result<BackupInfo, Box<dyn std::error::Error>> {
        let backup_id = Uuid::new_v4().to_string();
        let created_at = Utc::now();

        // Create backup info
        let mut backup = BackupInfo {
            id: backup_id.clone(),
            server_id: server_id.to_string(),
            name: request.name,
            description: request.description,
            size: 0,
            created_at,
            status: BackupStatus::Creating,
            backup_type: request.backup_type,
            compression: request.compression,
            includes: request.includes,
            metadata: request.metadata,
        };

        // Update status in storage
        {
            let mut backups = self.backups.write().await;
            let server_backups = backups.entry(server_id.to_string()).or_insert_with(Vec::new);
            server_backups.push(backup.clone());
        }

        // Perform backup in background
        let manager = self.clone();
        let server_id = server_id.to_string();
        let backup_id = backup_id.clone();
            tokio::spawn(async move {
                let (success, error_msg) = {
                    let result = manager.perform_backup(&server_id, &backup_id).await;
                    match result {
                        Ok(_) => (true, None),
                        Err(e) => (false, Some(format!("{}", e))),
                    }
                };
                
                if success {
                    let _ = manager.update_backup_status(&server_id, &backup_id, BackupStatus::Completed).await;
                } else {
                    if let Some(msg) = error_msg {
                        tracing::error!("Backup failed for server {}: {}", server_id, msg);
                    }
                    let _ = manager.update_backup_status(&server_id, &backup_id, BackupStatus::Failed).await;
                }
            });

        Ok(backup)
    }

    /// Perform the actual backup operation
    async fn perform_backup(
        &self,
        server_id: &str,
        backup_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Get backup info
        let backup = self.get_backup(server_id, backup_id).await?;
        
        // Create backup directory
        let backup_dir = self.backups_base_dir.join(server_id).join(backup_id);
        async_fs::create_dir_all(&backup_dir).await?;

        // Create backup archive
        let archive_path = backup_dir.join(format!("backup.{}", self.get_compression_extension(&backup.compression)));
        let mut archive = self.create_archive(&archive_path, &backup.compression).await?;

        // Add files to backup
        let server_dir = self.servers_base_dir.join(server_id);
        
        if backup.includes.world {
            self.add_directory_to_archive(&mut archive, &server_dir.join("world"), "world")?;
        }
        
        if backup.includes.mods {
            self.add_directory_to_archive(&mut archive, &server_dir.join("mods"), "mods")?;
        }
        
        if backup.includes.config {
            self.add_directory_to_archive(&mut archive, &server_dir.join("config"), "config")?;
        }
        
        if backup.includes.logs {
            self.add_directory_to_archive(&mut archive, &server_dir.join("logs"), "logs")?;
        }
        
        if backup.includes.server_properties {
            self.add_file_to_archive(&mut archive, &server_dir.join("server.properties"), "server.properties")?;
        }
        
        if backup.includes.whitelist {
            self.add_file_to_archive(&mut archive, &server_dir.join("whitelist.json"), "whitelist.json")?;
        }
        
        if backup.includes.ops {
            self.add_file_to_archive(&mut archive, &server_dir.join("ops.json"), "ops.json")?;
        }
        
        if backup.includes.banned_players {
            self.add_file_to_archive(&mut archive, &server_dir.join("banned-players.json"), "banned-players.json")?;
        }
        
        if backup.includes.banned_ips {
            self.add_file_to_archive(&mut archive, &server_dir.join("banned-ips.json"), "banned-ips.json")?;
        }

        // Finalize archive
        archive.finish()?;

        // Update backup size and status
        let metadata = async_fs::metadata(&archive_path).await?;
        let size = metadata.len();
        
        self.update_backup_size(server_id, backup_id, size).await?;
        self.update_backup_status(server_id, backup_id, BackupStatus::Completed).await?;

        Ok(())
    }

    /// Create archive based on compression type
    async fn create_archive(
        &self,
        path: &Path,
        compression: &CompressionType,
    ) -> Result<ZipWriter<std::fs::File>, Box<dyn std::error::Error>> {
        let file = std::fs::File::create(path)?;
        let zip = ZipWriter::new(file);
        Ok(zip)
    }

    /// Get compression extension
    fn get_compression_extension(&self, compression: &CompressionType) -> &'static str {
        match compression {
            CompressionType::None => "tar",
            CompressionType::Gzip => "tar.gz",
            CompressionType::Zip => "zip",
        }
    }

    /// Add directory to archive
    fn add_directory_to_archive(
        &self,
        archive: &mut ZipWriter<std::fs::File>,
        dir_path: &Path,
        archive_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !dir_path.exists() {
            return Ok(());
        }

        let entries = std::fs::read_dir(dir_path)?;
        for entry in entries {
            let entry = entry?;
            let entry_path = entry.path();
            let relative_path = entry_path.strip_prefix(dir_path)?;
            let archive_entry_path = format!("{}/{}", archive_path, relative_path.to_string_lossy());
            
            if entry.metadata()?.is_file() {
                self.add_file_to_archive(archive, &entry_path, &archive_entry_path)?;
            } else if entry.metadata()?.is_dir() {
                self.add_directory_to_archive(archive, &entry_path, &archive_entry_path)?;
            }
        }

        Ok(())
    }

    /// Add file to archive
    fn add_file_to_archive(
        &self,
        archive: &mut ZipWriter<std::fs::File>,
        file_path: &Path,
        archive_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !file_path.exists() {
            return Ok(());
        }

        let file_data = std::fs::read(file_path)?;
        archive.start_file(archive_path, zip::write::FileOptions::default())?;
        archive.write_all(&file_data)?;

        Ok(())
    }

    /// Restore a backup
    pub async fn restore_backup(
        &self,
        server_id: &str,
        request: RestoreBackupRequest,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let backup = self.get_backup(server_id, &request.backup_id).await?;
        
        if backup.status != BackupStatus::Completed {
            return Err("Backup is not completed".into());
        }

        // Update status to restoring
        self.update_backup_status(server_id, &request.backup_id, BackupStatus::Restoring).await?;

        // Create backup before restore if requested
        if request.create_backup {
            let pre_restore_backup = CreateBackupRequest {
                name: format!("Pre-restore backup for {}", backup.name),
                description: Some("Automatic backup before restore".to_string()),
                backup_type: BackupType::Automatic,
                compression: CompressionType::Zip,
                includes: BackupIncludes {
                    world: true,
                    mods: true,
                    config: true,
                    logs: false,
                    server_properties: true,
                    whitelist: true,
                    ops: true,
                    banned_players: true,
                    banned_ips: true,
                },
                metadata: None,
            };
            self.create_backup(server_id, pre_restore_backup).await?;
        }

        // Restore files
        let backup_dir = self.backups_base_dir.join(server_id).join(&request.backup_id);
        let archive_path = backup_dir.join(format!("backup.{}", self.get_compression_extension(&backup.compression)));
        let server_dir = self.servers_base_dir.join(server_id);

        // Extract archive
        self.extract_archive(&archive_path, &server_dir, &request).await?;

        // Update status back to completed
        self.update_backup_status(server_id, &request.backup_id, BackupStatus::Completed).await?;

        Ok(())
    }

    /// Extract archive
    async fn extract_archive(
        &self,
        archive_path: &Path,
        target_dir: &Path,
        request: &RestoreBackupRequest,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // In a real implementation, this would extract the archive
        // For now, just create placeholder files
        async_fs::create_dir_all(target_dir).await?;

        if request.restore_world {
            async_fs::create_dir_all(target_dir.join("world")).await?;
        }
        
        if request.restore_mods {
            async_fs::create_dir_all(target_dir.join("mods")).await?;
        }
        
        if request.restore_config {
            async_fs::create_dir_all(target_dir.join("config")).await?;
        }
        
        if request.restore_logs {
            async_fs::create_dir_all(target_dir.join("logs")).await?;
        }

        Ok(())
    }

    /// Get backup by ID
    async fn get_backup(&self, server_id: &str, backup_id: &str) -> Result<BackupInfo, Box<dyn std::error::Error>> {
        let backups = self.backups.read().await;
        if let Some(server_backups) = backups.get(server_id) {
            server_backups.iter()
                .find(|b| b.id == backup_id)
                .cloned()
                .ok_or_else(|| "Backup not found".into())
        } else {
            Err("Server not found".into())
        }
    }

    /// Update backup status
    async fn update_backup_status(
        &self,
        server_id: &str,
        backup_id: &str,
        status: BackupStatus,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut backups = self.backups.write().await;
        if let Some(server_backups) = backups.get_mut(server_id) {
            if let Some(backup) = server_backups.iter_mut().find(|b| b.id == backup_id) {
                backup.status = status;
            }
        }
        Ok(())
    }

    /// Update backup size
    async fn update_backup_size(
        &self,
        server_id: &str,
        backup_id: &str,
        size: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut backups = self.backups.write().await;
        if let Some(server_backups) = backups.get_mut(server_id) {
            if let Some(backup) = server_backups.iter_mut().find(|b| b.id == backup_id) {
                backup.size = size;
            }
        }
        Ok(())
    }

    /// Get backups for a server
    pub async fn get_backups(&self, server_id: &str) -> Result<Vec<BackupInfo>, Box<dyn std::error::Error>> {
        let backups = self.backups.read().await;
        Ok(backups.get(server_id).cloned().unwrap_or_default())
    }

    /// Delete backup
    pub async fn delete_backup(&self, server_id: &str, backup_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Remove from storage
        {
            let mut backups = self.backups.write().await;
            if let Some(server_backups) = backups.get_mut(server_id) {
                server_backups.retain(|b| b.id != backup_id);
            }
        }

        // Remove files
        let backup_dir = self.backups_base_dir.join(server_id).join(backup_id);
        if backup_dir.exists() {
            async_fs::remove_dir_all(&backup_dir).await?;
        }

        Ok(())
    }

    /// Get backup statistics
    pub async fn get_backup_stats(&self, server_id: &str) -> Result<BackupStats, Box<dyn std::error::Error>> {
        let backups = self.get_backups(server_id).await?;
        
        let total_backups = backups.len() as u32;
        let total_size: u64 = backups.iter().map(|b| b.size).sum();
        let average_size = if total_backups > 0 { total_size / total_backups as u64 } else { 0 };
        
        let completed_backups = backups.iter().filter(|b| b.status == BackupStatus::Completed).count();
        let success_rate = if total_backups > 0 { completed_backups as f64 / total_backups as f64 } else { 0.0 };
        
        let oldest_backup = backups.iter().map(|b| b.created_at).min();
        let newest_backup = backups.iter().map(|b| b.created_at).max();
        let last_backup = backups.iter()
            .filter(|b| b.status == BackupStatus::Completed)
            .map(|b| b.created_at)
            .max();

        Ok(BackupStats {
            total_backups,
            total_size,
            oldest_backup,
            newest_backup,
            average_size,
            success_rate,
            last_backup,
        })
    }

    /// Create backup schedule
    pub async fn create_backup_schedule(
        &self,
        server_id: &str,
        schedule: serde_json::Value,
    ) -> Result<BackupSchedule, Box<dyn std::error::Error>> {
        let schedule_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let backup_schedule = BackupSchedule {
            id: schedule_id,
            server_id: server_id.to_string(),
            name: schedule.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            enabled: schedule.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true),
            cron_expression: schedule.get("cron_expression").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            retention_days: schedule.get("retention_days").and_then(|v| v.as_u64()).unwrap_or(30) as u32,
            compression: match schedule.get("compression").and_then(|v| v.as_str()).unwrap_or("zip") {
                "gzip" => CompressionType::Gzip,
                "zip" => CompressionType::Zip,
                _ => CompressionType::Zip,
            },
            includes: BackupIncludes::default(),
            last_run: None,
            next_run: None,
            created_at: now,
            updated_at: now,
        };

        {
            let mut schedules = self.schedules.write().await;
            let server_schedules = schedules.entry(server_id.to_string()).or_insert_with(Vec::new);
            server_schedules.push(backup_schedule.clone());
        }

        Ok(backup_schedule)
    }

    /// Get backup schedules for a server
    pub async fn get_backup_schedules(&self, server_id: &str) -> Result<Vec<BackupSchedule>, Box<dyn std::error::Error>> {
        let schedules = self.schedules.read().await;
        Ok(schedules.get(server_id).cloned().unwrap_or_default())
    }

    /// Cleanup old backups
    pub async fn cleanup_old_backups(&self, server_id: &str, retention_days: u32) -> Result<CleanupResult, Box<dyn std::error::Error>> {
        let cutoff_date = Utc::now() - chrono::Duration::days(retention_days as i64);
        let mut deleted = 0;
        let mut freed_space = 0;

        let mut backups = self.backups.write().await;
        if let Some(server_backups) = backups.get_mut(server_id) {
            let old_backups: Vec<_> = server_backups.iter()
                .filter(|b| b.created_at < cutoff_date)
                .cloned()
                .collect();

            for backup in old_backups {
                freed_space += backup.size;
                deleted += 1;
                
                // Remove from list
                server_backups.retain(|b| b.id != backup.id);
                
                // Remove files
                let backup_dir = self.backups_base_dir.join(server_id).join(&backup.id);
                if backup_dir.exists() {
                    let _ = async_fs::remove_dir_all(&backup_dir).await;
                }
            }
        }

        Ok(CleanupResult { deleted, freed_space })
    }
}

/// Omit helper type

/// Cleanup result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    pub deleted: u32,
    pub freed_space: u64,
}

impl Clone for BackupManager {
    fn clone(&self) -> Self {
        Self {
            backups: self.backups.clone(),
            schedules: self.schedules.clone(),
            backups_base_dir: self.backups_base_dir.clone(),
            servers_base_dir: self.servers_base_dir.clone(),
        }
    }
}
