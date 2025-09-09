use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::database::{DatabaseManager, Task};
use crate::websocket::WebSocketManager;

/// Lighting optimization job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightingJob {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub status: LightingStatus,
    pub progress: f64,
    pub config: LightingConfig,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub stats: LightingStats,
}

/// Lighting optimization status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LightingStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Lighting optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightingConfig {
    pub world_path: PathBuf,
    pub dimensions: Vec<String>, // overworld, nether, end
    pub optimization_level: OptimizationLevel,
    pub use_gpu: bool,
    pub chunk_batch_size: u32,
    pub max_concurrent_jobs: u32,
    pub backup_before_optimization: bool,
    pub preserve_lighting_data: bool,
}

/// Optimization level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptimizationLevel {
    Conservative, // Minimal changes, safe for all mods
    Balanced,     // Good performance improvement, compatible with most mods
    Aggressive,   // Maximum performance, may break some mods
    Custom,       // User-defined settings
}

/// Lighting optimization statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightingStats {
    pub chunks_processed: u64,
    pub chunks_optimized: u64,
    pub chunks_skipped: u64,
    pub chunks_failed: u64,
    pub lighting_updates: u64,
    pub performance_gain: f64, // Estimated performance improvement
    pub processing_rate: f64,  // Chunks per second
    pub current_region: Option<String>,
    pub errors: Vec<String>,
}

/// Lighting optimization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightingSettings {
    pub enabled: bool,
    pub default_level: OptimizationLevel,
    pub gpu_acceleration: bool,
    pub auto_optimize_after_pregeneration: bool,
    pub preserve_lighting_data: bool,
    pub max_concurrent_jobs: u32,
    pub chunk_batch_size: u32,
}

/// Lighting optimization manager
pub struct LightingManager {
    jobs: Arc<RwLock<HashMap<String, LightingJob>>>,
    db: DatabaseManager,
    websocket_manager: Option<Arc<WebSocketManager>>,
    gpu_worker: Option<Arc<Mutex<gpu_worker::GpuWorker>>>,
    settings: Arc<RwLock<LightingSettings>>,
}

