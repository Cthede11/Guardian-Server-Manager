use gpu_worker::GpuWorker;
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting GPU Worker...");
    
    // Initialize GPU worker
    let mut worker = GpuWorker::new().await?;
    
    info!("GPU Worker started successfully");
    
    // Keep the worker running
    tokio::signal::ctrl_c().await?;
    
    info!("Shutting down GPU Worker...");
    worker.cleanup();
    
    Ok(())
}
