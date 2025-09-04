use hostd::*;
use hostd::config::Config;
use clap::Parser;
use tracing::{info, error};
use tracing_subscriber;
use axum::{
    Router,
    routing::get,
    response::Html,
};
use tower_http::cors::CorsLayer;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Parser)]
#[command(name = "hostd")]
#[command(about = "Guardian Host Daemon - Process supervision and high availability")]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "/configs/hostd.yaml")]
    config: String,
    
    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,
    
    /// Daemon mode
    #[arg(short, long)]
    daemon: bool,
    
    /// HTTP server port
    #[arg(short, long, default_value = "8080")]
    port: u16,
    
    /// Database URL
    #[arg(long, default_value = "sqlite:guardian.db")]
    database_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    // Initialize logging
    let log_level = args.log_level.parse::<tracing::Level>()?;
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .init();
    
    info!("Starting Guardian Host Daemon...");
    
    // Initialize database
    let db = database::DatabaseManager::new(&args.database_url).await?;
    info!("Database initialized: {}", args.database_url);
    
    // Initialize WebSocket manager
    let websocket_manager = websocket::WebSocketManager::new();
    info!("WebSocket manager initialized");
    
    // Initialize Minecraft manager
    let minecraft_manager = minecraft::MinecraftManager::new(db.clone());
    minecraft_manager.load_servers().await?;
    info!("Minecraft manager initialized");
    
    // Create application state
    let app_state = api::AppState {
        websocket_manager: websocket_manager.clone(),
        minecraft_manager: minecraft_manager.clone(),
        database: db.clone(),
    };
    
    // Create API router
    let api_router = api::create_api_router(app_state);
    
    // Create WebSocket router
    let ws_router = websocket::create_websocket_router();
    
    // Combine routers
    let app = Router::new()
        .merge(api_router)
        .merge(ws_router)
        .route("/", get(|| async { Html("<h1>Guardian Host Daemon</h1><p>API is running</p>") }))
        .layer(CorsLayer::permissive());
    
    // Start HTTP server
    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    info!("Starting HTTP server on {}", addr);
    
    let server_handle = tokio::spawn(async move {
        let listener = TcpListener::bind(&addr).await.unwrap();
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    });
    
    // Load configuration
    let config = Config::load(&args.config)?;
    info!("Configuration loaded from: {}", args.config);
    
    // Create host daemon
    let mut hostd = HostDaemon::new(config).await?;
    
    // Start the daemon
    if args.daemon {
        info!("Running in daemon mode");
        hostd.run_daemon().await?;
    } else {
        info!("Running in foreground mode");
        hostd.run().await?;
    }
    
    // Stop HTTP server
    server_handle.abort();
    
    info!("Guardian Host Daemon stopped");
    Ok(())
}