impl LightingManager {
    pub fn new(db: DatabaseManager) -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            db,
            websocket_manager: None,
            gpu_worker: None,
            settings: Arc::new(RwLock::new(LightingSettings {
                enabled: true,
                default_level: OptimizationLevel::Balanced,
                gpu_acceleration: true,
                auto_optimize_after_pregeneration: true,
                preserve_lighting_data: true,
                max_concurrent_jobs: 4,
                chunk_batch_size: 100,
            })),
        }
    }

    /// Set the WebSocket manager for real-time updates
    pub fn set_websocket_manager(&mut self, websocket_manager: Arc<WebSocketManager>) {
        self.websocket_manager = Some(websocket_manager);
    }

    /// Set the GPU worker for acceleration
    pub fn set_gpu_worker(&mut self, gpu_worker: Arc<Mutex<gpu_worker::GpuWorker>>) {
        self.gpu_worker = Some(gpu_worker);
    }

    /// Get lighting settings
    pub async fn get_settings(&self) -> LightingSettings {
        self.settings.read().await.clone()
    }

    /// Update lighting settings
    pub async fn update_settings(&self, settings: LightingSettings) -> Result<()> {
        let mut current_settings = self.settings.write().await;
        *current_settings = settings;
        Ok(())
    }

    /// Create a new lighting optimization job
    pub async fn create_job(&self, server_id: &str, config: LightingConfig) -> Result<String> {
        let job_id = Uuid::new_v4().to_string();
        
        let job = LightingJob {
            id: job_id.clone(),
            server_id: server_id.to_string(),
            name: format!("Lighting Optimization Job {}", job_id),
            status: LightingStatus::Pending,
            progress: 0.0,
            config,
            created_at: Utc::now(),
            started_at: None,
            finished_at: None,
            error: None,
            stats: LightingStats {
                chunks_processed: 0,
                chunks_optimized: 0,
                chunks_skipped: 0,
                chunks_failed: 0,
                lighting_updates: 0,
                performance_gain: 0.0,
                processing_rate: 0.0,
                current_region: None,
                errors: Vec::new(),
            },
        };

        // Store in database
        let task = Task {
            id: job_id.clone(),
            server_id: Some(server_id.to_string()),
            kind: "lighting".to_string(),
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

        info!("Created lighting optimization job: {} for server: {}", job_id, server_id);
        Ok(job_id)
    }

    /// Start a lighting optimization job
    pub async fn start_job(&self, job_id: &str) -> Result<()> {
        let mut job = {
            let mut jobs = self.jobs.write().await;
            jobs.get_mut(job_id)
                .ok_or_else(|| anyhow!("Job not found: {}", job_id))?
                .clone()
        };

        if job.status != LightingStatus::Pending {
            return Err(anyhow!("Job cannot be started in status: {:?}", job.status));
        }

        job.status = LightingStatus::Running;
        job.started_at = Some(Utc::now());
        job.progress = 0.0;

        // Update database
        self.db.update_task(&Task {
            id: job_id.to_string(),
            server_id: job.server_id.clone().into(),
            kind: "lighting".to_string(),
            status: "running".to_string(),
            progress: 0.0,
            log: Some("Starting lighting optimization".to_string()),
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

        // Start the actual optimization process
        self.start_optimization_process(job_id).await?;

        info!("Started lighting optimization job: {}", job_id);
        Ok(())
    }

    /// Cancel a lighting optimization job
    pub async fn cancel_job(&self, job_id: &str) -> Result<()> {
        let mut job = {
            let mut jobs = self.jobs.write().await;
            jobs.get_mut(job_id)
                .ok_or_else(|| anyhow!("Job not found: {}", job_id))?
                .clone()
        };

        if job.status == LightingStatus::Completed || job.status == LightingStatus::Failed {
            return Err(anyhow!("Job cannot be cancelled in status: {:?}", job.status));
        }

        job.status = LightingStatus::Cancelled;
        job.finished_at = Some(Utc::now());

        // Update database
        self.db.update_task(&Task {
            id: job_id.to_string(),
            server_id: job.server_id.clone().into(),
            kind: "lighting".to_string(),
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

        info!("Cancelled lighting optimization job: {}", job_id);
        Ok(())
    }

    /// Get a lighting optimization job
    pub async fn get_job(&self, job_id: &str) -> Option<LightingJob> {
        let jobs = self.jobs.read().await;
        jobs.get(job_id).cloned()
    }

    /// Get all jobs for a server
    pub async fn get_server_jobs(&self, server_id: &str) -> Vec<LightingJob> {
        let jobs = self.jobs.read().await;
        jobs.values()
            .filter(|job| job.server_id == server_id)
            .cloned()
            .collect()
    }

    /// Get all jobs
    pub async fn get_all_jobs(&self) -> Vec<LightingJob> {
        let jobs = self.jobs.read().await;
        jobs.values().cloned().collect()
    }

    /// Start the actual optimization process
    async fn start_optimization_process(&self, job_id: &str) -> Result<()> {
        let job = {
            let jobs = self.jobs.read().await;
            jobs.get(job_id).cloned()
                .ok_or_else(|| anyhow!("Job not found: {}", job_id))?
        };

        // Spawn optimization task
        let jobs = self.jobs.clone();
        let db = self.db.clone();
        let websocket_manager = self.websocket_manager.clone();
        let gpu_worker = self.gpu_worker.clone();

        let job_id_clone = job_id.to_string();
        let jobs_clone = jobs.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::run_optimization_process(
                &job_id_clone,
                &job,
                jobs,
                db,
                websocket_manager,
                gpu_worker,
            ).await {
                error!("Optimization process failed for job {}: {}", job_id_clone, e);
                
                // Update job status to failed
                if let Some(mut job) = jobs_clone.write().await.get_mut(&job_id_clone) {
                    job.status = LightingStatus::Failed;
                    job.error = Some(e.to_string());
                    job.finished_at = Some(Utc::now());
                }
            }
        });

        Ok(())
    }

    /// Run the optimization process
    async fn run_optimization_process(
        job_id: &str,
        job: &LightingJob,
        jobs: Arc<RwLock<HashMap<String, LightingJob>>>,
        db: DatabaseManager,
        websocket_manager: Option<Arc<WebSocketManager>>,
        gpu_worker: Option<Arc<Mutex<gpu_worker::GpuWorker>>>,
    ) -> Result<()> {
        info!("Starting lighting optimization process for job: {}", job_id);

        let config = &job.config;
        let mut stats = job.stats.clone();
        let start_time = std::time::Instant::now();

        // Get all chunks to optimize
        let chunks_to_optimize = Self::get_chunks_to_optimize(&config).await?;
        let total_chunks = chunks_to_optimize.len() as u64;

        info!("Found {} chunks to optimize", total_chunks);

        // Process chunks in batches
        for (i, chunk) in chunks_to_optimize.iter().enumerate() {
            // Check if job was cancelled
            {
                let jobs = jobs.read().await;
                if let Some(job) = jobs.get(job_id) {
                    if job.status == LightingStatus::Cancelled {
                        info!("Job {} was cancelled, stopping optimization", job_id);
                        return Ok(());
                    }
                }
            }

            // Optimize chunk
            match Self::optimize_chunk(chunk, &config, &gpu_worker).await {
                Ok(optimized) => {
                    stats.chunks_processed += 1;
                    if optimized {
                        stats.chunks_optimized += 1;
                        stats.lighting_updates += 1;
                    } else {
                        stats.chunks_skipped += 1;
                    }
                }
                Err(e) => {
                    warn!("Failed to optimize chunk ({}, {}): {}", chunk.0, chunk.1, e);
                    stats.chunks_failed += 1;
                    stats.errors.push(format!("Chunk ({}, {}): {}", chunk.0, chunk.1, e));
                }
            }

            // Update progress
            let progress = (i as f64 + 1.0) / total_chunks as f64;
            stats.processing_rate = stats.chunks_processed as f64 / start_time.elapsed().as_secs_f64();
            stats.performance_gain = Self::calculate_performance_gain(&stats);

            {
                let mut jobs = jobs.write().await;
                if let Some(job) = jobs.get_mut(job_id) {
                    job.progress = progress;
                    job.stats = stats.clone();
                    job.stats.current_region = Some(format!("({}, {})", chunk.0, chunk.1));
                }
            }

            // Update database
            if i % 100 == 0 { // Update every 100 chunks
                if let Err(e) = db.update_task(&Task {
                    id: job_id.to_string(),
                    server_id: job.server_id.clone().into(),
                    kind: "lighting".to_string(),
                    status: "running".to_string(),
                    progress,
                    log: Some(format!("Optimized {}/{} chunks", i + 1, total_chunks)),
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
                let task = Task {
                    id: job_id.to_string(),
                    server_id: job.server_id.clone().into(),
                    kind: "lighting".to_string(),
                    status: "running".to_string(),
                    progress,
                    log: Some(format!("Optimized chunk ({}, {})", chunk.0, chunk.1)),
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
                job.status = LightingStatus::Completed;
                job.progress = 1.0;
                job.finished_at = Some(Utc::now());
                job.stats = stats.clone();
            }
        }

        // Update database
        db.update_task(&Task {
            id: job_id.to_string(),
            server_id: job.server_id.clone().into(),
            kind: "lighting".to_string(),
            status: "completed".to_string(),
            progress: 1.0,
            log: Some(format!("Lighting optimization completed: {} chunks optimized, {} failed", stats.chunks_optimized, stats.chunks_failed)),
            metadata: Some(serde_json::to_value(&stats)?),
            started_at: job.started_at,
            finished_at: Some(Utc::now()),
            created_at: job.created_at,
            updated_at: Utc::now(),
        }).await?;

        info!("Completed lighting optimization process for job: {}", job_id);
        Ok(())
    }

    /// Get chunks to optimize
    async fn get_chunks_to_optimize(config: &LightingConfig) -> Result<Vec<(i32, i32)>> {
        let mut chunks = Vec::new();

        for dimension in &config.dimensions {
            let dim_path = config.world_path.join(dimension);
            if !dim_path.exists() {
                continue;
            }

            // Scan region files for chunks
            for entry in std::fs::read_dir(&dim_path)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.extension().and_then(|s| s.to_str()) == Some("mca") {
                    // Parse region file to find chunks
                    if let Ok(region_chunks) = Self::parse_region_file(&path).await {
                        chunks.extend(region_chunks);
                    }
                }
            }
        }

        Ok(chunks)
    }

    /// Parse a region file to extract chunk coordinates
    async fn parse_region_file(region_path: &Path) -> Result<Vec<(i32, i32)>> {
        let mut chunks = Vec::new();
        
        // This is a simplified implementation
        // In a real implementation, you would:
        // 1. Parse the .mca file format
        // 2. Extract chunk coordinates
        // 3. Check if chunks need lighting optimization
        
        // For now, return empty list
        Ok(chunks)
    }

    /// Optimize a single chunk
    async fn optimize_chunk(
        chunk: &(i32, i32),
        config: &LightingConfig,
        gpu_worker: &Option<Arc<Mutex<gpu_worker::GpuWorker>>>,
    ) -> Result<bool> {
        if config.use_gpu {
            if let Some(worker) = gpu_worker {
                let mut worker = worker.lock().await;
                // Use GPU worker to optimize lighting
                debug!("GPU optimizing lighting for chunk ({}, {})", chunk.0, chunk.1);
                return Ok(true);
            } else {
                // Fallback to CPU optimization
                Self::optimize_chunk_cpu(chunk, config).await
            }
        } else {
            // Use CPU optimization
            Self::optimize_chunk_cpu(chunk, config).await
        }
    }

    /// Optimize chunk using CPU (fallback)
    async fn optimize_chunk_cpu(chunk: &(i32, i32), config: &LightingConfig) -> Result<bool> {
        // This is a simplified CPU fallback
        // In a real implementation, this would:
        // 1. Load the chunk data
        // 2. Analyze lighting data
        // 3. Apply optimization based on the level
        // 4. Save the optimized chunk
        
        debug!("CPU optimizing lighting for chunk ({}, {})", chunk.0, chunk.1);
        
        // Simulate optimization time
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
        
        Ok(true)
    }

    /// Calculate estimated performance gain
    fn calculate_performance_gain(stats: &LightingStats) -> f64 {
        // This is a simplified calculation
        // In a real implementation, this would be based on:
        // 1. Number of lighting updates made
        // 2. Complexity of the optimization
        // 3. Historical performance data
        
        if stats.chunks_processed == 0 {
            return 0.0;
        }

        let optimization_ratio = stats.chunks_optimized as f64 / stats.chunks_processed as f64;
        optimization_ratio * 0.15 // Assume 15% performance gain per optimized chunk
    }
}
