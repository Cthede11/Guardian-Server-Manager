use tauri::{Manager, Listener};
use std::process::{Command, Child};
use std::fs;
use std::io::Write;
use std::sync::Mutex;
use std::path::PathBuf;

// Global state to store the backend processes
struct AppState {
    hostd_process: Mutex<Option<Child>>,
    gpu_worker_process: Mutex<Option<Child>>,
}

// Simple logging function that writes to a file
fn log_debug(message: &str) {
    // Try to write to a simple log file
    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("guardian_debug.log")
    {
        let _ = file.write_all(format!("{}\n", message).as_bytes());
        let _ = file.flush();
    }
    
    // Also print to console
    println!("DEBUG: {}", message);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    log_debug("=== GUARDIAN APP STARTING ===");
    
    // Set up panic handler
    std::panic::set_hook(Box::new(|panic_info| {
        log_debug(&format!("PANIC: {:?}", panic_info));
    }));

    log_debug("Creating Tauri builder...");
    
    let result = tauri::Builder::default()
        .manage(AppState {
            hostd_process: Mutex::new(None),
            gpu_worker_process: Mutex::new(None),
        })
        .setup(|app| {
            log_debug("In setup function...");
            
            // Set up logging in debug mode
            if cfg!(debug_assertions) {
                log_debug("Setting up debug logging...");
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
                log_debug("Debug logging setup complete");
            }

            // Set up auto-updater (commented out due to panic)
            log_debug("Skipping auto-updater setup (causing panic)...");
            // app.handle().plugin(
            //     tauri_plugin_updater::Builder::new().build(),
            // )?;
            log_debug("Auto-updater setup skipped");

            // Start the backend services
            log_debug("Starting backend services...");
            
            // Start hostd
            if let Err(e) = start_hostd_service(app.handle()) {
                log_debug(&format!("Failed to start hostd: {}", e));
            } else {
                log_debug("Hostd started successfully");
            }
            
            // Start GPU worker
            if let Err(e) = start_gpu_worker_service(app.handle()) {
                log_debug(&format!("Failed to start GPU worker: {}", e));
            } else {
                log_debug("GPU worker started successfully");
            }
            
            // Backend process will be cleaned up automatically when the app exits

            log_debug("Setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_server,
            stop_server,
            restart_server,
            send_command,
            get_server_status,
            open_dev_tools
        ])
        .run(tauri::generate_context!());

    match result {
        Ok(_) => {
            log_debug("Tauri app ran successfully");
        }
        Err(e) => {
            log_debug(&format!("Tauri app failed: {:?}", e));
            panic!("Tauri app failed: {:?}", e);
        }
    }
}

// Start the hostd service
fn start_hostd_service(handle: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    log_debug("Starting hostd service...");
    
    // Try multiple possible locations for hostd
    let possible_paths = vec![
        // Try resource directory first
        handle.path().resource_dir()?.join("hostd.exe"),
        handle.path().resource_dir()?.join("hostd-x86_64-pc-windows-msvc.exe"),
        // Try current directory
        std::env::current_dir()?.join("hostd.exe"),
        std::env::current_dir()?.join("hostd-x86_64-pc-windows-msvc.exe"),
        // Try parent directory
        handle.path().resource_dir()?.parent().unwrap().join("hostd.exe"),
        handle.path().resource_dir()?.parent().unwrap().join("hostd-x86_64-pc-windows-msvc.exe"),
    ];
    
    log_debug(&format!("Searching {} possible locations for hostd", possible_paths.len()));
    
    for (i, hostd_path) in possible_paths.iter().enumerate() {
        log_debug(&format!("Checking location {}: {:?}", i + 1, hostd_path));
        
        if hostd_path.exists() {
            log_debug(&format!("Found hostd at: {:?}", hostd_path));
            return start_hostd_process(handle, hostd_path);
        }
    }
    
    log_debug("Could not find hostd in any location. App will run without hostd.");
    log_debug("Expected locations:");
    for (i, path) in possible_paths.iter().enumerate() {
        log_debug(&format!("  {} - {:?}", i + 1, path));
    }
    
    Ok(())
}

// Start the GPU worker service
fn start_gpu_worker_service(handle: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    log_debug("Starting GPU worker service...");
    
    // Try multiple possible locations for gpu-worker
    let possible_paths = vec![
        // Try resource directory first
        handle.path().resource_dir()?.join("gpu-worker.exe"),
        handle.path().resource_dir()?.join("gpu-worker-x86_64-pc-windows-msvc.exe"),
        // Try current directory
        std::env::current_dir()?.join("gpu-worker.exe"),
        std::env::current_dir()?.join("gpu-worker-x86_64-pc-windows-msvc.exe"),
        // Try parent directory
        handle.path().resource_dir()?.parent().unwrap().join("gpu-worker.exe"),
        handle.path().resource_dir()?.parent().unwrap().join("gpu-worker-x86_64-pc-windows-msvc.exe"),
    ];
    
    log_debug(&format!("Searching {} possible locations for gpu-worker", possible_paths.len()));
    
    for (i, gpu_worker_path) in possible_paths.iter().enumerate() {
        log_debug(&format!("Checking location {}: {:?}", i + 1, gpu_worker_path));
        
        if gpu_worker_path.exists() {
            log_debug(&format!("Found gpu-worker at: {:?}", gpu_worker_path));
            return start_gpu_worker_process(handle, gpu_worker_path);
        }
    }
    
    log_debug("Could not find gpu-worker in any location. App will run without GPU acceleration.");
    log_debug("Expected locations:");
    for (i, path) in possible_paths.iter().enumerate() {
        log_debug(&format!("  {} - {:?}", i + 1, path));
    }
    
    Ok(())
}

// Helper function to start the hostd process
fn start_hostd_process(handle: &tauri::AppHandle, hostd_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    log_debug(&format!("Starting hostd process from: {:?}", hostd_path));
    
    // Create data directory structure
    let data_dir = handle.path().resource_dir()?.join("data");
    let servers_dir = data_dir.join("servers");
    let backups_dir = data_dir.join("backups");
    let logs_dir = data_dir.join("logs");
    let gpu_cache_dir = data_dir.join("gpu-cache");
    
    std::fs::create_dir_all(&servers_dir)?;
    std::fs::create_dir_all(&backups_dir)?;
    std::fs::create_dir_all(&logs_dir)?;
    std::fs::create_dir_all(&gpu_cache_dir)?;
    
    log_debug(&format!("Created data directories: {:?}", data_dir));
    
    // Create empty database file if it doesn't exist (required for SQLite)
    let db_file = data_dir.join("guardian.db");
    if !db_file.exists() {
        std::fs::File::create(&db_file)?;
        log_debug(&format!("Created empty database file: {:?}", db_file));
    }
    
    // Get absolute path to database
    let db_path = data_dir.join("guardian.db");
    let db_url = format!("sqlite:{}", db_path.to_string_lossy());
    
    log_debug(&format!("Using database: {}", db_url));
    
    // Start the hostd process in the background with persistent database
    let child = Command::new(hostd_path)
        .arg("--port")
        .arg("8080")
        .arg("--database-url")
        .arg(&db_url)
        .arg("--config")
        .arg("configs/hostd.yaml")
        .arg("--log-level")
        .arg("info")
        .current_dir(handle.path().resource_dir()?)
        .spawn()?;
    
    log_debug(&format!("Hostd process started with PID: {:?}", child.id()));
    
    // Store the process handle in app state
    if let Ok(mut process) = handle.state::<AppState>().hostd_process.lock() {
        *process = Some(child);
    }
    
    Ok(())
}

// Helper function to start the GPU worker process
fn start_gpu_worker_process(handle: &tauri::AppHandle, gpu_worker_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    log_debug(&format!("Starting GPU worker process from: {:?}", gpu_worker_path));
    
    // Start the GPU worker process in the background
    let child = Command::new(gpu_worker_path)
        .arg("--log-level")
        .arg("info")
        .spawn()?;
    
    log_debug(&format!("GPU worker process started with PID: {:?}", child.id()));
    
    // Store the process handle in app state
    if let Ok(mut process) = handle.state::<AppState>().gpu_worker_process.lock() {
        *process = Some(child);
    }
    
    Ok(())
}

// Tauri commands for server management
#[tauri::command]
async fn start_server(server_id: String) -> Result<String, String> {
    // This would call the backend API
    log_debug(&format!("Starting server: {}", server_id));
    Ok(format!("Server {} started", server_id))
}

#[tauri::command]
async fn stop_server(server_id: String) -> Result<String, String> {
    // This would call the backend API
    log_debug(&format!("Stopping server: {}", server_id));
    Ok(format!("Server {} stopped", server_id))
}

#[tauri::command]
async fn restart_server(server_id: String) -> Result<String, String> {
    // This would call the backend API
    log_debug(&format!("Restarting server: {}", server_id));
    Ok(format!("Server {} restarted", server_id))
}

#[tauri::command]
async fn send_command(server_id: String, command: String) -> Result<String, String> {
    // This would call the backend API
    log_debug(&format!("Sending command to server {}: {}", server_id, command));
    Ok(format!("Command sent to server {}", server_id))
}

#[tauri::command]
async fn get_server_status(server_id: String) -> Result<String, String> {
    // This would call the backend API
    log_debug(&format!("Getting status for server: {}", server_id));
    Ok("running".to_string())
}

#[tauri::command]
async fn open_dev_tools() -> Result<(), String> {
    log_debug("Opening developer tools...");
    // In Tauri, we can't directly open dev tools from the backend
    // The frontend will handle this
    Ok(())
}