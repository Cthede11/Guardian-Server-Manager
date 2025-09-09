use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::database::{DatabaseManager, Task};
use crate::websocket::WebSocketManager;
use crate::rcon::RconClient;

/// Hot import job for importing pre-generated chunks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotImportJob {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub status: ImportStatus,
    pub progress: f64,
    pub config: HotImportConfig,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub stats: ImportStats,
}

/// Import status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImportStatus {
    Pending,
    Scanning,
    Importing,
    Completed,
    Failed,
    Cancelled,
}

/// Hot import configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotImportConfig {
    pub source_dir: PathBuf,
    pub target_world: String,
    pub dimensions: Vec<String>, // overworld, nether, end
    pub chunk_batch_size: u32,
    pub tps_threshold: f64, // Pause if TPS drops below this
    pub safety_checks: bool,
    pub backup_before_import: bool,
}

/// Import statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportStats {
    pub chunks_scanned: u64,
    pub chunks_safe_to_import: u64,
    pub chunks_imported: u64,
    pub chunks_skipped: u64,
    pub chunks_failed: u64,
    pub import_rate: f64, // chunks per second
    pub current_region: Option<String>,
    pub errors: Vec<String>,
}

/// Chunk information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInfo {
    pub x: i32,
    pub z: i32,
    pub dimension: String,
    pub file_path: PathBuf,
    pub size_bytes: u64,
    pub last_modified: DateTime<Utc>,
    pub is_loaded: bool,
    pub is_safe_to_import: bool,
}

/// Hot import manager
pub struct HotImportManager {
    jobs: Arc<RwLock<HashMap<String, HotImportJob>>>,
    db: DatabaseManager,
    websocket_manager: Option<Arc<WebSocketManager>>,
    rcon_clients: Arc<RwLock<HashMap<String, RconClient>>>,
}

impl HotImportManager {
    pub fn new(db: DatabaseManager) -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            db,
            websocket_manager: None,
            rcon_clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set the WebSocket manager for real-time updates
    pub fn set_websocket_manager(&mut self, websocket_manager: Arc<WebSocketManager>) {
        self.websocket_manager = Some(websocket_manager);
    }

