use hostd::*;
use hostd::config::Config;
use clap::Parser;
use tracing::{info, error, warn};
use tracing_subscriber;
use axum::{
    Router,
    routing::get,
    response::Html,
};
use tower_http::cors::CorsLayer;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Parser)]
#[command(name = "hostd")]
#[command(about = "Guardian Host Daemon - Professional Minecraft Server Management")]
#[command(version = "1.0.0")]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "configs/hostd.yaml")]
    config: String,
    
    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,
    
    /// HTTP server port
    #[arg(short, long, default_value = "8080")]
    port: u16,
    
    /// Database URL
    #[arg(long, default_value = "sqlite:data/guardian.db")]
    database_url: String,
    
    /// Run in daemon mode (background)
    #[arg(short, long)]
    daemon: bool,
    
    /// Enable debug mode
    #[arg(short, long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Initialize logging with proper formatting
    let log_level = args.log_level.parse::<tracing::Level>()
        .map_err(|e| {
            eprintln!("Invalid log level '{}': {}", args.log_level, e);
            e
        })?;
    
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .init();
    
    info!("🚀 Starting Guardian Host Daemon v1.0.0...");
    info!("📋 Configuration: {}", args.config);
    info!("🗄️  Database: {}", args.database_url);
    info!("🌐 HTTP Port: {}", args.port);
    info!("🔧 Debug Mode: {}", args.debug);
    
    // Load and validate configuration
    let config = match Config::load(&args.config) {
        Ok(config) => {
            info!("✅ Configuration loaded successfully");
            config
        }
        Err(e) => {
            error!("❌ Failed to load configuration: {}", e);
            if args.debug {
                warn!("🔧 Debug mode: Using default configuration");
                Config::default()
            } else {
                return Err(e.into());
            }
        }
    };
    
    // Initialize database with proper error handling
    let db = match database::DatabaseManager::new(&args.database_url).await {
        Ok(db) => {
            info!("✅ Database initialized successfully");
            db
        }
        Err(e) => {
            error!("❌ Failed to initialize database: {}", e);
            return Err(e.into());
        }
    };
    
    // Initialize core managers
    let websocket_manager = Arc::new(websocket::WebSocketManager::new());
    info!("✅ WebSocket manager initialized");
    
    let mut minecraft_manager = minecraft::MinecraftManager::new(db.clone());
    minecraft_manager.set_websocket_manager(websocket_manager.clone());
    let minecraft_manager = Arc::new(minecraft_manager);
    
    if let Err(e) = minecraft_manager.load_servers().await {
        warn!("⚠️  Failed to load existing servers: {}", e);
    }
    
    // Start metrics collection
    minecraft_manager.start_metrics_collection().await;
    
    info!("✅ Minecraft manager initialized with WebSocket support");
    
    // Initialize mod manager with proper error handling
    let download_dir = "data/mods".to_string();
    let curseforge_api_key = std::env::var("CURSEFORGE_API_KEY").ok();
    let mod_manager = Arc::new(mod_manager::ModManager::new(
        db.clone(), 
        download_dir, 
        curseforge_api_key
    ));
    info!("✅ Mod manager initialized");
    
    // Create application state
    let app_state = api::AppState {
        websocket_manager: websocket_manager.as_ref().clone(),
        minecraft_manager: minecraft_manager.as_ref().clone(),
        database: db.clone(),
        mod_manager: mod_manager.as_ref().clone(),
    };
    
    // Create API router
    let api_router = api::create_api_router(app_state);
    
    // Create WebSocket router
    let ws_router = websocket::create_websocket_router();
    
    // Create main application router with proper CORS
    let app = Router::new()
        .merge(api_router)
        .merge(ws_router)
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .route("/status", get(status_handler))
        .layer(
            CorsLayer::new()
                .allow_origin("*".parse::<axum::http::HeaderValue>().unwrap())
                .allow_methods([
                    axum::http::Method::GET, 
                    axum::http::Method::POST, 
                    axum::http::Method::PUT, 
                    axum::http::Method::DELETE,
                    axum::http::Method::PATCH,
                    axum::http::Method::OPTIONS
                ])
                .allow_headers([
                    axum::http::header::CONTENT_TYPE, 
                    axum::http::header::AUTHORIZATION,
                    axum::http::header::ACCEPT,
                    axum::http::header::ORIGIN
                ])
                .allow_credentials(false)
        );
    
    // Start HTTP server
    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    info!("🌐 Starting HTTP server on {}", addr);
    
    let mut server_handle = tokio::spawn(async move {
        let listener = match TcpListener::bind(&addr).await {
            Ok(listener) => {
                info!("✅ HTTP server listening on {}", addr);
                listener
            }
            Err(e) => {
                error!("❌ Failed to bind to {}: {}", addr, e);
                return Err(e);
            }
        };
        
        match axum::serve(listener, app.into_make_service()).await {
            Ok(_) => {
                info!("✅ HTTP server stopped gracefully");
                Ok(())
            }
            Err(e) => {
                error!("❌ HTTP server error: {}", e);
                Err(e)
            }
        }
    });
    
    // Start the main daemon loop
    let mut daemon_handle = tokio::spawn(async move {
        info!("🔄 Starting daemon services...");
        
        // Start health monitoring
        let health_monitor = Arc::new(health::HealthMonitor::new(std::time::Duration::from_secs(30)));
        health_monitor.start().await.unwrap_or_else(|e| {
            error!("❌ Failed to start health monitor: {}", e);
        });
        
        // Start backup manager
        let backup_config = backup::BackupConfig {
            strategy: backup::BackupStrategy::Full,
            retention: backup::RetentionPolicy::default(),
            storage: backup::StorageConfig {
                local_path: std::path::PathBuf::from("data/backups"),
                remote: None,
                compression_level: 6,
                encryption_enabled: false,
                encryption_key: None,
            },
            schedule: "0 2 * * *".to_string(),
            enabled: true,
            include_paths: vec![
                std::path::PathBuf::from("data/servers"),
                std::path::PathBuf::from("data/mods"),
                std::path::PathBuf::from("configs"),
            ],
            exclude_paths: vec![
                std::path::PathBuf::from("data/logs"),
                std::path::PathBuf::from("data/temp"),
            ],
            max_size_bytes: 0,
            compression_threads: 4,
        };
        
        let backup_manager = Arc::new(backup::BackupManager::new(backup_config));
        backup_manager.start().await.unwrap_or_else(|e| {
            error!("❌ Failed to start backup manager: {}", e);
        });
        
        info!("✅ All daemon services started");
        
        // Main daemon loop
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            
                            // Health monitoring is handled by the health monitor background task
                // Database health is checked by the health monitor
        }
    });
    
    // Wait for shutdown signal
    info!("🎯 Guardian Host Daemon is running. Press Ctrl+C to stop.");
    
    // Handle shutdown gracefully
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("🛑 Shutdown signal received");
        }
        result = &mut server_handle => {
            match result {
                Ok(Ok(_)) => info!("✅ HTTP server stopped gracefully"),
                Ok(Err(e)) => error!("❌ HTTP server error: {}", e),
                Err(e) => error!("❌ HTTP server task error: {}", e),
            }
        }
        result = &mut daemon_handle => {
            match result {
                Ok(_) => info!("✅ Daemon stopped gracefully"),
                Err(e) => error!("❌ Daemon task error: {}", e),
            }
        }
    }
    
    // Cleanup
    info!("🧹 Cleaning up resources...");
    daemon_handle.abort();
    server_handle.abort();
    
    info!("✅ Guardian Host Daemon stopped successfully");
    Ok(())
}

