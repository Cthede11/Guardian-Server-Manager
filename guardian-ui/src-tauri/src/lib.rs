use tauri::Manager;
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
        .setup(|app: &mut tauri::App<tauri::Wry>| {
            log_debug("In setup function...");
            
            // Validate environment
            if let Err(e) = validate_environment(app.handle()) {
                log_debug(&format!("Environment validation failed: {}", e));
                return Err(e);
            }
            
            // Set up logging in debug mode only
            if cfg!(debug_assertions) {
                log_debug("Setting up debug logging...");
                // Skip plugin setup for now to avoid issues
                log_debug("Debug logging setup complete");
            }

            // Skip auto-updater for now to avoid issues
            log_debug("Skipping auto-updater setup...");

            // Initialize database first
            if let Err(e) = initialize_database(app.handle()) {
                log_debug(&format!("Database initialization failed: {}", e));
                return Err(e);
            }

            // Start backend services
            log_debug("Starting backend services...");
            
            // Start hostd with proper error handling
            log_debug("Attempting to start hostd service...");
            match start_hostd_service(app.handle()) {
                Ok(_) => {
                    log_debug("✅ Hostd started successfully");
                    // Give it a moment to initialize
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                },
                Err(e) => {
                    log_debug(&format!("❌ Failed to start hostd: {}", e));
                    log_debug("App will continue without hostd backend");
                    log_debug("You can manually start hostd.exe to enable server management");
                }
            }
            
            // Start GPU worker
            log_debug("Attempting to start GPU worker service...");
            match start_gpu_worker_service(app.handle()) {
                Ok(_) => {
                    log_debug("✅ GPU worker started successfully");
                },
                Err(e) => {
                    log_debug(&format!("❌ Failed to start GPU worker: {}", e));
                    log_debug("App will continue without GPU acceleration");
                }
            }

            log_debug("Setup complete successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_server,
            stop_server,
            restart_server,
            send_command,
            get_server_status,
            open_dev_tools,
            open_server_folder,
            get_debug_info
        ])
        .run(tauri::generate_context!());

    match result {
        Ok(_) => {
            log_debug("Tauri app completed successfully");
        }
        Err(e) => {
            log_debug(&format!("Tauri app failed: {:?}", e));
            eprintln!("Guardian failed to start: {:?}", e);
            eprintln!("Check guardian_debug.log for detailed error information");
            std::process::exit(1);
        }
    }
}

// Environment validation
fn validate_environment<R: tauri::Runtime>(handle: &tauri::AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    log_debug("Validating environment...");
    
    // Check resource directory exists
    let resource_dir = handle.path().resource_dir()?;
    log_debug(&format!("Resource directory: {:?}", resource_dir));
    
    if !resource_dir.exists() {
        return Err("Resource directory does not exist".into());
    }
    
    // Check required directories exist or can be created
    let data_dir = resource_dir.join("data");
    std::fs::create_dir_all(&data_dir)?;
    
    log_debug("Environment validation passed");
    Ok(())
}

// Database initialization
fn initialize_database<R: tauri::Runtime>(handle: &tauri::AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    log_debug("Initializing database...");
    
    let data_dir = handle.path().resource_dir()?.join("data");
    std::fs::create_dir_all(&data_dir)?;
    
    let db_path = data_dir.join("guardian.db");
    
    // Use rusqlite to properly initialize the database
    let conn = rusqlite::Connection::open(&db_path)?;
    
    // Create required tables
    conn.execute(
        "CREATE TABLE IF NOT EXISTS servers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            host TEXT NOT NULL DEFAULT '127.0.0.1',
            port INTEGER NOT NULL DEFAULT 25565,
            rcon_port INTEGER NOT NULL DEFAULT 25575,
            rcon_password TEXT NOT NULL DEFAULT '',
            java_path TEXT NOT NULL DEFAULT 'java',
            server_jar TEXT NOT NULL DEFAULT 'server.jar',
            jvm_args TEXT NOT NULL DEFAULT '-Xmx4G',
            server_args TEXT NOT NULL DEFAULT '',
            auto_start BOOLEAN NOT NULL DEFAULT FALSE,
            auto_restart BOOLEAN NOT NULL DEFAULT FALSE,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS health_checks (
            id TEXT PRIMARY KEY,
            timestamp TEXT NOT NULL,
            status TEXT NOT NULL,
            details TEXT
        )",
        [],
    )?;
    
    log_debug(&format!("Database initialized successfully: {:?}", db_path));
    Ok(())
}

// Enhanced resource path resolution
fn get_resource_path<R: tauri::Runtime>(handle: &tauri::AppHandle<R>, resource_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let app_dir = handle.path().resource_dir()?;
    log_debug(&format!("Looking for {} in app directory: {:?}", resource_name, app_dir));
    
    // Check multiple possible locations
    let possible_paths = vec![
        app_dir.join(resource_name),
        app_dir.join("bin").join(resource_name),
        app_dir.parent().unwrap_or(&app_dir).join(resource_name),
        std::env::current_dir()?.join(resource_name),
        // Add more specific paths for our setup
        std::env::current_dir()?.join("src-tauri").join("target").join("release").join(resource_name),
        std::env::current_dir()?.join("src-tauri").join(resource_name),
    ];
    
    for (i, path) in possible_paths.iter().enumerate() {
        log_debug(&format!("Checking path {}: {:?}", i + 1, path));
        if path.exists() {
            log_debug(&format!("Found {} at: {:?}", resource_name, path));
            return Ok(path.clone());
        }
    }
    
    // If not found, try to find it in the current working directory
    let current_dir = std::env::current_dir()?;
    log_debug(&format!("Current working directory: {:?}", current_dir));
    
    // List files in current directory to help debug
    if let Ok(entries) = std::fs::read_dir(&current_dir) {
        log_debug("Files in current directory:");
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                log_debug(&format!("  - {}", name));
            }
        }
    }
    
    Err(format!("Resource not found: {} (searched {} locations)", resource_name, possible_paths.len()).into())
}

