use tauri::Manager;
use std::process::{Command, Child, Stdio};
use std::fs;
use std::io::Write;
use std::sync::Mutex;
use std::path::PathBuf;
use tokio::time::sleep;

// Import our modules
mod dto;
mod commands;
mod events;
mod gpu_integration;

// Global state to store the backend processes
struct AppState {
    hostd_process: Mutex<Option<Child>>,
    gpu_worker_process: Mutex<Option<Child>>,
}

// Enhanced logging function
fn log_debug(message: &str) {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let log_message = format!("[{}] {}", timestamp, message);
    
    // Write to file
    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("guardian_debug.log")
    {
        let _ = file.write_all(format!("{}\n", log_message).as_bytes());
        let _ = file.flush();
    }
    
    // Also print to console
    println!("DEBUG: {}", log_message);
}

// Start backend command - this is the key addition
#[tauri::command]
async fn start_backend() -> Result<String, String> {
    log_debug("Starting backend via Tauri command...");
    
    // 1) Try existing healthy backend (probe 52100–52150 /healthz)
    let mut base = None;
    for port in 52100..=52150 {
        let url = format!("http://127.0.0.1:{}/healthz", port);
        if let Ok(resp) = reqwest::get(&url).await {
            if resp.status().is_success() { 
                base = Some(format!("http://127.0.0.1:{}", port)); 
                log_debug(&format!("Found existing healthy backend on port {}", port));
                break; 
            }
        }
    }
    if let Some(base_url) = base {
        return Ok(base_url);
    }

    // 2) Spawn sidecar hostd
    log_debug("No existing backend found, spawning sidecar hostd...");
    let child = Command::new("hostd")
        .spawn()
        .map_err(|e| format!("failed to spawn hostd: {e}"))?;

    log_debug("Hostd sidecar spawned successfully");

    // 3) Wait for healthz (max 20s)
    log_debug("Waiting for backend to become healthy...");
    let start = std::time::Instant::now();
    loop {
        for port in 52100..=52150 {
            let url = format!("http://127.0.0.1:{}/healthz", port);
            if let Ok(resp) = reqwest::get(&url).await {
                if resp.status().is_success() {
                    log_debug(&format!("Backend became healthy on port {}", port));
                    return Ok(format!("http://127.0.0.1:{}", port));
                }
            }
        }
        if start.elapsed().as_secs() > 20 {
            break;
        }
        sleep(std::time::Duration::from_millis(250)).await;
    }

    Err("backend did not become healthy within 20s".to_string())
}

// Ensure backend is running, attempt to start if not
#[tauri::command]
async fn ensure_backend<R: tauri::Runtime>(handle: tauri::AppHandle<R>) -> Result<String, String> {
    // Try to find existing healthy backend first
    for port in 52100..=52150 {
        let url = format!("http://127.0.0.1:{}/healthz", port);
        if let Ok(resp) = reqwest::get(&url).await {
            if resp.status().is_success() {
                return Ok("backend_running".to_string());
            }
        }
    }
    
    // If no healthy backend found, try to start one
    start_backend().await
}

// Start the hostd service
fn start_hostd_service<R: tauri::Runtime>(handle: &tauri::AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    log_debug("Starting hostd service...");
    
    // Use enhanced resource path resolution
    let hostd_path = match get_resource_path(handle, "hostd.exe") {
        Ok(path) => path,
        Err(_) => {
            log_debug("Could not find hostd.exe in resource directory, trying build directory...");
            // Fallback to build directory
            let build_path = std::env::current_dir()?.join("build").join("executables").join("hostd.exe");
            if build_path.exists() {
                log_debug(&format!("Found hostd.exe in build directory: {:?}", build_path));
                build_path
            } else {
                return Err("Could not find hostd.exe in resource directory or build directory".into());
            }
        }
    };

    log_debug(&format!("Starting hostd process from: {:?}", hostd_path));

    // Create data directories
    let data_dir = dirs::data_dir().unwrap_or_else(|| std::env::current_dir().unwrap()).join("Guardian").join("data");
    fs::create_dir_all(&data_dir).map_err(|e| format!("Failed to create data directory: {}", e))?;
    log_debug(&format!("Created data directories: {:?}", data_dir));

    // Start hostd process
    let mut hostd_cmd = Command::new(&hostd_path);
    hostd_cmd
        .arg("--config")
        .arg(get_resource_path(handle, "configs/hostd.yaml").unwrap_or_else(|_| "configs/hostd.yaml".into()))
        .arg("--database")
        .arg(format!("sqlite:{}", data_dir.join("guardian.db").display()))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut hostd_process = hostd_cmd.spawn()?;
    let hostd_pid = hostd_process.id();
    log_debug(&format!("Hostd process started with PID: {}", hostd_pid));

    // Store the process in app state
    if let Some(state) = handle.try_state::<AppState>() {
        if let Ok(mut hostd_guard) = state.hostd_process.lock() {
            *hostd_guard = Some(hostd_process);
        }
    }

    log_debug("✅ Hostd started successfully");
    Ok(())
}

