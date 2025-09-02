use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

mod kernels;
mod ffi;

use kernels::ChunkGenerator;
use ffi::*;

/// Global GPU worker instance
static mut GPU_WORKER: Option<Arc<Mutex<GpuWorker>>> = None;

/// GPU Worker structure
pub struct GpuWorker {
    device: wgpu::Device,
    queue: wgpu::Queue,
    chunk_generator: ChunkGenerator,
    is_healthy: bool,
}

impl GpuWorker {
    /// Initialize the GPU worker
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing GPU worker...");
        
        // Initialize wgpu
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        // Get adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or("Failed to get GPU adapter")?;
        
        info!("Using GPU adapter: {:?}", adapter.get_info());
        
        // Get device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Guardian GPU Worker"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;
        
        // Initialize chunk generator
        let chunk_generator = ChunkGenerator::new(&device).await?;
        
        info!("GPU worker initialized successfully");
        
        Ok(Self {
            device,
            queue,
            chunk_generator,
            is_healthy: true,
        })
    }
    
    /// Submit a chunk generation job
    pub async fn submit_chunk_job(&mut self, job: ChunkJob) -> Result<ChunkResult, Box<dyn std::error::Error>> {
        info!("Submitting chunk job: ({}, {})", job.chunk_x, job.chunk_z);
        
        // Generate chunk using GPU
        let result = self.chunk_generator.generate_chunk(
            &self.device,
            &self.queue,
            job.chunk_x,
            job.chunk_z,
            job.seed,
            &job.dimension,
        ).await?;
        
        info!("Chunk generation completed: ({}, {})", job.chunk_x, job.chunk_z);
        Ok(result)
    }
    
    /// Perform health check
    pub fn health_check(&mut self) -> bool {
        // Simple health check - in a real implementation, this would test GPU functionality
        self.is_healthy = true;
        self.is_healthy
    }
    
    /// Cleanup resources
    pub fn cleanup(&mut self) {
        info!("Cleaning up GPU worker...");
        self.is_healthy = false;
    }
}

/// Initialize the GPU worker (C ABI)
#[no_mangle]
pub extern "C" fn gpuw_init() -> c_int {
    let rt = tokio::runtime::Runtime::new().unwrap();
    match rt.block_on(GpuWorker::new()) {
        Ok(worker) => {
            unsafe {
                GPU_WORKER = Some(Arc::new(Mutex::new(worker)));
            }
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
            let rt = tokio::runtime::Handle::current();
            match rt.block_on(async {
                let mut worker = worker_arc.lock().await;
                worker.submit_chunk_job(job).await
            }) {
                Ok(result) => {
                    // Create job handle and store result
                    let handle = JobHandle {
                        result: Some(result),
                        completed: true,
                    };
                    ptr::write(out_handle, handle);
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
        let handle_ref = &*handle;
        if handle_ref.completed {
            if let Some(result) = &handle_ref.result {
                ptr::write(out_result, result.clone());
                0
            } else {
                -1
            }
        } else {
            -2 // Not ready
        }
    }
}

/// Free a result (C ABI)
#[no_mangle]
pub extern "C" fn gpuw_free_result(result: *mut ChunkResult) {
    unsafe {
        if !result.is_null() {
            ptr::drop_in_place(result);
        }
    }
}

/// Health check (C ABI)
#[no_mangle]
pub extern "C" fn gpuw_health_check() -> c_int {
    unsafe {
        if let Some(worker_arc) = &GPU_WORKER {
            let rt = tokio::runtime::Handle::current();
            match rt.block_on(async {
                let mut worker = worker_arc.lock().await;
                worker.health_check()
            }) {
                true => 0,
                false => -1,
            }
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
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mut worker = worker_arc.lock().await;
                worker.cleanup();
            });
            GPU_WORKER = None;
        }
    }
}