    /// Create a new hot import job
    pub async fn create_job(&self, server_id: &str, config: HotImportConfig) -> Result<String> {
        let job_id = Uuid::new_v4().to_string();
        
        let job = HotImportJob {
            id: job_id.clone(),
            server_id: server_id.to_string(),
            name: format!("Hot Import Job {}", job_id),
            status: ImportStatus::Pending,
            progress: 0.0,
            config,
            created_at: Utc::now(),
            started_at: None,
            finished_at: None,
            error: None,
            stats: ImportStats {
                chunks_scanned: 0,
                chunks_safe_to_import: 0,
                chunks_imported: 0,
                chunks_skipped: 0,
                chunks_failed: 0,
                import_rate: 0.0,
                current_region: None,
                errors: Vec::new(),
            },
        };

        // Store in database
        let task = Task {
            id: job_id.clone(),
            server_id: Some(server_id.to_string()),
            kind: "import".to_string(),
            status: "pending".to_string(),
            progress: 0.0,
            log: None,
            metadata: Some(serde_json::to_value(&job.config)?),
            started_at: None,
            finished_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.db.create_task(&task).await?;

        // Store in memory
        {
            let mut jobs = self.jobs.write().await;
            jobs.insert(job_id.clone(), job);
        }

        info!("Created hot import job: {} for server: {}", job_id, server_id);
        Ok(job_id)
    }

    /// Start a hot import job
    pub async fn start_job(&self, job_id: &str) -> Result<()> {
        let mut job = {
            let mut jobs = self.jobs.write().await;
            jobs.get_mut(job_id)
                .ok_or_else(|| anyhow!("Job not found: {}", job_id))?
                .clone()
        };

        if job.status != ImportStatus::Pending {
            return Err(anyhow!("Job cannot be started in status: {:?}", job.status));
        }

        job.status = ImportStatus::Scanning;
        job.started_at = Some(Utc::now());
        job.progress = 0.0;

        // Update database
        self.db.update_task(&Task {
            id: job_id.to_string(),
            server_id: job.server_id.clone().into(),
            kind: "import".to_string(),
            status: "running".to_string(),
            progress: 0.0,
            log: Some("Starting hot import job".to_string()),
            metadata: Some(serde_json::to_value(&job.config)?),
            started_at: job.started_at,
            finished_at: None,
            created_at: job.created_at,
            updated_at: Utc::now(),
        }).await?;

        // Update in memory
        {
            let mut jobs = self.jobs.write().await;
            jobs.insert(job_id.to_string(), job);
        }

        // Start the actual import process
        self.start_import_process(job_id).await?;

        info!("Started hot import job: {}", job_id);
        Ok(())
    }

    /// Cancel a hot import job
    pub async fn cancel_job(&self, job_id: &str) -> Result<()> {
        let mut job = {
            let mut jobs = self.jobs.write().await;
            jobs.get_mut(job_id)
                .ok_or_else(|| anyhow!("Job not found: {}", job_id))?
                .clone()
        };

        if job.status == ImportStatus::Completed || job.status == ImportStatus::Failed {
            return Err(anyhow!("Job cannot be cancelled in status: {:?}", job.status));
        }

        job.status = ImportStatus::Cancelled;
        job.finished_at = Some(Utc::now());

        // Update database
        self.db.update_task(&Task {
            id: job_id.to_string(),
            server_id: job.server_id.clone().into(),
            kind: "import".to_string(),
            status: "cancelled".to_string(),
            progress: job.progress,
            log: Some("Job cancelled by user".to_string()),
            metadata: Some(serde_json::to_value(&job.config)?),
            started_at: job.started_at,
            finished_at: job.finished_at,
            created_at: job.created_at,
            updated_at: Utc::now(),
        }).await?;

        // Update in memory
        {
            let mut jobs = self.jobs.write().await;
            jobs.insert(job_id.to_string(), job);
        }

        info!("Cancelled hot import job: {}", job_id);
        Ok(())
    }

    /// Get a hot import job
    pub async fn get_job(&self, job_id: &str) -> Option<HotImportJob> {
        let jobs = self.jobs.read().await;
        jobs.get(job_id).cloned()
    }

    /// Get all jobs for a server
    pub async fn get_server_jobs(&self, server_id: &str) -> Vec<HotImportJob> {
        let jobs = self.jobs.read().await;
        jobs.values()
            .filter(|job| job.server_id == server_id)
            .cloned()
            .collect()
    }

    /// Get all jobs
    pub async fn get_all_jobs(&self) -> Vec<HotImportJob> {
        let jobs = self.jobs.read().await;
        jobs.values().cloned().collect()
    }

    /// Start the actual import process
    async fn start_import_process(&self, job_id: &str) -> Result<()> {
        let job = {
            let jobs = self.jobs.read().await;
            jobs.get(job_id).cloned()
                .ok_or_else(|| anyhow!("Job not found: {}", job_id))?
        };

        // Spawn import task
        let jobs = self.jobs.clone();
        let db = self.db.clone();
        let websocket_manager = self.websocket_manager.clone();
        let rcon_clients = self.rcon_clients.clone();

        let job_id_clone = job_id.to_string();
        let jobs_clone = jobs.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::run_import_process(
                &job_id_clone,
                &job,
                jobs,
                db,
                websocket_manager,
                rcon_clients,
            ).await {
                error!("Import process failed for job {}: {}", job_id_clone, e);
                
                // Update job status to failed
                if let Some(mut job) = jobs_clone.write().await.get_mut(&job_id_clone) {
                    job.status = ImportStatus::Failed;
                    job.error = Some(e.to_string());
                    job.finished_at = Some(Utc::now());
                }
            }
        });