// Start the GPU worker service
fn start_gpu_worker_service<R: tauri::Runtime>(handle: &tauri::AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    log_debug("Starting GPU worker service...");
    
    let gpu_worker_path = match get_resource_path(handle, "gpu-worker.exe") {
        Ok(path) => path,
        Err(_) => {
            log_debug("Could not find gpu-worker.exe in resource directory, trying build directory...");
            let build_path = std::env::current_dir()?.join("build").join("executables").join("gpu-worker.exe");
            if build_path.exists() {
                log_debug(&format!("Found gpu-worker.exe in build directory: {:?}", build_path));
                build_path
            } else {
                return Err("Could not find gpu-worker.exe in resource directory or build directory".into());
            }
        }
    };

    log_debug(&format!("Starting GPU worker process from: {:?}", gpu_worker_path));

    let mut gpu_worker_cmd = Command::new(&gpu_worker_path);
    gpu_worker_cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut gpu_worker_process = gpu_worker_cmd.spawn()?;
    let gpu_worker_pid = gpu_worker_process.id();
    log_debug(&format!("GPU worker process started with PID: {}", gpu_worker_pid));

    // Store the process in app state
    if let Some(state) = handle.try_state::<AppState>() {
        if let Ok(mut gpu_worker_guard) = state.gpu_worker_process.lock() {
            *gpu_worker_guard = Some(gpu_worker_process);
        }
    }

    log_debug("✅ GPU worker started successfully");
    Ok(())
}

// Enhanced resource path resolution
fn get_resource_path<R: tauri::Runtime>(handle: &tauri::AppHandle<R>, resource: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Try to get resource path from Tauri
    if let Ok(resource_path) = handle.path().resource_dir() {
        let full_path = resource_path.join(resource);
        if full_path.exists() {
            return Ok(full_path);
        }
    }
    
    // Fallback to current directory
    let current_dir = std::env::current_dir()?;
    let fallback_path = current_dir.join(resource);
    if fallback_path.exists() {
        return Ok(fallback_path);
    }
    
    Err(format!("Resource not found: {}", resource).into())
}

