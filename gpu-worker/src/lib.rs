use std::os::raw::c_int;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};
use wgpu::*;
use anyhow::Result;

mod ffi;
mod kernels;

use ffi::*;
use kernels::ChunkGenerator;

/// Global GPU worker instance
static mut GPU_WORKER: Option<Arc<Mutex<GpuWorker>>> = None;

/// GPU Worker structure with real GPU acceleration
pub struct GpuWorker {
    device: Device,
    queue: Queue,
    chunk_generator: ChunkGenerator,
    is_healthy: bool,
    worker_id: String,
}

impl GpuWorker {
    /// Initialize the GPU worker with real GPU acceleration
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing GPU worker...");
        
        // Initialize WebGPU
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });
        
        // Get adapter
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or("Failed to get WebGPU adapter")?;
        
        info!("WebGPU adapter: {:?}", adapter.get_info());
        
        // Get device and queue
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("GPU Worker Device"),
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                },
                None,
            )
            .await?;
        
        info!("WebGPU device initialized successfully");
        
        // Initialize chunk generator
        let chunk_generator = ChunkGenerator::new(&device).await?;
        
        info!("GPU worker initialized successfully with real GPU acceleration");
        
        Ok(Self {
            device,
            queue,
            chunk_generator,
            is_healthy: true,
            worker_id: format!("gpu-worker-{}", uuid::Uuid::new_v4()),
        })
    }
    
    /// Submit a chunk generation job using real GPU acceleration
    pub async fn submit_chunk_job(&mut self, job: ChunkJob) -> Result<ChunkResult, Box<dyn std::error::Error>> {
        info!("Submitting GPU chunk job: ({}, {})", job.chunk_x, job.chunk_z);
        
        // Get dimension string from job
        let dimension = job.get_dimension();
        
        // Use real GPU chunk generation
        let chunk_data = self.chunk_generator.generate_chunk(
            &self.device,
            &self.queue,
            job.chunk_x,
            job.chunk_z,
            job.seed as u32,
            &dimension,
        ).await?;
        
        info!("GPU chunk generation completed for ({}, {})", job.chunk_x, job.chunk_z);
        
        // Convert ChunkData to ChunkResult
        let result = ChunkResult::new(
            job.chunk_x,
            job.chunk_z,
            job.seed,
            chunk_data.content_hash.to_string(),
            bytemuck::cast_slice(&chunk_data.density_data).to_vec(),
            bytemuck::cast_slice(&chunk_data.mask_data).to_vec(),
            bytemuck::cast_slice(&chunk_data.biome_data).to_vec(),
        );
        
        Ok(result)
    }
    
    /// Check if the GPU worker is healthy
    pub fn is_healthy(&self) -> bool {
        self.is_healthy
    }
    
    /// Get worker ID
    pub fn get_worker_id(&self) -> &str {
        &self.worker_id
    }
    
    /// Cleanup GPU resources
    pub fn cleanup(&mut self) {
        info!("Cleaning up GPU worker...");
        self.is_healthy = false;
        // GPU resources will be cleaned up automatically when dropped
    }
}

/// Initialize the GPU worker (C ABI)
#[no_mangle]
pub extern "C" fn gpuw_init() -> c_int {
    match pollster::block_on(GpuWorker::new()) {
        Ok(worker) => {
            unsafe {
                GPU_WORKER = Some(Arc::new(Mutex::new(worker)));
            }
            info!("GPU worker initialized successfully");
            0
        }
        Err(e) => {
            error!("Failed to initialize GPU worker: {}", e);
            -1
        }
    }
}

/// Submit a chunk job (C ABI)
#[no_mangle]
pub extern "C" fn gpuw_submit_chunk_job(job: ChunkJob, out_handle: *mut JobHandle) -> c_int {
    unsafe {
        if let Some(worker_arc) = &GPU_WORKER {
            let worker = worker_arc.clone();
            let rt = tokio::runtime::Runtime::new().unwrap();
            
            match rt.block_on(async {
                let mut worker_guard = worker.lock().await;
                worker_guard.submit_chunk_job(job).await
            }) {
                Ok(result) => {
                    let job_handle = JobHandle {
                        result: Some(result),
                        completed: true,
                    };
                    *out_handle = job_handle;
                    0
                }
                Err(e) => {
                    error!("Failed to submit chunk job: {}", e);
                    -1
                }
            }
        } else {
            error!("GPU worker not initialized");
            -1
        }
    }
}

/// Try to fetch a result (C ABI)
#[no_mangle]
pub extern "C" fn gpuw_try_fetch_result(handle: *mut JobHandle, out_result: *mut ChunkResult) -> c_int {
    unsafe {
        if handle.is_null() {
            return -1;
        }
        
        let job_handle = &*handle;
        if job_handle.completed {
            if let Some(ref result) = job_handle.result {
                *out_result = result.clone();
                0
            } else {
                -1
            }
        } else {
            -2 // Not ready yet
        }
    }
}

/// Free a result (C ABI)
#[no_mangle]
pub extern "C" fn gpuw_free_result(result: *mut ChunkResult) {
    if !result.is_null() {
        unsafe {
            let _ = Box::from_raw(result);
        }
    }
}

/// Health check (C ABI)
#[no_mangle]
pub extern "C" fn gpuw_health_check() -> c_int {
    unsafe {
        if let Some(worker_arc) = &GPU_WORKER {
            let worker = worker_arc.clone();
            let rt = tokio::runtime::Runtime::new().unwrap();
            
            rt.block_on(async {
                let worker_guard = worker.lock().await;
                if worker_guard.is_healthy() {
                    0
                } else {
                    -1
                }
            })
        } else {
            -1
        }
    }
}

/// Cleanup (C ABI)
#[no_mangle]
pub extern "C" fn gpuw_cleanup() {
    unsafe {
        if let Some(worker_arc) = &GPU_WORKER {
            let worker = worker_arc.clone();
            let rt = tokio::runtime::Runtime::new().unwrap();
            
            rt.block_on(async {
                let mut worker_guard = worker.lock().await;
                worker_guard.cleanup();
            });
            
            GPU_WORKER = None;
        }
    }
}