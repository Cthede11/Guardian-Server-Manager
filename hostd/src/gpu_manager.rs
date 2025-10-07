use crate::core::guardian_config::GuardianConfig;
use gpu_worker::GpuWorker;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn, error};
use std::time::{Duration, Instant};
use serde::Serialize;

/// GPU metrics for monitoring
#[derive(Debug, Clone, Serialize)]
pub struct GpuMetrics {
    pub utilization: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub temperature: f32,
    pub power_usage: f32,
    #[serde(skip)]
    pub last_update: Instant,
}

impl Default for GpuMetrics {
    fn default() -> Self {
        Self {
            utilization: 0.0,
            memory_used: 0,
            memory_total: 0,
            temperature: 0.0,
            power_usage: 0.0,
            last_update: Instant::now(),
        }
    }
}

/// GPU job types
#[derive(Debug, Clone)]
pub enum GpuJobType {
    ChunkGeneration { x: i32, z: i32, seed: u64, dimension: String },
    Lighting { x: i32, z: i32, y: i32 },
    Pregeneration { center_x: i32, center_z: i32, radius: u32, seed: u64 },
}

/// GPU job result
#[derive(Debug, Clone)]
pub struct GpuJobResult {
    pub job_type: GpuJobType,
    pub success: bool,
    pub duration: Duration,
    pub error: Option<String>,
    pub data: Option<Vec<u8>>,
}

/// GPU Manager for coordinating GPU acceleration
#[derive(Clone)]
pub struct GpuManager {
    worker: Option<Arc<Mutex<GpuWorker>>>,
    config: GuardianConfig,
    metrics: Arc<Mutex<GpuMetrics>>,
    is_enabled: bool,
    cpu_usage_threshold: f32,
    last_cpu_check: Arc<Mutex<Instant>>,
}

impl GpuManager {
    /// Create a new GPU manager
    pub async fn new(config: GuardianConfig) -> Result<Self, String> {
        let mut manager = Self {
            worker: None,
            config: config.clone(),
            metrics: Arc::new(Mutex::new(GpuMetrics {
                utilization: 0.0,
                memory_used: 0,
                memory_total: 0,
                temperature: 0.0,
                power_usage: 0.0,
                last_update: Instant::now(),
            })),
            is_enabled: false, // TODO: Add GPU config to GuardianConfig
            cpu_usage_threshold: 0.8, // 80% CPU usage threshold
            last_cpu_check: Arc::new(Mutex::new(Instant::now())),
        };

        if manager.is_enabled {
            manager.initialize_gpu().await?;
        }

        Ok(manager)
    }

    /// Initialize the GPU worker
    async fn initialize_gpu(&mut self) -> Result<(), String> {
        info!("Initializing GPU worker...");
        
        match GpuWorker::new().await {
            Ok(worker) => {
                self.worker = Some(Arc::new(Mutex::new(worker)));
                info!("GPU worker initialized successfully");
                // Don't log GPU metrics here to avoid Send issues
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("GPU initialization failed: {}", e);
                warn!("Failed to initialize GPU worker: {}. Falling back to CPU.", error_msg);
                self.is_enabled = false;
                // Don't log GPU metrics here to avoid Send issues
                Ok(()) // Don't fail, just disable GPU
            }
        }
    }

    /// Submit a GPU job with safe fallback to CPU
    pub async fn submit_job(&self, job: GpuJobType) -> Result<GpuJobResult, String> {
        let start_time = Instant::now();
        
        // If GPU is disabled, immediately fall back to CPU
        if !self.is_enabled {
            return self.fallback_to_cpu(job, start_time).await;
        }

        // Check if we should use GPU based on CPU usage
        if !self.should_use_gpu().await {
            return self.fallback_to_cpu(job, start_time).await;
        }

        // Try GPU processing first
        match self.try_gpu_processing(job.clone(), start_time).await {
            Ok(result) => Ok(result),
            Err(gpu_error) => {
                tracing::warn!("GPU processing failed, falling back to CPU: {}", gpu_error);
                self.fallback_to_cpu(job, start_time).await
            }
        }
    }

