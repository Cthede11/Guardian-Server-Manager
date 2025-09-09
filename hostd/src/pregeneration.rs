use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::database::{DatabaseManager, Task};
use crate::websocket::WebSocketManager;

/// Pre-generation job configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PregenerationJob {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub status: JobStatus,
    pub progress: f64,
    pub config: PregenerationConfig,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

/// Job status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// Pre-generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PregenerationConfig {
    pub center_x: i32,
    pub center_z: i32,
    pub radius: u32,
    pub dimensions: Vec<String>, // overworld, nether, end
    pub gpu_acceleration: bool,
    pub efficiency_package: bool,
    pub chunk_batch_size: u32,
    pub max_concurrent_jobs: u32,
}

/// Pre-generation progress update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub job_id: String,
    pub progress: f64,
    pub chunks_generated: u64,
    pub chunks_total: u64,
    pub rate: f64, // chunks per second
    pub eta_seconds: u64,
    pub current_region: Option<String>,
    pub errors: Vec<String>,
}

/// Pre-generation manager
pub struct PregenerationManager {
    jobs: Arc<RwLock<HashMap<String, PregenerationJob>>>,
    db: DatabaseManager,
    websocket_manager: Option<Arc<WebSocketManager>>,
    gpu_worker: Option<Arc<Mutex<gpu_worker::GpuWorker>>>,
}

impl PregenerationManager {
    pub fn new(db: DatabaseManager) -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            db,
            websocket_manager: None,
            gpu_worker: None,
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

