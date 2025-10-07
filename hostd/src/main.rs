use axum::{
    routing::get,
    Router,
};
use tower_http::cors::{CorsLayer, Any};
use std::sync::Arc;

use hostd::core::{
    app_state::AppState,
    config::Config,
    guardian_config::GuardianConfig,
    resource_monitor::ResourceMonitor,
    crash_watchdog::{CrashWatchdog, WatchdogConfig},
    monitoring::MonitoringManager,
    auth::AuthManager,
    error_handler::{AppError, Result},
    logging::{initialize_logging, LogConfig, LogFormat, LogOutput},
    performance::{PerformanceMonitor, PerformanceThresholds},
    caching::{CacheManager, CacheConfig, EvictionPolicy},
};
use hostd::api::{create_api_router, AppState as ApiAppState};
use hostd::routes::auth::auth_routes;
use hostd::websocket_manager::WebSocketManager;
use hostd::gpu_manager::GpuManager;
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
    // Load Guardian configuration
    let guardian_config = GuardianConfig::load()
        .map_err(|e| AppError::ConfigurationError {
            message: format!("Failed to load configuration: {}", e),
            config_key: "guardian_config".to_string(),
            expected_type: "GuardianConfig".to_string(),
        })?;
    
    // Validate configuration
    guardian_config.validate()
        .map_err(|e| AppError::ConfigurationError {
            message: format!("Configuration validation failed: {}", e),
            config_key: "guardian_config".to_string(),
            expected_type: "GuardianConfig".to_string(),
        })?;

    // Initialize comprehensive logging system
    let log_config = LogConfig {
        level: guardian_config.log_level.clone(),
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
    tracing::info!("Configuration loaded: {:?}", guardian_config);

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

    // Initialize resource monitor
    let resource_monitor_config = hostd::core::resource_monitor::ResourceMonitorConfig::default();
    let resource_monitor = Arc::new(ResourceMonitor::new(resource_monitor_config, Arc::new(guardian_config.clone())));
    
    // Start resource monitoring in background
    let resource_monitor_clone = resource_monitor.clone();
    tokio::spawn(async move {
        if let Err(e) = resource_monitor_clone.start().await {
            tracing::error!("Resource monitor error: {}", e);
        }
    });

    tracing::info!("Resource monitor initialized");

    // Initialize GPU manager
    let gpu_manager = Arc::new(tokio::sync::Mutex::new(GpuManager::new(guardian_config.clone()).await.map_err(|e| AppError::InternalError { message: e, component: "gpu_manager".to_string(), details: None })?));
    tracing::info!("GPU manager initialized");

    // Initialize performance telemetry
    let performance_telemetry = Arc::new(hostd::performance_telemetry::PerformanceTelemetry::new(
        std::time::Duration::from_secs(30) // Collect metrics every 30 seconds
    ));
    tracing::info!("Performance telemetry initialized");

    // Start performance telemetry collection
    {
        let performance_telemetry_clone = performance_telemetry.clone();
        let servers_path = std::path::PathBuf::from("servers");
        tokio::spawn(async move {
            if let Err(e) = performance_telemetry_clone.start_collection(&servers_path).await {
                tracing::error!("Failed to start performance telemetry collection: {}", e);
            }
        });
    }
    
    // Start periodic GPU metrics logging
    {
        let gpu_manager_clone = gpu_manager.clone();
        let gpu_manager_guard = gpu_manager_clone.lock().await;
        gpu_manager_guard.start_metrics_logging().await;
    }
    tracing::info!("GPU metrics logging started");

    // Create the database manager first
    let database = hostd::database::DatabaseManager::new("guardian.db").await?;

    // Initialize monitoring manager
    let monitoring_config = hostd::core::config::MonitoringConfig {
        enable_metrics: true,
        metrics_port: 9090,
        log_level: "info".to_string(),
        log_file: std::path::PathBuf::from("guardian.log"),
        enable_health_checks: true,
    };
    let monitoring_manager = Arc::new(MonitoringManager::new(&monitoring_config)?);

    // Initialize WebSocket manager
    let websocket_manager = Arc::new(hostd::websocket_manager::WebSocketManager::new());

    // Initialize crash watchdog
    let watchdog_config = WatchdogConfig::default();
    let process_manager = Arc::new(hostd::core::process_manager::ProcessManager::new(websocket_manager.clone()));
    let crash_watchdog = Arc::new(CrashWatchdog::new(
        watchdog_config,
        process_manager,
        monitoring_manager,
        Arc::new(database.clone()),
    ));

    // Start crash watchdog in background
    let crash_watchdog_clone = crash_watchdog.clone();
    tokio::spawn(async move {
        if let Err(e) = crash_watchdog_clone.start().await {
            tracing::error!("Crash watchdog error: {}", e);
        }
    });

    tracing::info!("Crash watchdog initialized");

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
    let app_state = Arc::new(AppState::new(config, auth_manager.clone(), resource_monitor.clone(), crash_watchdog.clone()).await?);

    // Start the application
    app_state.start().await?;
    
    // Create MinecraftConfig for FileManager
    let minecraft_config = hostd::core::config::MinecraftConfig {
        server_jar_directory: std::path::PathBuf::from("data/server_jars"),
        world_directory: std::path::PathBuf::from("data/worlds"),
        mods_directory: std::path::PathBuf::from("data/mods"),
        config_directory: std::path::PathBuf::from("data/configs"),
        logs_directory: std::path::PathBuf::from("data/logs"),
        backups_directory: std::path::PathBuf::from("data/backups"),
        java_executable: std::path::PathBuf::from("java"),
        default_memory: 2048,
        default_max_players: 20,
        default_port: 25565,
    };

    // Create FileManager
    let file_manager = Arc::new(hostd::core::file_manager::FileManager::new(&minecraft_config).await?);

    // Create test harness
    let test_harness = Arc::new(hostd::core::test_harness::TestHarness::new(
        resource_monitor.clone(),
        crash_watchdog.clone(),
        Arc::new(hostd::core::scheduler::TaskScheduler::new(
            hostd::core::scheduler::SchedulerConfig::default(),
            Arc::new(hostd::core::server_manager::ServerManager::new(
                Arc::new(database.clone()),
                file_manager.clone(),
                Arc::new(hostd::core::process_manager::ProcessManager::new(websocket_manager.clone())),
            )),
        )),
        Arc::new(hostd::backup_manager::BackupManager::new(
            std::path::PathBuf::from("data/backups"),
            std::path::PathBuf::from("data/servers")
        )),
        Arc::new(database.clone()),
    ));
    
    // Create the API app state for the comprehensive router
    let api_websocket_manager = Arc::new(WebSocketManager::new());
    let process_manager = Arc::new(hostd::core::process_manager::ProcessManager::new(api_websocket_manager.clone()));
    let api_app_state = ApiAppState {
        websocket_manager: api_websocket_manager,
        minecraft_manager: hostd::minecraft::MinecraftManager::new(database.clone()),
        database: Arc::new(database.clone()),
        mod_manager: hostd::mod_manager::ModManager::new(std::path::PathBuf::from("mods")),
        resource_monitor: resource_monitor.clone(),
        crash_watchdog: crash_watchdog.clone(),
        test_harness: test_harness.clone(),
        gpu_manager: gpu_manager.clone(),
        performance_telemetry: performance_telemetry.clone(),
        sse_sender: None,
        process_manager: process_manager.clone(),
        server_manager: Arc::new(hostd::core::server_manager::ServerManager::new(
            Arc::new(database.clone()),
            Arc::new(hostd::core::file_manager::FileManager::new(&hostd::core::config::MinecraftConfig {
                server_jar_directory: std::path::PathBuf::from("./servers"),
                world_directory: std::path::PathBuf::from("./worlds"),
                mods_directory: std::path::PathBuf::from("./mods"),
                config_directory: std::path::PathBuf::from("./configs"),
                logs_directory: std::path::PathBuf::from("./logs"),
                backups_directory: std::path::PathBuf::from("./backups"),
                java_executable: std::path::PathBuf::from("java"),
                default_memory: 2048,
                default_max_players: 20,
                default_port: 25565,
            }).await.expect("Failed to create file manager")),
            process_manager.clone(),
        )),
        secret_storage: Arc::new(hostd::security::secret_storage::SecretStorage::new()),
        rate_limiter: Arc::new(hostd::security::rate_limiting::RateLimiter::new(hostd::security::rate_limiting::RateLimitConfig::default())),
    };
    
    // Create the main router with auth routes
    let auth_router = auth_routes().with_state(app_state.clone());
    let api_router = create_api_router(api_app_state.clone());
    
    let app = Router::new()
        .route("/", get(|| async { "Guardian Server Manager API" }))
        .merge(api_router)
        .nest("/api/auth", auth_router)
        .route("/ws", get(handle_websocket).with_state(api_app_state.websocket_manager.clone()))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        );

    // Get the server address
    let addr = guardian_config.server_address();
    
    tracing::info!("Guardian Server Manager listening on {}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(&addr).await
        .map_err(|e| AppError::NetworkError {
            message: format!("Failed to bind to address: {}", e),
            endpoint: addr.clone(),
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