    /// Try to process a job on GPU
    async fn try_gpu_processing(&self, job: GpuJobType, start_time: Instant) -> Result<GpuJobResult, String> {
        if let Some(worker) = &self.worker {
            let worker_guard = worker.lock().await;
            
            if !worker_guard.is_healthy() {
                return Err("GPU worker unhealthy".to_string());
            }

            // Submit the job based on type
            match job {
                GpuJobType::ChunkGeneration { x, z, seed, ref dimension } => {
                    // Simulate GPU processing with occasional failures for testing
                    if rand::random::<f32>() < 0.15 { // 15% chance of GPU failure
                        return Err("Simulated GPU processing failure".to_string());
                    }

                    // Simulate GPU processing time (faster than CPU)
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                    
                    let result = format!("GPU chunk generation result for ({}, {})", x, z);
                    self.update_metrics(0.7).await; // Assume 70% GPU utilization
                    
                    Ok(GpuJobResult {
                        job_type: job.clone(),
                        success: true,
                        duration: start_time.elapsed(),
                        error: None,
                        data: Some(result.into_bytes()),
                    })
                }
                _ => {
                    Err("Job type not implemented for GPU".to_string())
                }
            }
        } else {
            Err("GPU worker not available".to_string())
        }
    }

    /// Fallback to CPU processing when GPU fails
    async fn fallback_to_cpu(&self, job: GpuJobType, start_time: Instant) -> Result<GpuJobResult, String> {
        tracing::info!("Processing job on CPU as fallback");
        
        // Simulate CPU processing (longer than GPU)
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        
        match job {
            GpuJobType::ChunkGeneration { x, z, seed, ref dimension } => {
                let result = format!("CPU fallback chunk generation result for ({}, {})", x, z);
                
                Ok(GpuJobResult {
                    job_type: job.clone(),
                    success: true,
                    duration: start_time.elapsed(),
                    error: Some("Processed on CPU due to GPU unavailability".to_string()),
                    data: Some(result.into_bytes()),
                })
            }
            _ => {
                Ok(GpuJobResult {
                    job_type: job.clone(),
                    success: false,
                    duration: start_time.elapsed(),
                    error: Some("Job type not implemented for CPU fallback".to_string()),
                    data: None,
                })
            }
        }
    }

    /// Check if we should use GPU based on CPU usage and system conditions
    async fn should_use_gpu(&self) -> bool {
        if !self.is_enabled {
            return false;
        }

        // Check if enough time has passed since last CPU check
        let now = Instant::now();
        let last_check = *self.last_cpu_check.lock().await;
        if now.duration_since(last_check) < Duration::from_secs(2) {
            return true; // Use cached decision for better performance
        }

        // Get current system metrics
        let mut system = sysinfo::System::new_all();
        system.refresh_cpu();
        system.refresh_memory();
        
        let cpu_usage = system.global_cpu_info().cpu_usage() / 100.0;
        let memory_usage = system.used_memory() as f32 / system.total_memory() as f32;
        
        // Update last check time
        *self.last_cpu_check.lock().await = now;
        
        // Adaptive decision making based on multiple factors
        let should_use = self.calculate_adaptive_decision(cpu_usage, memory_usage).await;
        
        // Log the decision for debugging (avoiding recursion)
        tracing::info!(
            "Adaptive decision: CPU={:.1}%, Memory={:.1}%, UseGPU={}",
            cpu_usage * 100.0,
            memory_usage * 100.0,
            should_use
        );
        
        should_use
    }

    /// Calculate adaptive decision based on system metrics
    async fn calculate_adaptive_decision(&self, cpu_usage: f32, memory_usage: f32) -> bool {
        // Base CPU threshold
        let cpu_threshold = self.cpu_usage_threshold;
        
        // Memory threshold (don't use GPU if memory is too high)
        let memory_threshold = 0.85; // 85% memory usage
        
        // GPU worker health check
        let gpu_healthy = if let Some(worker) = &self.worker {
            let worker_guard = worker.lock().await;
            worker_guard.is_healthy()
        } else {
            false
        };
        
        // Adaptive thresholds based on current load
        let adaptive_cpu_threshold = if memory_usage > 0.7 {
            // If memory is high, be more conservative with CPU threshold
            cpu_threshold * 0.8
        } else if memory_usage < 0.3 {
            // If memory is low, be more aggressive with GPU usage
            cpu_threshold * 1.2
        } else {
            cpu_threshold
        };
        
        // Decision logic
        let use_gpu = gpu_healthy 
            && cpu_usage < adaptive_cpu_threshold 
            && memory_usage < memory_threshold
            && self.worker.is_some();
        
        // Additional safety checks
        if use_gpu {
            // Check if GPU has been working too hard recently
            let metrics = self.metrics.lock().await;
            if metrics.utilization > 0.9 {
                // GPU is at 90%+ utilization, give it a break
                return false;
            }
        }
        
        use_gpu
    }

    /// Update GPU metrics
    async fn update_metrics(&self, utilization: f32) {
        let mut metrics = self.metrics.lock().await;
        metrics.utilization = utilization;
        metrics.last_update = Instant::now();
    }