    /// Create a new pre-generation job
    pub async fn create_job(&self, server_id: &str, config: PregenerationConfig) -> Result<String> {
        let job_id = Uuid::new_v4().to_string();
        
        let job = PregenerationJob {
            id: job_id.clone(),
            server_id: server_id.to_string(),
            name: format!("Pregen Job {}", job_id),
            status: JobStatus::Pending,
            progress: 0.0,
            config,
            created_at: Utc::now(),
            started_at: None,
            finished_at: None,
            error: None,
        };

        // Store in database
        let task = Task {
            id: job_id.clone(),
            server_id: Some(server_id.to_string()),
            kind: "worldgen".to_string(),
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

        info!("Created pre-generation job: {} for server: {}", job_id, server_id);
        Ok(job_id)
    }

    /// Start a pre-generation job
    pub async fn start_job(&self, job_id: &str) -> Result<()> {
        let mut job = {
            let mut jobs = self.jobs.write().await;
            jobs.get_mut(job_id)
                .ok_or_else(|| anyhow!("Job not found: {}", job_id))?
                .clone()
        };

        if job.status != JobStatus::Pending && job.status != JobStatus::Paused {
            return Err(anyhow!("Job cannot be started in status: {:?}", job.status));
        }

        job.status = JobStatus::Running;
        job.started_at = Some(Utc::now());
        job.progress = 0.0;

        // Update database
        self.db.update_task(&Task {
            id: job_id.to_string(),
            server_id: job.server_id.clone().into(),
            kind: "worldgen".to_string(),
            status: "running".to_string(),
            progress: 0.0,
            log: None,
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

        // Start the actual generation process
        self.start_generation_process(job_id).await?;

        info!("Started pre-generation job: {}", job_id);
        Ok(())
    }

    /// Pause a pre-generation job
    pub async fn pause_job(&self, job_id: &str) -> Result<()> {
        let mut job = {
            let mut jobs = self.jobs.write().await;
            jobs.get_mut(job_id)
                .ok_or_else(|| anyhow!("Job not found: {}", job_id))?
                .clone()
        };

        if job.status != JobStatus::Running {
            return Err(anyhow!("Job cannot be paused in status: {:?}", job.status));
        }

        job.status = JobStatus::Paused;

        // Update database
        self.db.update_task(&Task {
            id: job_id.to_string(),
            server_id: job.server_id.clone().into(),
            kind: "worldgen".to_string(),
            status: "paused".to_string(),
            progress: job.progress,
            log: None,
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

        info!("Paused pre-generation job: {}", job_id);
        Ok(())
    }

    /// Resume a paused pre-generation job
    pub async fn resume_job(&self, job_id: &str) -> Result<()> {
        let mut job = {
            let mut jobs = self.jobs.write().await;
            jobs.get_mut(job_id)
                .ok_or_else(|| anyhow!("Job not found: {}", job_id))?
                .clone()
        };

        if job.status != JobStatus::Paused {
            return Err(anyhow!("Job cannot be resumed in status: {:?}", job.status));
        }

        job.status = JobStatus::Running;

        // Update database
        self.db.update_task(&Task {
            id: job_id.to_string(),
            server_id: job.server_id.clone().into(),
            kind: "worldgen".to_string(),
            status: "running".to_string(),
            progress: job.progress,
            log: None,
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

        // Resume the generation process
        self.start_generation_process(job_id).await?;

        info!("Resumed pre-generation job: {}", job_id);
        Ok(())
    }

    /// Cancel a pre-generation job
    pub async fn cancel_job(&self, job_id: &str) -> Result<()> {
        let mut job = {
            let mut jobs = self.jobs.write().await;
            jobs.get_mut(job_id)
                .ok_or_else(|| anyhow!("Job not found: {}", job_id))?
                .clone()
        };

        if job.status == JobStatus::Completed || job.status == JobStatus::Failed {
            return Err(anyhow!("Job cannot be cancelled in status: {:?}", job.status));
        }

        job.status = JobStatus::Cancelled;
        job.finished_at = Some(Utc::now());

        // Update database
        self.db.update_task(&Task {
            id: job_id.to_string(),
            server_id: job.server_id.clone().into(),
            kind: "worldgen".to_string(),
            status: "cancelled".to_string(),
            progress: job.progress,
            log: None,
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

        info!("Cancelled pre-generation job: {}", job_id);
        Ok(())
    }

    /// Get a pre-generation job
    pub async fn get_job(&self, job_id: &str) -> Option<PregenerationJob> {
        let jobs = self.jobs.read().await;
        jobs.get(job_id).cloned()
    }

    /// Get all jobs for a server
    pub async fn get_server_jobs(&self, server_id: &str) -> Vec<PregenerationJob> {
        let jobs = self.jobs.read().await;
        jobs.values()
            .filter(|job| job.server_id == server_id)
            .cloned()
            .collect()
    }

    /// Get all jobs
    pub async fn get_all_jobs(&self) -> Vec<PregenerationJob> {
        let jobs = self.jobs.read().await;
        jobs.values().cloned().collect()
    }

    /// Start the actual generation process
    async fn start_generation_process(&self, job_id: &str) -> Result<()> {
        let job = {
            let jobs = self.jobs.read().await;
            jobs.get(job_id).cloned()
                .ok_or_else(|| anyhow!("Job not found: {}", job_id))?
        };

        // Spawn generation task
        let jobs = self.jobs.clone();
        let db = self.db.clone();
        let websocket_manager = self.websocket_manager.clone();
        let gpu_worker = self.gpu_worker.clone();

        let job_id_clone = job_id.to_string();
        let jobs_clone = jobs.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::run_generation_process(
                &job_id_clone,
                &job,
                jobs,
                db,
                websocket_manager,
                gpu_worker,
            ).await {
                error!("Generation process failed for job {}: {}", job_id_clone, e);
                
                // Update job status to failed
                if let Some(mut job) = jobs_clone.write().await.get_mut(&job_id_clone) {
                    job.status = JobStatus::Failed;
                    job.error = Some(e.to_string());
                    job.finished_at = Some(Utc::now());
                }
            }
        });

        Ok(())
    }

    /// Run the generation process
    async fn run_generation_process(
        job_id: &str,
        job: &PregenerationJob,
        jobs: Arc<RwLock<HashMap<String, PregenerationJob>>>,
        db: DatabaseManager,
        websocket_manager: Option<Arc<WebSocketManager>>,
        gpu_worker: Option<Arc<Mutex<gpu_worker::GpuWorker>>>,
    ) -> Result<()> {
        info!("Starting generation process for job: {}", job_id);

        let config = &job.config;
        let total_chunks = (config.radius * 2 + 1) as u64 * (config.radius * 2 + 1) as u64;
        let mut generated_chunks = 0u64;
        let start_time = std::time::Instant::now();

        // Generate chunks in a spiral pattern from center outward
        for ring in 0..=config.radius {
            for x in (config.center_x - ring as i32)..=(config.center_x + ring as i32) {
                for z in (config.center_z - ring as i32)..=(config.center_z + ring as i32) {
                    // Check if this is the outer ring
                    let is_outer_ring = x == config.center_x - ring as i32 || 
                                      x == config.center_x + ring as i32 ||
                                      z == config.center_z - ring as i32 || 
                                      z == config.center_z + ring as i32;

                    if ring == 0 || is_outer_ring {
                        // Generate chunk
                        if let Err(e) = Self::generate_chunk(
                            x, z, &config, &gpu_worker
                        ).await {
                            warn!("Failed to generate chunk ({}, {}): {}", x, z, e);
                        }

                        generated_chunks += 1;
                        let progress = generated_chunks as f64 / total_chunks as f64;

                        // Update progress
                        {
                            let mut jobs = jobs.write().await;
                            if let Some(job) = jobs.get_mut(job_id) {
                                job.progress = progress;
                            }
                        }

                        // Update database
                        if let Err(e) = db.update_task(&Task {
                            id: job_id.to_string(),
                            server_id: job.server_id.clone().into(),
                            kind: "worldgen".to_string(),
                            status: "running".to_string(),
                            progress,
                            log: None,
                            metadata: Some(serde_json::to_value(&config)?),
                            started_at: job.started_at,
                            finished_at: None,
                            created_at: job.created_at,
                            updated_at: Utc::now(),
                        }).await {
                            error!("Failed to update task progress: {}", e);
                        }

                        // Send WebSocket update
                        if let Some(ws_manager) = &websocket_manager {
                            let rate = generated_chunks as f64 / start_time.elapsed().as_secs_f64();
                            let eta_seconds = if rate > 0.0 {
                                ((total_chunks - generated_chunks) as f64 / rate) as u64
                            } else {
                                0
                            };

                            let progress_update = ProgressUpdate {
                                job_id: job_id.to_string(),
                                progress,
                                chunks_generated: generated_chunks,
                                chunks_total: total_chunks,
                                rate,
                                eta_seconds,
                                current_region: Some(format!("({}, {})", x, z)),
                                errors: Vec::new(),
                            };

                            // Convert to task update
                            let task = Task {
                                id: job_id.to_string(),
                                server_id: job.server_id.clone().into(),
                                kind: "worldgen".to_string(),
                                status: "running".to_string(),
                                progress,
                                log: Some(format!("Generated chunk ({}, {})", x, z)),
                                metadata: Some(serde_json::to_value(&progress_update)?),
                                started_at: job.started_at,
                                finished_at: None,
                                created_at: job.created_at,
                                updated_at: Utc::now(),
                            };

                            ws_manager.send_task_update(Some(&job.server_id), task).await;
                        }

                        // Check if job was cancelled
                        {
                            let jobs = jobs.read().await;
                            if let Some(job) = jobs.get(job_id) {
                                if job.status == JobStatus::Cancelled {
                                    info!("Job {} was cancelled, stopping generation", job_id);
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Mark job as completed
        {
            let mut jobs = jobs.write().await;
            if let Some(job) = jobs.get_mut(job_id) {
                job.status = JobStatus::Completed;
                job.progress = 1.0;
                job.finished_at = Some(Utc::now());
            }
        }

        // Update database
        db.update_task(&Task {
            id: job_id.to_string(),
            server_id: job.server_id.clone().into(),
            kind: "worldgen".to_string(),
            status: "completed".to_string(),
            progress: 1.0,
            log: Some("Generation completed successfully".to_string()),
            metadata: Some(serde_json::to_value(&config)?),
            started_at: job.started_at,
            finished_at: Some(Utc::now()),
            created_at: job.created_at,
            updated_at: Utc::now(),
        }).await?;

        info!("Completed generation process for job: {}", job_id);
        Ok(())
    }

    /// Generate a single chunk
    async fn generate_chunk(
        x: i32,
        z: i32,
        config: &PregenerationConfig,
        gpu_worker: &Option<Arc<Mutex<gpu_worker::GpuWorker>>>,
    ) -> Result<()> {
        if config.gpu_acceleration {
            if let Some(worker) = gpu_worker {
                let mut worker = worker.lock().await;
                // Use GPU worker to generate chunk
                // This would call the actual GPU generation
                debug!("GPU generating chunk ({}, {})", x, z);
            } else {
                // Fallback to CPU generation
                Self::generate_chunk_cpu(x, z, config).await?;
            }
        } else {
            // Use CPU generation
            Self::generate_chunk_cpu(x, z, config).await?;
        }

        Ok(())
    }

    /// Generate chunk using CPU (fallback)
    async fn generate_chunk_cpu(x: i32, z: i32, config: &PregenerationConfig) -> Result<()> {
        // This is a simplified CPU fallback
        // In a real implementation, this would use the server's world generation
        debug!("CPU generating chunk ({}, {})", x, z);
        
        // Simulate generation time
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        Ok(())
    }
}
