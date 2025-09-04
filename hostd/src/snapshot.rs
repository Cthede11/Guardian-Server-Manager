use crate::config::Config;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use chrono::{DateTime, Utc};
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

/// Snapshot manager for handling world backups and snapshots
pub struct SnapshotManager {
    config: Config,
    snapshots: Arc<RwLock<Vec<SnapshotInfo>>>,
    last_snapshot: Arc<RwLock<Option<DateTime<Utc>>>>,
    is_running: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotInfo {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub description: String,
}

impl SnapshotManager {
    /// Create a new snapshot manager
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing Snapshot Manager...");
        
        // Ensure backup directory exists
        let backup_dir = Path::new(&config.paths.backup_dir);
        if !backup_dir.exists() {
            tokio::fs::create_dir_all(backup_dir).await?;
            info!("Created backup directory: {:?}", backup_dir);
        }
        
        // Load existing snapshots
        let snapshots = Self::load_snapshots(backup_dir).await?;
        
        Ok(Self {
            config,
            snapshots: Arc::new(RwLock::new(snapshots)),
            last_snapshot: Arc::new(RwLock::new(None)),
            is_running: Arc::new(RwLock::new(false)),
        })
    }
    
    /// Start the snapshot manager
    pub async fn start(&self) -> Result<()> {
        info!("Starting Snapshot Manager...");
        
        {
            let mut running_guard = self.is_running.write().await;
            *running_guard = true;
        }
        
        // Start periodic snapshot task
        self.start_periodic_snapshots().await;
        
        info!("Snapshot Manager started");
        Ok(())
    }
    
    /// Stop the snapshot manager
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Snapshot Manager...");
        
        {
            let mut running_guard = self.is_running.write().await;
            *running_guard = false;
        }
        