// Open server folder command
#[tauri::command]
async fn open_server_folder(server_id: String) -> Result<(), String> {
    log_debug(&format!("Opening server folder for server: {}", server_id));
    
    // Get the server directory path
    let server_dir = dirs::data_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("Guardian")
        .join("servers")
        .join(&server_id);

    if !server_dir.exists() {
        return Err(format!("Server directory does not exist: {:?}", server_dir));
    }

    #[cfg(target_os = "windows")]
    {
        let result = Command::new("explorer")
            .arg(&server_dir)
            .spawn();
        
        match result {
            Ok(_) => {
                log_debug(&format!("Opened server folder: {:?}", server_dir));
                Ok(())
            }
            Err(e) => {
                log_debug(&format!("Failed to open server folder: {}", e));
                Err(format!("Failed to open server folder: {}", e))
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        let result = Command::new("open")
            .arg(&server_dir)
            .spawn();
        
        match result {
            Ok(_) => {
                log_debug(&format!("Opened server folder: {:?}", server_dir));
                Ok(())
            }
            Err(e) => {
                log_debug(&format!("Failed to open server folder: {}", e));
                Err(format!("Failed to open server folder: {}", e))
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        let result = Command::new("xdg-open")
            .arg(&server_dir)
            .spawn();
        
        match result {
            Ok(_) => {
                log_debug(&format!("Opened server folder: {:?}", server_dir));
                Ok(())
            }
            Err(e) => {
                log_debug(&format!("Failed to open server folder: {}", e));
                Err(format!("Failed to open server folder: {}", e))
            }
        }
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err("Opening folders is not supported on this platform".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    log_debug("=== GUARDIAN APP STARTING ===");
    
    // Enhanced panic handler
    std::panic::set_hook(Box::new(|panic_info| {
        let backtrace = std::backtrace::Backtrace::capture();
        log_debug(&format!("PANIC: {:?}", panic_info));
        log_debug(&format!("BACKTRACE: {:?}", backtrace));
        
        // Try to show error dialog if possible
        eprintln!("Guardian crashed! Check guardian_debug.log for details.");
    }));

    log_debug("Creating Tauri builder...");
    
    let result = tauri::Builder::default()
        .manage(AppState {
            hostd_process: Mutex::new(None),
            gpu_worker_process: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            start_backend,
            ensure_backend,
            open_server_folder,
            // Server management
            commands::get_server_summary,
            commands::get_servers,
            commands::create_server,
            commands::delete_server,
            // Server control
            commands::start_server,
            commands::stop_server,
            commands::restart_server,
            commands::promote_server,
            // Console and commands
            commands::send_rcon,
            commands::get_console_messages,
            // Server health and metrics
            commands::get_server_health,
            commands::get_players,
            commands::get_metrics,
            // Player actions
            commands::kick_player,
            commands::ban_player,
            // Backups
            commands::get_backups,
            commands::create_backup,
            commands::delete_backup,
            commands::restore_backup,
            // World management
            commands::get_freeze_tickets,
            commands::thaw_world,
            // Pregen jobs
            commands::get_pregen_jobs,
            commands::create_pregen_job,
            commands::start_pregen_job,
            commands::stop_pregen_job,
            commands::delete_pregen_job,
            // Mods and rules
            commands::get_mods,
            commands::get_rules,
            commands::get_conflicts,
            // Settings
            commands::get_server_settings,
            commands::update_server_settings,
            // Sharding
            commands::get_sharding_topology,
            commands::get_shard_assignments,
            // Events
            commands::get_events,
            commands::create_event,
            // GPU status
            commands::get_gpu_status,
        ])
        .setup(|app| {
            log_debug("In setup function...");
            
            // Export TypeScript types
            log_debug("Exporting TypeScript types...");
            // TODO: Fix tauri-specta API usage - temporarily disabled
            // tauri_specta::ts::export(
            //     specta::collect_types![
            //         dto::ServerSummary,
            //         dto::BlueGreen,
            //         dto::ConsoleLines,
            //         dto::ConsoleLine,
            //         dto::Metrics,
            //         dto::Player,
            //         dto::FreezeTicket,
            //         dto::Location,
            //         dto::Snapshot,
            //         dto::Rule,
            //         dto::PregenJob,
            //         dto::Region,
            //         dto::ServerHealth,
            //         dto::ServerSettings,
            //         dto::GeneralSettings,
            //         dto::JVMSettings,
            //         dto::GPUSettings,
            //         dto::HASettings,
            //         dto::PathSettings,
            //         dto::ComposerSettings,
            //         dto::TokenSettings,
            //         dto::ModInfo,
            //         dto::Conflict,
            //         dto::Event,
            //         dto::Shard,
            //         dto::ShardingTopology,
            //         dto::ShardAssignment,
            //         dto::CrashSignature,
            //         dto::ApiResponse,
            //     ],
            //     "../src/lib/types.gen.ts"
            // ).expect("Failed to export TypeScript types");
            log_debug("TypeScript types export temporarily disabled");
            
            // Validate environment
            log_debug("Validating environment...");
            let resource_dir = app.path().resource_dir().unwrap_or_else(|_| std::env::current_dir().unwrap());
            log_debug(&format!("Resource directory: {:?}", resource_dir));
            log_debug("Environment validation passed");
            
            // Skip auto-updater setup for now
            log_debug("Skipping auto-updater setup...");
            
            // Initialize database
            log_debug("Initializing database...");
            let data_dir = dirs::data_dir().unwrap_or_else(|| std::env::current_dir().unwrap()).join("Guardian").join("data");
            fs::create_dir_all(&data_dir).map_err(|e| format!("Failed to create data directory: {}", e))?;
            let db_path = data_dir.join("guardian.db");
            log_debug(&format!("Database initialized successfully: {:?}", db_path));
            
            // Start backend services
            log_debug("Starting backend services...");
            
            // Try to start hostd service
            log_debug("Attempting to start hostd service...");
            if let Err(e) = start_hostd_service(app.handle()) {
                log_debug(&format!("Failed to start hostd service: {}", e));
                // Don't fail the entire app startup, just log the error
            }
            
            // Try to start GPU worker service
            log_debug("Attempting to start GPU worker service...");
            if let Err(e) = start_gpu_worker_service(app.handle()) {
                log_debug(&format!("Failed to start GPU worker service: {}", e));
                // Don't fail the entire app startup, just log the error
            }
            
            // Initialize GPU integration
            log_debug("Initializing GPU integration...");
            if let Err(e) = gpu_integration::init_gpu_integration() {
                log_debug(&format!("Failed to initialize GPU integration: {}", e));
            }
            
            log_debug("Setup complete successfully");
            Ok(())
        })
        .run(tauri::generate_context!());

    match result {
        Ok(_) => {
            log_debug("Guardian app exited normally");
        }
        Err(e) => {
            log_debug(&format!("Guardian app failed to start: {}", e));
            eprintln!("Failed to start Guardian: {}", e);
        }
    }
}