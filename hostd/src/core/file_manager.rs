use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use tokio::fs;

use crate::core::{
    config::MinecraftConfig,
    error_handler::{AppError, Result},
};

#[derive(Debug)]
pub struct FileManager {
    config: MinecraftConfig,
}

impl FileManager {
    pub async fn new(config: &MinecraftConfig) -> Result<Self> {
        // Ensure all directories exist
        let directories = vec![
            &config.server_jar_directory,
            &config.world_directory,
            &config.mods_directory,
            &config.config_directory,
            &config.logs_directory,
            &config.backups_directory,
        ];
        
        for dir in directories {
            if !dir.exists() {
                tokio::fs::create_dir_all(dir).await
                    .map_err(|e| AppError::FileSystemError {
                        message: format!("Failed to create directory {:?}: {}", dir, e),
                        path: dir.to_string_lossy().to_string(),
                        operation: "create".to_string(),
                    })?;
            }
        }
        
        Ok(Self {
            config: config.clone(),
        })
    }
    
    pub async fn create_server_directory(&self, server_id: Uuid) -> Result<()> {
        let server_dir = self.get_server_directory(server_id);
        
        if !server_dir.exists() {
            fs::create_dir_all(&server_dir).await
                .map_err(|e| AppError::FileSystemError {
                    message: format!("Failed to create server directory: {}", e),
                    path: server_dir.to_string_lossy().to_string(),
                    operation: "create".to_string(),
                })?;
        }
        
        // Create subdirectories
        let subdirs = vec![
            "world",
            "mods",
            "config",
            "logs",
            "backups",
            "plugins",
        ];
        
        for subdir in subdirs {
            let subdir_path = server_dir.join(subdir);
            if !subdir_path.exists() {
                fs::create_dir_all(&subdir_path).await
                    .map_err(|e| AppError::FileSystemError {
                        message: format!("Failed to create subdirectory {}: {}", subdir, e),
                        path: subdir.to_string(),
                        operation: "create".to_string(),
                    })?;
            }
        }
        
        Ok(())
    }
    
    pub async fn delete_server_directory(&self, server_id: Uuid) -> Result<()> {
        let server_dir = self.get_server_directory(server_id);
        
        if server_dir.exists() {
            fs::remove_dir_all(&server_dir).await
                .map_err(|e| AppError::FileSystemError {
                    message: format!("Failed to delete server directory: {}", e),
                    path: server_dir.to_string_lossy().to_string(),
                    operation: "delete".to_string(),
                })?;
        }
        
        Ok(())
    }
    
    pub fn get_server_directory(&self, server_id: Uuid) -> PathBuf {
        self.config.server_jar_directory.join(server_id.to_string())
    }
    
    pub fn get_world_directory(&self, server_id: Uuid) -> PathBuf {
        self.get_server_directory(server_id).join("world")
    }
    
    pub fn get_mods_directory(&self, server_id: Uuid) -> PathBuf {
        self.get_server_directory(server_id).join("mods")
    }
    
    pub fn get_config_directory(&self, server_id: Uuid) -> PathBuf {
        self.get_server_directory(server_id).join("config")
    }
    
    pub fn get_logs_directory(&self, server_id: Uuid) -> PathBuf {
        self.get_server_directory(server_id).join("logs")
    }
    
    pub fn get_backups_directory(&self, server_id: Uuid) -> PathBuf {
        self.get_server_directory(server_id).join("backups")
    }
    
    pub async fn create_backup(&self, server_id: Uuid, backup_name: &str) -> Result<PathBuf> {
        let world_dir = self.get_world_directory(server_id);
        let backup_dir = self.get_backups_directory(server_id);
        let backup_path = backup_dir.join(backup_name);
        
        if !backup_dir.exists() {
            fs::create_dir_all(&backup_dir).await
                .map_err(|e| AppError::FileSystemError {
                    message: format!("Failed to create backup directory: {}", e),
                    path: backup_dir.to_string_lossy().to_string(),
                    operation: "create".to_string(),
                })?;
        }
        
        // Copy world directory to backup
        if world_dir.exists() {
            self.copy_directory(&world_dir, &backup_path).await?;
        }
        
        Ok(backup_path)
    }
    