        info!("Snapshot Manager stopped");
        Ok(())
    }
    
    /// Start periodic snapshot task
    async fn start_periodic_snapshots(&self) {
        let config = self.config.clone();
        let snapshots = self.snapshots.clone();
        let last_snapshot = self.last_snapshot.clone();
        let is_running = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(
                config.ha.autosave_minutes as u64 * 60
            ));
            
            loop {
                interval.tick().await;
                
                // Check if we should still be running
                {
                    let running_guard = is_running.read().await;
                    if !*running_guard {
                        break;
                    }
                }
                
                // Take snapshot if needed
                if let Err(e) = Self::take_snapshot_if_needed_internal(
                    &config,
                    &snapshots,
                    &last_snapshot,
                ).await {
                    error!("Failed to take periodic snapshot: {}", e);
                }
            }
        });
    }
    
    /// Take a snapshot if needed based on configuration
    pub async fn take_snapshot_if_needed(&self) -> Result<()> {
        Self::take_snapshot_if_needed_internal(
            &self.config,
            &self.snapshots,
            &self.last_snapshot,
        ).await
    }
    
    /// Internal method to take snapshot if needed
    async fn take_snapshot_if_needed_internal(
        config: &Config,
        snapshots: &Arc<RwLock<Vec<SnapshotInfo>>>,
        last_snapshot: &Arc<RwLock<Option<DateTime<Utc>>>>,
    ) -> Result<()> {
        let now = Utc::now();
        
        // Check if enough time has passed since last snapshot
        {
            let last_snapshot_guard = last_snapshot.read().await;
            if let Some(last) = last_snapshot_guard.as_ref() {
                let time_since_last = now.signed_duration_since(*last);
                let min_interval = Duration::from_secs(config.ha.autosave_minutes as u64 * 60);
                
                if time_since_last < chrono::Duration::from_std(min_interval).unwrap_or_default() {
                    debug!("Skipping snapshot - not enough time has passed");
                    return Ok(());
                }
            }
        }
        
        // Take the snapshot
        let snapshot_info = Self::create_snapshot_internal(config, "automatic").await?;
        
        // Update last snapshot time
        {
            let mut last_snapshot_guard = last_snapshot.write().await;
            *last_snapshot_guard = Some(now);
        }
        
        // Add to snapshots list
        {
            let mut snapshots_guard = snapshots.write().await;
            snapshots_guard.push(snapshot_info.clone());
            
            // Clean up old snapshots
            Self::cleanup_old_snapshots(&mut snapshots_guard, config.ha.snapshot_keep).await;
        }
        
        info!("Automatic snapshot created: {}", snapshot_info.id);
        Ok(())
    }
    
    /// Create a new snapshot
    pub async fn create_snapshot(&self, description: &str) -> Result<SnapshotInfo> {
        let snapshot_info = Self::create_snapshot_internal(&self.config, description).await?;
        
        // Add to snapshots list
        {
            let mut snapshots_guard = self.snapshots.write().await;
            snapshots_guard.push(snapshot_info.clone());
            
            // Clean up old snapshots
            Self::cleanup_old_snapshots(&mut snapshots_guard, self.config.ha.snapshot_keep).await;
        }
        
        // Update last snapshot time
        {
            let mut last_snapshot_guard = self.last_snapshot.write().await;
            *last_snapshot_guard = Some(Utc::now());
        }
        
        info!("Snapshot created: {} - {}", snapshot_info.id, description);
        Ok(snapshot_info)
    }
    
    /// Internal method to create a snapshot
    async fn create_snapshot_internal(config: &Config, description: &str) -> Result<SnapshotInfo> {
        let snapshot_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();
        
        let world_dir = Path::new(&config.paths.world_dir);
        let backup_dir = Path::new(&config.paths.backup_dir);
        let snapshot_dir = backup_dir.join(&snapshot_id);
        
        // Create snapshot directory
        tokio::fs::create_dir_all(&snapshot_dir).await?;
        
        // Copy world directory to snapshot
        Self::copy_directory(world_dir, &snapshot_dir).await?;
        
        // Calculate snapshot size
        let size_bytes = Self::calculate_directory_size(&snapshot_dir).await?;
        
        Ok(SnapshotInfo {
            id: snapshot_id,
            timestamp,
            path: snapshot_dir,
            size_bytes,
            description: description.to_string(),
        })
    }
    
    /// Restore from a snapshot
    pub async fn restore_snapshot(&self, snapshot_id: &str) -> Result<()> {
        info!("Restoring from snapshot: {}", snapshot_id);
        
        let snapshot_info = {
            let snapshots_guard = self.snapshots.read().await;
            snapshots_guard.iter()
                .find(|s| s.id == snapshot_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Snapshot not found: {}", snapshot_id))?
        };
        
        let world_dir = Path::new(&self.config.paths.world_dir);
        
        // Create backup of current world
        let backup_id = Uuid::new_v4().to_string();
        let backup_dir = Path::new(&self.config.paths.backup_dir).join(&backup_id);
        tokio::fs::create_dir_all(&backup_dir).await?;
        Self::copy_directory(world_dir, &backup_dir).await?;
        
        // Remove current world
        if world_dir.exists() {
            tokio::fs::remove_dir_all(world_dir).await?;
        }
        
        // Restore from snapshot
        Self::copy_directory(&snapshot_info.path, world_dir).await?;
        
        info!("Successfully restored from snapshot: {}", snapshot_id);
        Ok(())
    }
    
    /// List all available snapshots
    pub async fn list_snapshots(&self) -> Vec<SnapshotInfo> {
        let snapshots_guard = self.snapshots.read().await;
        snapshots_guard.clone()
    }
    
    /// Delete a snapshot
    pub async fn delete_snapshot(&self, snapshot_id: &str) -> Result<()> {
        info!("Deleting snapshot: {}", snapshot_id);
        
        let snapshot_info = {
            let mut snapshots_guard = self.snapshots.write().await;
            let index = snapshots_guard.iter()
                .position(|s| s.id == snapshot_id)
                .ok_or_else(|| anyhow::anyhow!("Snapshot not found: {}", snapshot_id))?;
            
            snapshots_guard.remove(index)
        };
        
        // Remove snapshot directory
        if snapshot_info.path.exists() {
            tokio::fs::remove_dir_all(&snapshot_info.path).await?;
        }
        
        info!("Snapshot deleted: {}", snapshot_id);
        Ok(())
    }
    
    /// Load existing snapshots from backup directory
    async fn load_snapshots(backup_dir: &Path) -> Result<Vec<SnapshotInfo>> {
        let mut snapshots = Vec::new();
        
        if !backup_dir.exists() {
            return Ok(snapshots);
        }
        
        let mut entries = tokio::fs::read_dir(backup_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                // Try to parse as UUID
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if Uuid::parse_str(dir_name).is_ok() {
                        let metadata = entry.metadata().await?;
                        let size_bytes = Self::calculate_directory_size(&path).await?;
                        
                        snapshots.push(SnapshotInfo {
                            id: dir_name.to_string(),
                            timestamp: metadata.created()?.into(),
                            path,
                            size_bytes,
                            description: "loaded".to_string(),
                        });
                    }
                }
            }
        }
        
        // Sort by timestamp (newest first)
        snapshots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        Ok(snapshots)
    }
    
    /// Clean up old snapshots
    async fn cleanup_old_snapshots(snapshots: &mut Vec<SnapshotInfo>, keep_count: u32) {
        if snapshots.len() <= keep_count as usize {
            return;
        }
        
        // Sort by timestamp (newest first)
        snapshots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        // Remove old snapshots
        let to_remove = snapshots.split_off(keep_count as usize);
        for snapshot in to_remove {
            if let Err(e) = tokio::fs::remove_dir_all(&snapshot.path).await {
                error!("Failed to remove old snapshot {}: {}", snapshot.id, e);
            } else {
                info!("Removed old snapshot: {}", snapshot.id);
            }
        }
    }
    
    /// Copy directory recursively
    async fn copy_directory(src: &Path, dst: &Path) -> Result<()> {
        Box::pin(Self::copy_directory_recursive(src, dst)).await
    }
    
    async fn copy_directory_recursive(src: &Path, dst: &Path) -> Result<()> {
        tokio::fs::create_dir_all(dst).await?;
        
        let mut entries = tokio::fs::read_dir(src).await?;
        while let Some(entry) = entries.next_entry().await? {
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            
            if src_path.is_dir() {
                Box::pin(Self::copy_directory_recursive(&src_path, &dst_path)).await?;
            } else {
                tokio::fs::copy(&src_path, &dst_path).await?;
            }
        }
        
        Ok(())
    }
    
    /// Calculate directory size
    async fn calculate_directory_size(path: &Path) -> Result<u64> {
        Box::pin(Self::calculate_directory_size_recursive(path)).await
    }
    
    async fn calculate_directory_size_recursive(path: &Path) -> Result<u64> {
        let mut total_size = 0u64;
        
        let mut entries = tokio::fs::read_dir(path).await?;
        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            
            if entry_path.is_dir() {
                total_size += Box::pin(Self::calculate_directory_size_recursive(&entry_path)).await?;
            } else {
                let metadata = entry.metadata().await?;
                total_size += metadata.len();
            }
        }
        
        Ok(total_size)
    }
    
    /// Get snapshot manager statistics
    pub async fn get_stats(&self) -> serde_json::Map<String, serde_json::Value> {
        let mut stats = serde_json::Map::new();
        
        let snapshots_guard = self.snapshots.read().await;
        stats.insert("total_snapshots".to_string(), 
                    serde_json::Value::Number(snapshots_guard.len().into()));
        
        let total_size: u64 = snapshots_guard.iter().map(|s| s.size_bytes).sum();
        stats.insert("total_size_bytes".to_string(), 
                    serde_json::Value::Number(total_size.into()));
        
        if let Some(last_snapshot) = snapshots_guard.first() {
            stats.insert("last_snapshot_id".to_string(), 
                        serde_json::Value::String(last_snapshot.id.clone()));
            stats.insert("last_snapshot_time".to_string(), 
                        serde_json::Value::String(last_snapshot.timestamp.to_string()));
        }
        
        stats
    }
}
