use tauri::Manager;
use tauri_plugin_dialog::DialogExt as _;
use std::process::{Command, Child, Stdio};
use std::fs;
use std::io::{Write, BufRead, BufReader};
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
        // register dialog plugin to enable native file dialogs from the frontend
        .plugin(tauri_plugin_dialog::init())
        // register HTTP plugin to enable HTTP requests from the frontend
        .plugin(tauri_plugin_http::init())
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
            get_debug_info,
            ensure_backend
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
    
    // Ensure user-writable application data directory exists
    let app_data_dir = dirs::data_dir().ok_or("Failed to resolve app data directory")?.join("Guardian");
    std::fs::create_dir_all(&app_data_dir)?;
    std::fs::create_dir_all(app_data_dir.join("data"))?;
    std::fs::create_dir_all(app_data_dir.join("logs"))?;
    
    log_debug("Environment validation passed");
    Ok(())
}

// Database initialization
fn initialize_database<R: tauri::Runtime>(handle: &tauri::AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    log_debug("Initializing database...");
    
    // Place the database under the user-writable AppData directory
    let app_data_dir = dirs::data_dir().ok_or("Failed to resolve app data directory")?.join("Guardian");
    let data_dir = app_data_dir.join("data");
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
        // Add paths for installed app
        app_dir.join("..").join(resource_name),
        app_dir.join("..").join("..").join(resource_name),
        // Try to find in the same directory as the executable
        std::env::current_exe()?.parent().unwrap().join(resource_name),
        std::env::current_exe()?.parent().unwrap().join("..").join(resource_name),
        std::env::current_exe()?.parent().unwrap().join("..").join("..").join(resource_name),
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
    log_debug(&format!("Current executable: {:?}", std::env::current_exe()?));
    
    // List files in current directory to help debug
    if let Ok(entries) = std::fs::read_dir(&current_dir) {
        log_debug("Files in current directory:");
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                log_debug(&format!("  - {}", name));
            }
        }
    }
    
    // List files in app directory to help debug
    if let Ok(entries) = std::fs::read_dir(&app_dir) {
        log_debug("Files in app directory:");
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
    start_hostd_process(handle, &hostd_path)
}

// Start the GPU worker service
fn start_gpu_worker_service<R: tauri::Runtime>(handle: &tauri::AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    log_debug("Starting GPU worker service...");
    
    // Use enhanced resource path resolution
    let gpu_worker_path = match get_resource_path(handle, "gpu-worker.exe") {
        Ok(path) => path,
        Err(_) => {
            log_debug("Could not find gpu-worker.exe in resource directory, trying build directory...");
            // Fallback to build directory
            let build_path = std::env::current_dir()?.join("build").join("executables").join("gpu-worker.exe");
            if build_path.exists() {
                log_debug(&format!("Found gpu-worker.exe in build directory: {:?}", build_path));
                build_path
            } else {
                return Err("Could not find gpu-worker.exe in resource directory or build directory".into());
            }
        }
    };
    start_gpu_worker_process(handle, &gpu_worker_path)
}

// Helper function to start the hostd process
fn start_hostd_process<R: tauri::Runtime>(handle: &tauri::AppHandle<R>, hostd_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    log_debug(&format!("Starting hostd process from: {:?}", hostd_path));
    
    // Use user-writable AppData directory for all runtime data
    let app_data_dir = dirs::data_dir().ok_or("Failed to resolve app data directory")?.join("Guardian");
    let data_dir = app_data_dir.join("data");
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
        // Fallback to project configs when running unpackaged (dev/testing)
        let project_config = std::env::current_dir()?.join("configs").join("hostd.yaml");
        if project_config.exists() {
            log_debug(&format!("Using fallback config from project: {:?}", project_config));
        } else {
            return Err(format!("Configuration file not found in resources or project: {:?}", config_path).into());
        }
    }
    
    log_debug(&format!("Using config file: {:?}", config_path));
    
    // Get database URL. We will set the working directory to the AppData folder,
    // so a relative sqlite path will place the DB in AppData/Guardian/data/guardian.db
    let db_url = "sqlite:data/guardian.db".to_string();
    
    log_debug(&format!("Using database: {}", db_url));
    
    // Start the hostd process in the background with proper error handling
    // The new consumer-ready hostd handles port selection and configuration internally
    let mut child = Command::new(hostd_path)
        .current_dir(&app_data_dir) // Set working directory to user-writable directory
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    
    log_debug(&format!("Hostd process started with PID: {:?}", child.id()));

    // Pipe stdout/stderr to guardian_hostd.log
    if let Some(out) = child.stdout.take() {
        let mut writer = fs::OpenOptions::new().create(true).append(true).open("guardian_hostd.log")?;
        std::thread::spawn(move || {
            let reader = BufReader::new(out);
            for line in reader.lines().flatten() {
                let _ = writeln!(writer, "[HOSTD OUT] {}", line);
            }
        });
    }
    if let Some(err) = child.stderr.take() {
        let mut writer = fs::OpenOptions::new().create(true).append(true).open("guardian_hostd.log")?;
        std::thread::spawn(move || {
            let reader = BufReader::new(err);
            for line in reader.lines().flatten() {
                let _ = writeln!(writer, "[HOSTD ERR] {}", line);
            }
        });
    }
    
    // Store the process handle in app state
    if let Ok(mut process) = handle.state::<AppState>().hostd_process.lock() {
        *process = Some(child);
    }
    
    Ok(())
}

// Ensure backend is running, attempt to start if not
#[tauri::command]
async fn ensure_backend<R: tauri::Runtime>(handle: tauri::AppHandle<R>) -> Result<String, String> {
    // Quick health check
    let ok = reqwest::Client::new()
        .get("http://127.0.0.1:8080/health")
        .timeout(std::time::Duration::from_millis(800))
        .send().await
        .map(|r| r.status().is_success())
        .unwrap_or(false);
    if ok {
        return Ok("backend_running".to_string());
    }
    // Try to start hostd
    if let Err(e) = start_hostd_service(&handle) {
        return Err(format!("failed_to_start_backend: {}", e));
    }
    // Wait briefly and recheck
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    let ok2 = reqwest::Client::new()
        .get("http://127.0.0.1:8080/health")
        .timeout(std::time::Duration::from_millis(1000))
        .send().await
        .map(|r| r.status().is_success())
        .unwrap_or(false);
    if ok2 { Ok("backend_started".to_string()) } else { Err("backend_unreachable".to_string()) }
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