    /// Dynamically adjust CPU threshold based on system performance
    pub async fn adjust_cpu_threshold(&mut self) {
        let mut system = sysinfo::System::new_all();
        system.refresh_cpu();
        system.refresh_memory();
        
        let cpu_usage = system.global_cpu_info().cpu_usage() / 100.0;
        let memory_usage = system.used_memory() as f32 / system.total_memory() as f32;
        
        // Adjust threshold based on system performance
        if memory_usage > 0.8 {
            // High memory usage - be more conservative
            self.cpu_usage_threshold = (self.cpu_usage_threshold * 0.9).max(0.3);
        } else if memory_usage < 0.4 && cpu_usage < 0.5 {
            // Low system load - be more aggressive
            self.cpu_usage_threshold = (self.cpu_usage_threshold * 1.1).min(0.8);
        }
        
        // Log threshold adjustment
        self.log_gpu_metrics(&format!(
            "CPU threshold adjusted to {:.1}% (CPU: {:.1}%, Memory: {:.1}%)",
            self.cpu_usage_threshold * 100.0,
            cpu_usage * 100.0,
            memory_usage * 100.0
        )).await;
    }

    /// Get current GPU metrics
    pub async fn get_metrics(&self) -> GpuMetrics {
        self.metrics.lock().await.clone()
    }

    /// Check if GPU is enabled
    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    /// Enable or disable GPU
    pub async fn set_enabled(&mut self, enabled: bool) -> Result<(), String> {
        if enabled && !self.is_enabled {
            self.initialize_gpu().await?;
        } else if !enabled && self.is_enabled {
            if let Some(worker) = &self.worker {
                let mut worker_guard = worker.lock().await;
                worker_guard.cleanup();
            }
            self.worker = None;
        }
        
        self.is_enabled = enabled;
        self.log_gpu_metrics(&format!("GPU {}", if enabled { "enabled" } else { "disabled" })).await;
        Ok(())
    }

    /// Log GPU metrics to file
    async fn log_gpu_metrics(&self, message: &str) {
        let metrics = self.metrics.lock().await;
        let should_use_gpu = self.should_use_gpu().await;
        
        // Create detailed JSON log entry
        let log_entry = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "gpu_enabled": self.is_enabled,
            "utilization_percent": metrics.utilization * 100.0,
            "memory_used_mb": metrics.memory_used / 1024 / 1024,
            "memory_total_mb": metrics.memory_total / 1024 / 1024,
            "memory_usage_percent": if metrics.memory_total > 0 { 
                (metrics.memory_used as f32 / metrics.memory_total as f32) * 100.0 
            } else { 0.0 },
            "temperature_celsius": metrics.temperature,
            "power_usage_watts": metrics.power_usage,
            "worker_healthy": self.worker.is_some(),
            "cpu_usage_threshold": self.cpu_usage_threshold,
            "should_use_gpu": should_use_gpu,
            "message": message
        });

        // Write to guardian-gpu.log
        if let Err(e) = async {
            use tokio::io::AsyncWriteExt;
            let mut file = tokio::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("guardian-gpu.log")
                .await?;
            file.write_all(format!("{}\n", log_entry).as_bytes()).await?;
            Ok::<(), std::io::Error>(())
        }.await
        {
            error!("Failed to write GPU metrics: {}", e);
        }
    }

    /// Start periodic metrics logging and threshold adjustment
    pub async fn start_metrics_logging(&self) {
        let gpu_manager = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                gpu_manager.log_gpu_metrics("Periodic metrics update").await;
            }
        });

        // Start periodic threshold adjustment
        let gpu_manager = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                // Note: This would need to be mutable, but we can't make it mutable here
                // In a real implementation, you'd use a different approach like a channel
                // or make the threshold adjustment part of the metrics logging
            }
        });
    }

    /// Cleanup GPU resources
    pub async fn cleanup(&mut self) {
        if let Some(worker) = &self.worker {
            let mut worker_guard = worker.lock().await;
            worker_guard.cleanup();
        }
        self.worker = None;
        self.is_enabled = false;
        self.log_gpu_metrics("GPU manager cleanup completed").await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::guardian_config::GpuConfig;

    #[tokio::test]
    async fn test_gpu_manager_creation() {
        let config = GuardianConfig {
            gpu: GpuConfig {
                enabled: false,
                worker_ipc: "gpu-worker-ipc".to_string(),
                max_memory: 1024,
                timeout: 30,
            },
            ..Default::default()
        };

        let manager = GpuManager::new(config).await;
        assert!(manager.is_ok());
    }
}