        Ok(())
    }

    /// Run the import process
    async fn run_import_process(
        job_id: &str,
        job: &HotImportJob,
        jobs: Arc<RwLock<HashMap<String, HotImportJob>>>,
        db: DatabaseManager,
        websocket_manager: Option<Arc<WebSocketManager>>,
        rcon_clients: Arc<RwLock<HashMap<String, RconClient>>>,
    ) -> Result<()> {
        info!("Starting import process for job: {}", job_id);

        let config = &job.config;
        let mut stats = job.stats.clone();

        // Phase 1: Scan for chunks to import
        info!("Phase 1: Scanning for chunks to import");
        let chunks_to_import = Self::scan_chunks_to_import(&config).await?;
        stats.chunks_scanned = chunks_to_import.len() as u64;

        // Update progress
        {
            let mut jobs = jobs.write().await;
            if let Some(job) = jobs.get_mut(job_id) {
                job.status = ImportStatus::Importing;
                job.stats = stats.clone();
                job.progress = 0.1; // 10% for scanning
            }
        }

        // Phase 2: Safety checks
        info!("Phase 2: Performing safety checks");
        let safe_chunks = Self::perform_safety_checks(
            &chunks_to_import,
            &job.server_id,
            &rcon_clients,
        ).await?;
        stats.chunks_safe_to_import = safe_chunks.len() as u64;

        // Update progress
        {
            let mut jobs = jobs.write().await;
            if let Some(job) = jobs.get_mut(job_id) {
                job.stats = stats.clone();
                job.progress = 0.2; // 20% for safety checks
            }
        }

        // Phase 3: Import chunks
        info!("Phase 3: Importing chunks");
        let start_time = std::time::Instant::now();
        let mut imported_count = 0u64;
        let mut skipped_count = 0u64;
        let mut failed_count = 0u64;

        for (i, chunk) in safe_chunks.iter().enumerate() {
            // Check if job was cancelled
            {
                let jobs = jobs.read().await;
                if let Some(job) = jobs.get(job_id) {
                    if job.status == ImportStatus::Cancelled {
                        info!("Job {} was cancelled, stopping import", job_id);
                        return Ok(());
                    }
                }
            }

            // Check TPS threshold
            if config.tps_threshold > 0.0 {
                if let Some(tps) = Self::get_server_tps(&job.server_id, &rcon_clients).await? {
                    if tps < config.tps_threshold {
                        warn!("TPS too low ({}), pausing import", tps);
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        continue;
                    }
                }
            }

            // Import chunk
            match Self::import_chunk(chunk, &config).await {
                Ok(_) => {
                    imported_count += 1;
                    stats.chunks_imported = imported_count;
                }
                Err(e) => {
                    warn!("Failed to import chunk ({}, {}): {}", chunk.x, chunk.z, e);
                    failed_count += 1;
                    stats.chunks_failed = failed_count;
                    stats.errors.push(format!("Chunk ({}, {}): {}", chunk.x, chunk.z, e));
                }
            }

            // Update progress
            let progress = 0.2 + 0.7 * (i as f64 / safe_chunks.len() as f64);
            {
                let mut jobs = jobs.write().await;
                if let Some(job) = jobs.get_mut(job_id) {
                    job.progress = progress;
                    job.stats = stats.clone();
                    job.stats.current_region = Some(format!("({}, {})", chunk.x, chunk.z));
                }
            }

            // Update database
            if i % 100 == 0 { // Update every 100 chunks
                if let Err(e) = db.update_task(&Task {
                    id: job_id.to_string(),
                    server_id: job.server_id.clone().into(),
                    kind: "import".to_string(),
                    status: "running".to_string(),
                    progress,
                    log: Some(format!("Imported {}/{} chunks", i + 1, safe_chunks.len())),
                    metadata: Some(serde_json::to_value(&stats)?),
                    started_at: job.started_at,
                    finished_at: None,
                    created_at: job.created_at,
                    updated_at: Utc::now(),
                }).await {
                    error!("Failed to update task progress: {}", e);
                }
            }

            // Send WebSocket update
            if let Some(ws_manager) = &websocket_manager {
                let rate = imported_count as f64 / start_time.elapsed().as_secs_f64();
                stats.import_rate = rate;

                let task = Task {
                    id: job_id.to_string(),
                    server_id: job.server_id.clone().into(),
                    kind: "import".to_string(),
                    status: "running".to_string(),
                    progress,
                    log: Some(format!("Imported chunk ({}, {})", chunk.x, chunk.z)),
                    metadata: Some(serde_json::to_value(&stats)?),
                    started_at: job.started_at,
                    finished_at: None,
                    created_at: job.created_at,
                    updated_at: Utc::now(),
                };

                ws_manager.send_task_update(Some(&job.server_id), task).await;
            }

            // Batch processing
            if (i + 1) % config.chunk_batch_size as usize == 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        }

        // Mark job as completed
        {
            let mut jobs = jobs.write().await;
            if let Some(job) = jobs.get_mut(job_id) {
                job.status = ImportStatus::Completed;
                job.progress = 1.0;
                job.finished_at = Some(Utc::now());
                job.stats = stats.clone();
            }
        }

        // Update database
        db.update_task(&Task {
            id: job_id.to_string(),
            server_id: job.server_id.clone().into(),
            kind: "import".to_string(),
            status: "completed".to_string(),
            progress: 1.0,
            log: Some(format!("Import completed: {} chunks imported, {} failed", imported_count, failed_count)),
            metadata: Some(serde_json::to_value(&stats)?),
            started_at: job.started_at,
            finished_at: Some(Utc::now()),
            created_at: job.created_at,
            updated_at: Utc::now(),
        }).await?;

        info!("Completed import process for job: {}", job_id);
        Ok(())
    }

    /// Scan for chunks to import
    async fn scan_chunks_to_import(config: &HotImportConfig) -> Result<Vec<ChunkInfo>> {
        let mut chunks = Vec::new();

        for dimension in &config.dimensions {
            let dim_dir = config.source_dir.join(dimension);
            if !dim_dir.exists() {
                continue;
            }

            // Scan region files
            for entry in std::fs::read_dir(&dim_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.extension().and_then(|s| s.to_str()) == Some("mca") {
                    // Parse region file to find chunks
                    if let Ok(region_chunks) = Self::parse_region_file(&path, dimension).await {
                        chunks.extend(region_chunks);
                    }
                }
            }
        }

        Ok(chunks)
    }

    /// Parse a region file to extract chunk information
    async fn parse_region_file(region_path: &Path, dimension: &str) -> Result<Vec<ChunkInfo>> {
        let mut chunks = Vec::new();
        
        // This is a simplified implementation
        // In a real implementation, you would:
        // 1. Parse the .mca file format
        // 2. Extract chunk coordinates and metadata
        // 3. Check if chunks are valid
        
        // For now, return empty list
        Ok(chunks)
    }

    /// Perform safety checks on chunks
    async fn perform_safety_checks(
        chunks: &[ChunkInfo],
        server_id: &str,
        rcon_clients: &Arc<RwLock<HashMap<String, RconClient>>>,
    ) -> Result<Vec<ChunkInfo>> {
        let mut safe_chunks = Vec::new();

        // Get loaded chunks from server
        let loaded_chunks = Self::get_loaded_chunks(server_id, rcon_clients).await?;

        for chunk in chunks {
            // Check if chunk is loaded
            if loaded_chunks.contains(&(chunk.x, chunk.z)) {
                warn!("Skipping loaded chunk ({}, {})", chunk.x, chunk.z);
                continue;
            }

            // Check if chunk already exists in target world
            if Self::chunk_exists_in_target(&chunk).await? {
                warn!("Skipping existing chunk ({}, {})", chunk.x, chunk.z);
                continue;
            }

            safe_chunks.push(chunk.clone());
        }

        Ok(safe_chunks)
    }

    /// Get loaded chunks from server via RCON
    async fn get_loaded_chunks(
        server_id: &str,
        rcon_clients: &Arc<RwLock<HashMap<String, RconClient>>>,
    ) -> Result<HashSet<(i32, i32)>> {
        let mut loaded_chunks = HashSet::new();

        // This would use RCON to query loaded chunks
        // For now, return empty set
        Ok(loaded_chunks)
    }

    /// Check if chunk exists in target world
    async fn chunk_exists_in_target(chunk: &ChunkInfo) -> Result<bool> {
        // This would check if the chunk already exists in the target world
        // For now, return false
        Ok(false)
    }

    /// Get server TPS via RCON
    async fn get_server_tps(
        server_id: &str,
        rcon_clients: &Arc<RwLock<HashMap<String, RconClient>>>,
    ) -> Result<Option<f64>> {
        // This would use RCON to get server TPS
        // For now, return None
        Ok(None)
    }

    /// Import a single chunk
    async fn import_chunk(chunk: &ChunkInfo, config: &HotImportConfig) -> Result<()> {
        // This would perform the actual chunk import
        // 1. Copy chunk data from source to target
        // 2. Update region files atomically
        // 3. Verify import success
        
        debug!("Importing chunk ({}, {}) from {:?}", chunk.x, chunk.z, chunk.file_path);
        
        // Simulate import time
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        
        Ok(())
    }
}
