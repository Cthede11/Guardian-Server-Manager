use tauri::{
    CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, 
    SystemTrayMenuItem, WindowEvent, WindowBuilder, WindowUrl
};
use std::process::Command;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Create system tray menu
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let show = CustomMenuItem::new("show".to_string(), "Show");
    let hide = CustomMenuItem::new("hide".to_string(), "Hide");
    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_item(hide)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick {
                position: _,
                size: _,
                ..
            } => {
                let window = app.get_window("main").unwrap();
                window.show().unwrap();
                window.set_focus().unwrap();
            }
            SystemTrayEvent::MenuItemClick { id, .. } => {
                match id.as_str() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "show" => {
                        let window = app.get_window("main").unwrap();
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                    "hide" => {
                        let window = app.get_window("main").unwrap();
                        window.hide().unwrap();
                    }
                    _ => {}
                }
            }
            _ => {}
        })
        .on_window_event(|event| match event.event() {
            WindowEvent::CloseRequested { api, .. } => {
                // Hide to tray instead of closing
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .setup(|app| {
            // Set up logging in debug mode
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Start the backend hostd process
            start_backend_process(app.handle())?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_server,
            stop_server,
            restart_server,
            send_command,
            get_server_status,
            open_file_dialog,
            show_notification
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Start the backend hostd process
fn start_backend_process(handle: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
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

#[tauri::command]
async fn open_file_dialog() -> Result<Option<String>, String> {
    use tauri::api::dialog::FileDialogBuilder;
    
    let result = FileDialogBuilder::new()
        .add_filter("Minecraft Server", &["jar"])
        .add_filter("All Files", &["*"])
        .pick_file();
    
    match result {
        Some(path) => Ok(Some(path.to_string_lossy().to_string())),
        None => Ok(None),
    }
}

#[tauri::command]
async fn show_notification(title: String, body: String) -> Result<(), String> {
    use tauri::api::notification::Notification;
    
    Notification::new("com.guardian.minecraft")
        .title(&title)
        .body(&body)
        .show()
        .map_err(|e| e.to_string())?;
    
    Ok(())
}