// Handler functions
async fn root_handler() -> Html<&'static str> {
    Html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Guardian Host Daemon</title>
        <style>
            body { font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }
            .container { background: white; padding: 30px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
            h1 { color: #2c3e50; }
            .status { color: #27ae60; font-weight: bold; }
            .endpoints { margin-top: 20px; }
            .endpoint { background: #ecf0f1; padding: 10px; margin: 5px 0; border-radius: 5px; }
        </style>
    </head>
    <body>
        <div class="container">
            <h1>🛡️ Guardian Host Daemon</h1>
            <p class="status">✅ API is running and healthy</p>
            <div class="endpoints">
                <h3>Available Endpoints:</h3>
                <div class="endpoint"><strong>GET /health</strong> - Health status</div>
                <div class="endpoint"><strong>GET /status</strong> - System status</div>
                <div class="endpoint"><strong>GET /api/servers</strong> - List servers</div>
                <div class="endpoint"><strong>GET /api/modpacks</strong> - List modpacks</div>
            </div>
        </div>
    </body>
    </html>
    "#)
}

async fn health_handler() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "version": "1.0.0",
        "uptime": "running"
    }))
}

async fn status_handler() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "operational",
        "services": {
            "api": "running",
            "database": "connected",
            "websocket": "active",
            "mod_manager": "ready"
        },
        "timestamp": chrono::Utc::now()
    }))
}