    pub async fn restore_backup(&self, server_id: Uuid, backup_name: &str) -> Result<()> {
        let world_dir = self.get_world_directory(server_id);
        let backup_dir = self.get_backups_directory(server_id);
        let backup_path = backup_dir.join(backup_name);
        
        if !backup_path.exists() {
            return Err(AppError::FileSystemError {
                message: format!("Backup {} not found", backup_name),
                path: backup_path.to_string_lossy().to_string(),
                operation: "restore".to_string(),
            });
        }
        
        // Remove existing world directory
        if world_dir.exists() {
            fs::remove_dir_all(&world_dir).await
                .map_err(|e| AppError::FileSystemError {
                    message: format!("Failed to remove existing world: {}", e),
                    path: world_dir.to_string_lossy().to_string(),
                    operation: "remove".to_string(),
                })?;
        }
        
        // Restore from backup
        self.copy_directory(&backup_path, &world_dir).await?;
        
        Ok(())
    }
    
    pub async fn list_backups(&self, server_id: Uuid) -> Result<Vec<BackupInfo>> {
        let backup_dir = self.get_backups_directory(server_id);
        
        if !backup_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut entries = fs::read_dir(&backup_dir).await
            .map_err(|e| AppError::FileSystemError {
                message: format!("Failed to read backup directory: {}", e),
                path: backup_dir.to_string_lossy().to_string(),
                operation: "read".to_string(),
            })?;
        
        let mut backups = Vec::new();
        
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| AppError::FileSystemError {
                message: format!("Failed to read directory entry: {}", e),
                path: backup_dir.to_string_lossy().to_string(),
                operation: "read".to_string(),
            })? {
            
            let path = entry.path();
            if path.is_dir() {
                let metadata = entry.metadata().await
                    .map_err(|e| AppError::FileSystemError {
                        message: format!("Failed to read metadata: {}", e),
                        path: path.to_string_lossy().to_string(),
                        operation: "read".to_string(),
                    })?;
                
                let backup_info = BackupInfo {
                    name: path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    path: path.clone(),
                    size_bytes: self.get_directory_size(&path).await?,
                    created_at: metadata.created()
                        .map_err(|e| AppError::FileSystemError {
                            message: format!("Failed to get creation time: {}", e),
                            path: path.to_string_lossy().to_string(),
                            operation: "get_creation_time".to_string(),
                        })?
                        .into(),
                };
                
                backups.push(backup_info);
            }
        }
        
