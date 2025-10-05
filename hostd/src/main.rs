use axum::{
    routing::get,
    Router,
};
use tower_http::cors::{CorsLayer, Any};
use tracing_subscriber::{fmt, EnvFilter};
use std::sync::Arc;

use hostd::core::{
    app_state::AppState,
    config::Config,
    auth::AuthManager,
    error_handler::{AppError, Result},
    logging::{initialize_logging, LogConfig, LogFormat, LogOutput},
    performance::{PerformanceMonitor, PerformanceThresholds},
    caching::{CacheManager, CacheConfig, EvictionPolicy},
};
use hostd::api::{create_api_router, AppState as ApiAppState};
use hostd::routes::auth::auth_routes;
use hostd::websocket_manager::WebSocketManager;
use axum::{
    extract::{
        ws::WebSocketUpgrade,
        State,
    },
    response::Response,
};

async fn handle_websocket(
    ws: WebSocketUpgrade,
    State(manager): State<Arc<WebSocketManager>>,
) -> Response {
    ws.on_upgrade(|socket| manager.handle_socket(socket))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize comprehensive logging system
    let log_config = LogConfig {
        level: "info".to_string(),
        format: LogFormat::Pretty,
        output: LogOutput::Both,
        file_path: Some("logs/guardian.log".to_string()),
        max_file_size: Some(10 * 1024 * 1024), // 10MB
        max_files: Some(5),
        include_timestamp: true,
        include_thread_id: false,
        include_target: true,
        structured: true,
    };
    
    initialize_logging(log_config)
        .map_err(|e| AppError::ConfigurationError {
            message: format!("Failed to initialize logging: {}", e),
            config_key: "logging".to_string(),
            expected_type: "LogConfig".to_string(),
        })?;

    tracing::info!("Starting Guardian Server Manager...");

    // Initialize performance monitoring
    let performance_thresholds = PerformanceThresholds::default();
    let performance_monitor = Arc::new(PerformanceMonitor::new(performance_thresholds));
    performance_monitor.start_monitoring().await
        .map_err(|e| AppError::ConfigurationError {
            message: format!("Failed to start performance monitoring: {}", e),
            config_key: "performance_monitoring".to_string(),
            expected_type: "PerformanceMonitor".to_string(),
        })?;

    // Initialize cache manager
    let cache_manager = Arc::new(CacheManager::new());
    
    // Create API response cache
    let api_cache_config = CacheConfig {
        max_size: 1000,
        default_ttl: Some(std::time::Duration::from_secs(300)), // 5 minutes
        eviction_policy: EvictionPolicy::LRU,
        cleanup_interval: std::time::Duration::from_secs(60),
        enable_metrics: true,
    };
    let api_cache = cache_manager.create_cache::<String, serde_json::Value>(
        "api_responses".to_string(),
        api_cache_config,
    ).await;

    tracing::info!("Performance monitoring and caching initialized");

    // Load configuration
    let config = Config::load()
        .map_err(|e| AppError::ConfigurationError {
            message: format!("Failed to load config: {}", e),
            config_key: "config_file".to_string(),
            expected_type: "Config".to_string(),
        })?;

    // Create authentication manager
    let auth_manager = Arc::new(AuthManager::new("your-secret-key-here".to_string()));
    auth_manager.initialize().await
        .map_err(|e| AppError::ConfigurationError {
            message: format!("Failed to initialize auth: {}", e),
            config_key: "auth_manager".to_string(),
            expected_type: "AuthManager".to_string(),
        })?;

    // Create application state
    let app_state = Arc::new(AppState::new(config, auth_manager.clone()).await?);

    // Start the application
    app_state.start().await?;

    // Create the database manager
    let database = hostd::database::DatabaseManager::new("guardian.db").await?;
    
    // Create the API app state for the comprehensive router
    let api_app_state = ApiAppState {
        websocket_manager: app_state.websocket.clone(),
        minecraft_manager: hostd::minecraft::MinecraftManager::new(database.clone()),
        database: database.clone(),
        mod_manager: hostd::mod_manager::ModManager::new(std::path::PathBuf::from("mods")),
    };
    
    // Create the main router with auth routes
    let auth_router = auth_routes().with_state(app_state.clone());
    let api_router = create_api_router(api_app_state);
    
    let app = Router::new()
        .route("/", get(|| async { "Guardian Server Manager API" }))
        .merge(api_router)
        .nest("/api/auth", auth_router)
        .route("/ws", get(handle_websocket).with_state(app_state.websocket.clone()))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        );

    // Get the server address
    let addr = app_state.config.server_addr();
    
    tracing::info!("Guardian Server Manager listening on {}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await
        .map_err(|e| AppError::NetworkError {
            message: format!("Failed to bind to address: {}", e),
            endpoint: addr.to_string(),
            status_code: None,
        })?;

    axum::serve(listener, app).await
        .map_err(|e| AppError::NetworkError {
            message: format!("Server error: {}", e),
            endpoint: addr.to_string(),
            status_code: None,
        })?;

    // Cleanup on shutdown
    app_state.stop().await?;

    Ok(())
}