// Add debug info command
#[tauri::command]
async fn get_debug_info() -> Result<String, String> {
    let info = format!(
        "Guardian Debug Information\n\
         Current Directory: {:?}\n\
         Environment: {}\n\
         Timestamp: {}",
        std::env::current_dir().unwrap_or_default(),
        if cfg!(debug_assertions) { "Debug" } else { "Release" },
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );
    log_debug("Debug info requested from frontend");
    Ok(info)
}

// Start the hostd service
fn start_hostd_service<R: tauri::Runtime>(handle: &tauri::AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    log_debug("Starting hostd service...");
    
    // Use enhanced resource path resolution
    let hostd_path = get_resource_path(handle, "hostd.exe")?;
    start_hostd_process(handle, &hostd_path)
}

// Start the GPU worker service
fn start_gpu_worker_service<R: tauri::Runtime>(handle: &tauri::AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    log_debug("Starting GPU worker service...");
    
    // Use enhanced resource path resolution
    let gpu_worker_path = get_resource_path(handle, "gpu-worker.exe")?;
    start_gpu_worker_process(handle, &gpu_worker_path)
}

// Helper function to start the hostd process
fn start_hostd_process<R: tauri::Runtime>(handle: &tauri::AppHandle<R>, hostd_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
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
    
    // Resolve configuration file path
    let config_path = handle.path().resource_dir()?.join("configs").join("hostd.yaml");
    if !config_path.exists() {
        return Err(format!("Configuration file not found: {:?}", config_path).into());
    }
    
    log_debug(&format!("Using config file: {:?}", config_path));
    
    // Get absolute path to database
    let db_path = data_dir.join("guardian.db");
    let db_url = format!("sqlite:{}", db_path.to_string_lossy());
    
    log_debug(&format!("Using database: {}", db_url));
    
    // Start the hostd process in the background with proper error handling
    let child = Command::new(hostd_path)
        .arg("--port")
        .arg("8080")
        .arg("--database-url")
        .arg(&db_url)
        .arg("--config")
        .arg(&config_path) // Use absolute path
        .arg("--log-level")
        .arg("info")
        .current_dir(handle.path().resource_dir()?) // Set working directory
        .spawn()?;
    
    log_debug(&format!("Hostd process started with PID: {:?}", child.id()));
    
    // Store the process handle in app state
    if let Ok(mut process) = handle.state::<AppState>().hostd_process.lock() {
        *process = Some(child);
    }
    
    Ok(())
}

// Helper function to start the GPU worker process
fn start_gpu_worker_process<R: tauri::Runtime>(handle: &tauri::AppHandle<R>, gpu_worker_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
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

#[tauri::command]
async fn open_server_folder(server_id: String) -> Result<(), String> {
    log_debug(&format!("Opening folder for server: {}", server_id));
    
    // Get the server directory path - try multiple possible locations
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    
    // Try to get the app data directory first
    let app_data_dir = dirs::data_dir()
        .ok_or("Failed to get app data directory")?
        .join("Guardian");
    
    let possible_paths = vec![
        // Check in the project directory (for development)
        current_dir.join("data").join("servers").join(&server_id),
        current_dir.join("world").join(&server_id),
        current_dir.join("servers").join(&server_id),
        // Check in app data directory (for installed app)
        app_data_dir.join("data").join("servers").join(&server_id),
        app_data_dir.join("world").join(&server_id),
        app_data_dir.join("servers").join(&server_id),
        // Check in user's home directory
        dirs::home_dir()
            .ok_or("Failed to get home directory")?
            .join("Guardian")
            .join("data")
            .join("servers")
            .join(&server_id),
        // Check in the actual project directory structure
        current_dir.join("..").join("data").join("servers").join(&server_id),
        current_dir.join("..").join("world").join(&server_id),
        current_dir.join("..").join("servers").join(&server_id),
    ];
    
    let mut server_dir = None;
    for path in &possible_paths {
        if path.exists() {
            server_dir = Some(path.clone());
            break;
        }
    }
    
    let server_dir = match server_dir {
        Some(dir) => dir,
        None => {
            // If no directory found, try to create one in the most likely location
            let fallback_dir = current_dir.join("data").join("servers").join(&server_id);
            log_debug(&format!("No server directory found, attempting to create: {:?}", fallback_dir));
            
            // Create the directory structure
            if let Err(e) = std::fs::create_dir_all(&fallback_dir) {
                let paths_str = possible_paths.iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(format!("Server directory not found and could not create fallback directory. Checked: {}. Error: {}", paths_str, e));
            }
            
            if !fallback_dir.exists() {
                let paths_str = possible_paths.iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err(format!("Server directory not found. Checked: {}", paths_str));
            }
            
            fallback_dir
        }
    };
    
    log_debug(&format!("Found server directory: {:?}", server_dir));
    
    // Open the folder in the system file explorer
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