        // Sort by creation time (newest first)
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(backups)
    }
    
    pub async fn delete_backup(&self, server_id: Uuid, backup_name: &str) -> Result<()> {
        let backup_dir = self.get_backups_directory(server_id);
        let backup_path = backup_dir.join(backup_name);
        
        if backup_path.exists() {
            fs::remove_dir_all(&backup_path).await
                .map_err(|e| AppError::FileSystemError {
                    message: format!("Failed to delete backup: {}", e),
                    path: backup_path.to_string_lossy().to_string(),
                    operation: "delete".to_string(),
                })?;
        }
        
        Ok(())
    }
    
    async fn copy_directory(&self, src: &Path, dst: &Path) -> Result<()> {
        if !src.exists() {
            return Err(AppError::FileSystemError {
                message: "Source directory does not exist".to_string(),
                path: src.to_string_lossy().to_string(),
                operation: "copy".to_string(),
            });
        }
        
        if !dst.parent().unwrap().exists() {
            fs::create_dir_all(dst.parent().unwrap()).await
                .map_err(|e| AppError::FileSystemError {
                    message: format!("Failed to create parent directory: {}", e),
                    path: dst.parent().unwrap().to_string_lossy().to_string(),
                    operation: "create".to_string(),
                })?;
        }
        
        self.copy_directory_recursive(src, dst).await?;
        
        Ok(())
    }
    
    fn copy_directory_recursive(&self, src: &Path, dst: &Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        let src = src.to_path_buf();
        let dst = dst.to_path_buf();
        Box::pin(async move {
        let src_str = src.to_string_lossy().to_string();
        let mut entries = fs::read_dir(src).await
            .map_err(|e| AppError::FileSystemError {
                message: format!("Failed to read source directory: {}", e),
                path: src_str.clone(),
                operation: "read".to_string(),
            })?;
        
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| AppError::FileSystemError {
                message: format!("Failed to read directory entry: {}", e),
                path: src_str.clone(),
                operation: "read".to_string(),
            })? {
            
            let path = entry.path();
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| AppError::FileSystemError {
                    message: "Invalid file name".to_string(),
                    path: path.to_string_lossy().to_string(),
                    operation: "read".to_string(),
                })?;
            
            let dest_path = dst.join(file_name);
            
            if path.is_dir() {
                fs::create_dir_all(&dest_path).await
                    .map_err(|e| AppError::FileSystemError {
                        message: format!("Failed to create directory: {}", e),
                        path: dest_path.to_string_lossy().to_string(),
                        operation: "create".to_string(),
                    })?;
                self.copy_directory_recursive(&path, &dest_path).await?;
            } else {
                fs::copy(&path, &dest_path).await
                    .map_err(|e| AppError::FileSystemError {
                        message: format!("Failed to copy file: {}", e),
                        path: path.to_string_lossy().to_string(),
                        operation: "copy".to_string(),
                    })?;
            }
        }
        
        Ok(())
        })
    }
    
    fn get_directory_size(&self, path: &Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<u64>> + Send + '_>> {
        let path = path.to_path_buf();
        Box::pin(async move {
        let mut total_size = 0;
        let path_str = path.to_string_lossy().to_string();
        let mut entries = fs::read_dir(path).await
            .map_err(|e| AppError::FileSystemError {
                message: format!("Failed to read directory: {}", e),
                path: path_str.clone(),
                operation: "read".to_string(),
            })?;
        
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| AppError::FileSystemError {
                message: format!("Failed to read directory entry: {}", e),
                path: path_str.clone(),
                operation: "read".to_string(),
            })? {
            
            let path = entry.path();
            let metadata = entry.metadata().await
                .map_err(|e| AppError::FileSystemError {
                    message: format!("Failed to read metadata: {}", e),
                    path: path.to_string_lossy().to_string(),
                    operation: "read".to_string(),
                })?;
            
            if path.is_dir() {
                total_size += self.get_directory_size(&path).await?;
            } else {
                total_size += metadata.len();
            }
        }
        
        Ok(total_size)
        })
    }
    
    pub async fn write_server_log(&self, server_id: Uuid, level: &str, message: &str) -> Result<()> {
        let logs_dir = self.get_logs_directory(server_id);
        let log_file = logs_dir.join(format!("server_{}.log", chrono::Utc::now().format("%Y-%m-%d")));
        
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        let log_entry = format!("[{}] [{}] {}\n", timestamp, level.to_uppercase(), message);
        
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .await
            .map_err(|e| AppError::FileSystemError {
                message: format!("Failed to open log file: {}", e),
                path: log_file.to_string_lossy().to_string(),
                operation: "open_log_file".to_string(),
            })?
            .write_all(log_entry.as_bytes())
            .await
            .map_err(|e| AppError::FileSystemError {
                message: format!("Failed to write to log file: {}", e),
                path: log_file.to_string_lossy().to_string(),
                operation: "write_log_file".to_string(),
            })?;
        
        Ok(())
    }
    
    pub async fn read_server_logs(&self, server_id: Uuid, lines: Option<usize>) -> Result<Vec<String>> {
        let logs_dir = self.get_logs_directory(server_id);
        let log_file = logs_dir.join(format!("server_{}.log", chrono::Utc::now().format("%Y-%m-%d")));
        
        if !log_file.exists() {
            return Ok(Vec::new());
        }
        
        let content = fs::read_to_string(&log_file).await
            .map_err(|e| AppError::FileSystemError {
                message: format!("Failed to read log file: {}", e),
                path: log_file.to_string_lossy().to_string(),
                operation: "read_log_file".to_string(),
            })?;
        
        let mut log_lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        
        if let Some(n) = lines {
            if log_lines.len() > n {
                log_lines = log_lines.into_iter().rev().take(n).rev().collect();
            }
        }
        
        Ok(log_lines)
    }
}

#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
