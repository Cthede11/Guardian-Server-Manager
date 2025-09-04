use tauri::Manager;
use std::process::Command;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Set up logging in debug mode
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Set up auto-updater
            app.handle().plugin(
                tauri_plugin_updater::Builder::new().build(),
            )?;

            // Start the backend hostd process
            start_backend_process(app.handle())?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_server,
            stop_server,
            restart_server,
            send_command,
            get_server_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Start the backend hostd process
fn start_backend_process(_handle: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // Try to start the hostd process
    let output = Command::new("hostd")
        .arg("--port")
        .arg("8080")
        .arg("--database-url")
        .arg("sqlite:guardian.db")
        .spawn();

    match output {
        Ok(_) => {
            log::info!("Backend hostd process started successfully");
        }
        Err(e) => {
            log::warn!("Failed to start backend hostd process: {}", e);
            // Continue without backend - will use mock data
        }
    }

    Ok(())
}

// Tauri commands for server management
#[tauri::command]
async fn start_server(server_id: String) -> Result<String, String> {
    // This would call the backend API
    log::info!("Starting server: {}", server_id);
    Ok(format!("Server {} started", server_id))
}

#[tauri::command]
async fn stop_server(server_id: String) -> Result<String, String> {
    // This would call the backend API
    log::info!("Stopping server: {}", server_id);
    Ok(format!("Server {} stopped", server_id))
}

#[tauri::command]
async fn restart_server(server_id: String) -> Result<String, String> {
    // This would call the backend API
    log::info!("Restarting server: {}", server_id);
    Ok(format!("Server {} restarted", server_id))
}

#[tauri::command]
async fn send_command(server_id: String, command: String) -> Result<String, String> {
    // This would call the backend API
    log::info!("Sending command to server {}: {}", server_id, command);
    Ok(format!("Command sent to server {}", server_id))
}

#[tauri::command]
async fn get_server_status(server_id: String) -> Result<String, String> {
    // This would call the backend API
    log::info!("Getting status for server: {}", server_id);
    Ok("running".to_string())
}