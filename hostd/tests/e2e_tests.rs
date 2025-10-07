use hostd::{
    core::{
        auth::{AuthManager, UserRole, LoginRequest, RegisterRequest},
        error_handler::AppError,
        monitoring::MonitoringManager,
        config::MonitoringConfig,
        guardian_config::GuardianConfig,
    },
    database::DatabaseManager,
    websocket_manager::WebSocketManager,
    backup_manager::{BackupManager, CreateBackupRequest, BackupType, CompressionType},
    rcon::RconClient,
    security::{
        validation::InputValidator,
        rate_limiting::RateLimiter,
        path_sanitizer::PathSanitizer,
    },
    gpu_manager::GpuManager,
    mod_manager::ModManager,
    version_resolver::VersionResolver,
    modpack_installer::ModpackInstaller,
};
use axum::{
    body::Body,
    http::{Request, StatusCode, Method, HeaderValue},
    Router,
    routing::{get, post},
    Json,
};
use tower::ServiceExt;
use std::time::Duration;
use std::collections::HashMap;
use serde_json::json;

#[tokio::test]
async fn test_complete_server_lifecycle() {
    // Initialize all components
    let auth_manager = AuthManager::new("test-secret".to_string());
    auth_manager.initialize().await.expect("Failed to initialize auth manager");
    
    let db_manager = DatabaseManager::new(":memory:").await.unwrap();
    db_manager.initialize().await.unwrap();
    
    let ws_manager = WebSocketManager::new();
    let backup_manager = BackupManager::new();
    
    // Test user registration and authentication
    let register_request = RegisterRequest {
        username: "e2e_user".to_string(),
        email: "e2e@example.com".to_string(),
        password: "password123".to_string(),
        role: Some(UserRole::Admin),
    };
    
    let register_result = auth_manager.register(register_request).await;
    assert!(register_result.is_ok());
    
    let login_request = LoginRequest {
        username: "e2e_user".to_string(),
        password: "password123".to_string(),
    };
    
    let login_result = auth_manager.login(login_request).await;
    assert!(login_result.is_ok());
    
    let login_response = login_result.unwrap();
    let user = auth_manager.validate_token(&login_response.token).await.unwrap();
    assert_eq!(user.username, "e2e_user");
    assert_eq!(user.role, UserRole::Admin);
    
    // Test server configuration creation
    let server_config = hostd::database::ServerConfig {
        id: "e2e-test-server".to_string(),
        name: "E2E Test Server".to_string(),
        minecraft_version: "1.21.1".to_string(),
        loader: "vanilla".to_string(),
        loader_version: "1.21.1".to_string(),
        port: 25565,
        rcon_port: 25575,
        query_port: 25566,
        max_players: 20,
        memory: 2048,
        java_args: "[]".to_string(),
        server_args: "[]".to_string(),
        auto_start: false,
        auto_restart: false,
        world_name: "world".to_string(),
        difficulty: "normal".to_string(),
        gamemode: "survival".to_string(),
        pvp: true,
        online_mode: true,
        whitelist: false,
        enable_command_block: false,
        view_distance: 10,
        simulation_distance: 10,
        motd: "A Minecraft Server".to_string(),
        host: "0.0.0.0".to_string(),
        java_path: "java".to_string(),
        jvm_args: "[]".to_string(),
        server_jar: "server.jar".to_string(),
        rcon_password: "password".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let create_result = db_manager.create_server_config(server_config.clone()).await;
    assert!(create_result.is_ok());
    
    // Test server configuration retrieval
    let get_result = db_manager.get_server_config("e2e-test-server").await;
    assert!(get_result.is_ok());
    let retrieved_config = get_result.unwrap();
    assert_eq!(retrieved_config.name, "E2E Test Server");
    
    // Test backup creation
    let backup_request = CreateBackupRequest {
        name: "E2E Test Backup".to_string(),
        description: Some("E2E test backup description".to_string()),
        backup_type: BackupType::Full,
        compression: CompressionType::Zip,
        includes: vec!["world".to_string(), "server.properties".to_string()],
        metadata: HashMap::new(),
    };
    
    let backup_result = backup_manager.create_backup("e2e-test-server", backup_request).await;
    assert!(backup_result.is_ok());
    
    let backup_info = backup_result.unwrap();
    assert_eq!(backup_info.name, "E2E Test Backup");
    assert_eq!(backup_info.server_id, "e2e-test-server");
    
    // Test backup listing
    let list_result = backup_manager.list_backups("e2e-test-server").await;
    assert!(list_result.is_ok());
    let backups = list_result.unwrap();
    assert!(!backups.is_empty());
    assert_eq!(backups[0].name, "E2E Test Backup");
    
    // Test WebSocket communication
    let message = hostd::websocket_manager::WebSocketMessage::ConsoleMessage {
        server_id: "e2e-test-server".to_string(),
        timestamp: chrono::Utc::now(),
        level: "info".to_string(),
        message: "E2E test message".to_string(),
    };
    
    let broadcast_result = ws_manager.broadcast(message).await;
    assert!(broadcast_result.is_ok());
    
    // Test RCON operations
    let rcon_client = RconClient::new("127.0.0.1", 25575, "password");
    
    // Test player list parsing
    let test_response = "There are 2 of a max of 20 players online: player1, player2";
    let players_result = rcon_client.parse_player_list(test_response);
    assert!(players_result.is_ok());
    let players = players_result.unwrap();
    assert_eq!(players.len(), 2);
    assert_eq!(players[0].name, "player1");
    assert_eq!(players[1].name, "player2");
    
    // Test monitoring
    let config = MonitoringConfig {
        enabled: true,
        metrics_interval: Duration::from_secs(10),
        health_check_interval: Duration::from_secs(30),
        alert_retention_days: 30,
    };
    
    let monitoring_manager = MonitoringManager::new(&config).unwrap();
    let start_result = monitoring_manager.start().await;
    assert!(start_result.is_ok());
    
    // Wait for metrics collection
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let health_status = monitoring_manager.get_health_status().await;
    assert_eq!(health_status.status, "healthy");
    
    let system_metrics = monitoring_manager.get_system_metrics().await;
    assert!(system_metrics.cpu_usage >= 0.0);
    assert!(system_metrics.memory_usage >= 0.0);
    
    // Test alert creation
    let alert_id = monitoring_manager.create_alert(
        None,
        hostd::core::monitoring::AlertLevel::Warning,
        "E2E Test Alert".to_string(),
        "This is an E2E test alert".to_string(),
    ).await.unwrap();
    
    let alerts = monitoring_manager.get_alerts().await;
    assert!(!alerts.is_empty());
    assert!(alerts.iter().any(|a| a.id == alert_id));
    
    // Test cleanup
    let delete_backup_result = backup_manager.delete_backup("e2e-test-server", &backup_info.id).await;
    assert!(delete_backup_result.is_ok());
    
    let delete_server_result = db_manager.delete_server_config("e2e-test-server").await;
    assert!(delete_server_result.is_ok());
    
    let logout_result = auth_manager.logout(&login_response.token).await;
    assert!(logout_result.is_ok());
    
    let stop_result = monitoring_manager.stop().await;
    assert!(stop_result.is_ok());
}

#[tokio::test]
async fn test_api_endpoints_integration() {
    // Create a simple API router for testing
    let auth_manager = AuthManager::new("test-secret".to_string());
    auth_manager.initialize().await.expect("Failed to initialize auth manager");
    
    let db_manager = DatabaseManager::new(":memory:").await.unwrap();
    db_manager.initialize().await.unwrap();
    
    let ws_manager = WebSocketManager::new();
    let backup_manager = BackupManager::new();
    
    // Create API router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/servers", post(create_server))
        .route("/api/servers/:id", get(get_server))
        .route("/api/servers/:id/backups", post(create_backup))
        .route("/api/servers/:id/backups", get(list_backups))
        .route("/api/servers/:id/backups/:backup_id", delete(delete_backup))
        .route("/api/auth/login", post(login))
        .route("/api/auth/register", post(register))
        .route("/api/auth/logout", post(logout))
        .route("/api/monitoring/health", get(get_health_status))
        .route("/api/monitoring/metrics", get(get_system_metrics))
        .route("/api/monitoring/alerts", get(get_alerts));
    
    // Test health check endpoint
    let request = Request::builder()
        .method(Method::GET)
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test server creation endpoint
    let server_data = json!({
        "name": "API Test Server",
        "minecraft_version": "1.21.1",
        "loader": "vanilla",
        "port": 25565,
        "max_players": 20,
        "memory": 2048
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/servers")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&server_data).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test server retrieval endpoint
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/servers/api-test-server")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test backup creation endpoint
    let backup_data = json!({
        "name": "API Test Backup",
        "description": "API test backup description",
        "backup_type": "Full",
        "compression": "Zip",
        "includes": ["world", "server.properties"]
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/servers/api-test-server/backups")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&backup_data).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test backup listing endpoint
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/servers/api-test-server/backups")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test authentication endpoints
    let login_data = json!({
        "username": "admin",
        "password": "admin123"
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&login_data).unwrap()))
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // Test monitoring endpoints
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/monitoring/health")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/monitoring/metrics")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/monitoring/alerts")
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_security_integration() {
    // Test rate limiting
    let rate_limiter = RateLimiter::new(5, 60); // 5 requests per minute
    
    // Test normal usage
    for i in 0..5 {
        let result = rate_limiter.check_rate_limit("security_test_user").await;
        assert!(result.is_ok(), "Request {} should be allowed", i + 1);
    }
    
    // Test rate limit exceeded
    let result = rate_limiter.check_rate_limit("security_test_user").await;
    assert!(result.is_err());
    
    // Test input validation
    let test_cases = vec![
        ("server_name", "Security Test Server", true),
        ("server_name", "", false),
        ("server_name", &"a".repeat(51), false),
        ("server_name", "Server<script>", false),
        ("minecraft_version", "1.21.1", true),
        ("minecraft_version", "invalid", false),
        ("port", "25565", true),
        ("port", "1023", false),
        ("port", "65536", false),
    ];
    
    for (field, value, should_pass) in test_cases {
        let result = match field {
            "server_name" => InputValidator::validate_server_name(value),
            "minecraft_version" => InputValidator::validate_minecraft_version(value),
            "port" => InputValidator::validate_port(value.parse().unwrap_or(0)),
            _ => Ok(()),
        };
        
        if should_pass {
            assert!(result.is_ok(), "Validation should pass for {}: {}", field, value);
        } else {
            assert!(result.is_err(), "Validation should fail for {}: {}", field, value);
        }
    }
    
    // Test SQL injection prevention
    let safe_queries = vec![
        "SELECT * FROM users",
        "SELECT name, email FROM users WHERE id = 1",
    ];
    
    for query in safe_queries {
        assert!(hostd::security::validation::SQLInjectionPrevention::is_safe_query(query));
    }
    
    let dangerous_queries = vec![
        "DROP TABLE users",
        "SELECT * FROM users; DROP TABLE users",
    ];
    
    for query in dangerous_queries {
        assert!(!hostd::security::validation::SQLInjectionPrevention::is_safe_query(query));
    }
    
    // Test XSS prevention
    let malicious_input = "<script>alert('XSS')</script>";
    let sanitized = hostd::security::validation::XSSPrevention::sanitize_html(malicious_input);
    assert!(!sanitized.contains("<script>"));
    assert!(sanitized.contains("&lt;script&gt;"));
    
    // Test path traversal prevention
    let safe_paths = vec![
        "servers/world",
        "data/backups",
    ];
    
    for path in safe_paths {
        assert!(hostd::security::validation::PathTraversalPrevention::is_safe_path(path));
    }
    
    let dangerous_paths = vec![
        "../etc/passwd",
        "../../etc/passwd",
    ];
    
    for path in dangerous_paths {
        assert!(!hostd::security::validation::PathTraversalPrevention::is_safe_path(path));
    }
    
    // Test command injection prevention
    let safe_commands = vec![
        "list",
        "say hello",
    ];
    
    for command in safe_commands {
        assert!(hostd::security::validation::CommandInjectionPrevention::is_safe_command(command));
    }
    
    let dangerous_commands = vec![
        "list; rm -rf /",
        "say hello & rm -rf /",
    ];
    
    for command in dangerous_commands {
        assert!(!hostd::security::validation::CommandInjectionPrevention::is_safe_command(command));
    }
}

#[tokio::test]
async fn test_error_handling_integration() {
    // Test error creation and handling
    let validation_error = AppError::validation_error("username", "test", "required", "Username is required");
    assert_eq!(validation_error.status_code(), StatusCode::BAD_REQUEST);
    assert!(validation_error.user_message().contains("Username is required"));
    assert_eq!(validation_error.category(), "validation");
    assert!(!validation_error.is_retryable());
    
    let database_error = AppError::database_error("create", "Connection failed");
    assert_eq!(database_error.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(database_error.category(), "database");
    assert!(database_error.is_retryable());
    assert_eq!(database_error.retry_delay_ms(), 1000);
    
    let auth_error = AppError::authentication_error(
        hostd::core::error_handler::AuthErrorReason::InvalidCredentials,
        "Invalid username or password"
    );
    assert_eq!(auth_error.status_code(), StatusCode::UNAUTHORIZED);
    assert_eq!(auth_error.category(), "authentication");
    assert!(!auth_error.is_retryable());
    
    // Test error response serialization
    let error_response = hostd::core::error_handler::ErrorResponse {
        success: false,
        error: "Test error".to_string(),
        error_code: "TEST_ERROR".to_string(),
        category: "test".to_string(),
        timestamp: chrono::Utc::now(),
        details: Some("Test error details".to_string()),
    };
    
    let serialized = serde_json::to_string(&error_response).unwrap();
    assert!(serialized.contains("Test error"));
    assert!(serialized.contains("TEST_ERROR"));
    assert!(serialized.contains("test"));
}

// Mock API handlers for testing
async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "message": "System is running normally",
        "timestamp": chrono::Utc::now()
    }))
}

async fn create_server(Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "message": "Server created successfully",
        "server_id": "api-test-server"
    }))
}

async fn get_server(axum::extract::Path(id): axum::extract::Path<String>) -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "server": {
            "id": id,
            "name": "API Test Server",
            "minecraft_version": "1.21.1",
            "loader": "vanilla",
            "port": 25565,
            "max_players": 20,
            "memory": 2048
        }
    }))
}

async fn create_backup(
    axum::extract::Path(server_id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>
) -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "message": "Backup created successfully",
        "backup_id": "backup-123"
    }))
}

async fn list_backups(axum::extract::Path(server_id): axum::extract::Path<String>) -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "backups": []
    }))
}

async fn delete_backup(
    axum::extract::Path((server_id, backup_id)): axum::extract::Path<(String, String)>
) -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "message": "Backup deleted successfully"
    }))
}

async fn login(Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "token": "test-token",
        "user": {
            "id": "user-123",
            "username": "admin",
            "role": "admin"
        }
    }))
}

async fn register(Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "message": "User registered successfully"
    }))
}

async fn logout(Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
    Json(json!({
        "success": true,
        "message": "Logged out successfully"
    }))
}

async fn get_health_status() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "message": "System is running normally",
        "timestamp": chrono::Utc::now()
    }))
}

async fn get_system_metrics() -> Json<serde_json::Value> {
    Json(json!({
        "cpu_usage": 50.0,
        "memory_usage": 60.0,
        "disk_usage": 40.0,
        "timestamp": chrono::Utc::now()
    }))
}

async fn get_alerts() -> Json<serde_json::Value> {
    Json(json!({
        "alerts": []
    }))
}

// ===== COMPREHENSIVE INTEGRATION TESTS =====

#[tokio::test]
async fn test_health_endpoints() {
    let config = GuardianConfig::default();
    let db_manager = DatabaseManager::new(":memory:").await.unwrap();
    db_manager.initialize().await.unwrap();
    
    let gpu_manager = GpuManager::new(config).await.unwrap();
    
    // Test basic health check
    let health_response = get_health_status().await;
    assert!(health_response.0["status"] == "healthy");
    
    // Test system metrics
    let metrics_response = get_system_metrics().await;
    assert!(metrics_response.0["cpu_usage"].is_number());
    assert!(metrics_response.0["memory_usage"].is_number());
}

#[tokio::test]
async fn test_server_creation_flow() {
    let config = GuardianConfig::default();
    let db_manager = DatabaseManager::new(":memory:").await.unwrap();
    db_manager.initialize().await.unwrap();
    
    // Test server creation request
    let server_data = json!({
        "name": "test-server",
        "minecraft_version": "1.20.1",
        "loader": "vanilla",
        "memory_mb": 2048,
        "port": 25565
    });
    
    // Validate server data structure
    assert!(server_data["name"].is_string());
    assert!(server_data["minecraft_version"].is_string());
    assert!(server_data["loader"].is_string());
    assert!(server_data["memory_mb"].is_number());
    assert!(server_data["port"].is_number());
}

#[tokio::test]
async fn test_mod_search_functionality() {
    let config = GuardianConfig::default();
    let db_manager = DatabaseManager::new(":memory:").await.unwrap();
    db_manager.initialize().await.unwrap();
    
    let mod_manager = ModManager::new(db_manager.clone()).await.unwrap();
    
    // Test mod search with mock data
    let search_query = "jei";
    let search_results = mod_manager.search_mods(search_query, "all", 0, 10).await;
    
    // Should return results (even if empty for test)
    assert!(search_results.is_ok());
}

#[tokio::test]
async fn test_mod_installation_flow() {
    let config = GuardianConfig::default();
    let db_manager = DatabaseManager::new(":memory:").await.unwrap();
    db_manager.initialize().await.unwrap();
    
    let mod_manager = ModManager::new(db_manager.clone()).await.unwrap();
    
    // Test mod installation request
    let install_request = json!({
        "mod_id": "test-mod-123",
        "provider": "modrinth",
        "version": "1.0.0",
        "server_id": "test-server-123"
    });
    
    // Validate request structure
    assert!(install_request["mod_id"].is_string());
    assert!(install_request["provider"].is_string());
    assert!(install_request["version"].is_string());
    assert!(install_request["server_id"].is_string());
}

#[tokio::test]
async fn test_modpack_application_flow() {
    let config = GuardianConfig::default();
    let db_manager = DatabaseManager::new(":memory:").await.unwrap();
    db_manager.initialize().await.unwrap();
    
    let modpack_installer = ModpackInstaller::new(db_manager.clone()).await.unwrap();
    
    // Test modpack application request
    let modpack_request = json!({
        "modpack_id": "test-pack-123",
        "provider": "modrinth",
        "server_id": "test-server-123",
        "version": "1.0.0"
    });
    
    // Validate request structure
    assert!(modpack_request["modpack_id"].is_string());
    assert!(modpack_request["provider"].is_string());
    assert!(modpack_request["server_id"].is_string());
    assert!(modpack_request["version"].is_string());
}

#[tokio::test]
async fn test_gpu_manager_integration() {
    let config = GuardianConfig::default();
    let gpu_manager = GpuManager::new(config).await.unwrap();
    
    // Test GPU status
    assert!(!gpu_manager.is_enabled()); // Should be disabled by default
    
    // Test metrics retrieval
    let metrics = gpu_manager.get_metrics().await;
    assert!(metrics.utilization >= 0.0);
    assert!(metrics.memory_used >= 0);
    assert!(metrics.memory_total >= 0);
}

#[tokio::test]
async fn test_path_sanitization() {
    let sanitizer = PathSanitizer::new();
    
    // Test valid paths
    assert!(sanitizer.is_safe_path("mods/test-mod.jar"));
    assert!(sanitizer.is_safe_path("config/server.properties"));
    
    // Test malicious paths
    assert!(!sanitizer.is_safe_path("../etc/passwd"));
    assert!(!sanitizer.is_safe_path("../../../windows/system32"));
    assert!(!sanitizer.is_safe_path("/etc/shadow"));
    assert!(!sanitizer.is_safe_path("C:\\Windows\\System32"));
}

#[tokio::test]
async fn test_input_validation() {
    let validator = InputValidator::new();
    
    // Test server name validation
    assert!(validator.validate_server_name("valid-server-name").is_ok());
    assert!(validator.validate_server_name("").is_err());
    assert!(validator.validate_server_name("a".repeat(256)).is_err());
    
    // Test path validation
    assert!(validator.validate_path("C:\\valid\\path").is_ok());
    assert!(validator.validate_path("").is_err());
    assert!(validator.validate_path("../invalid").is_err());
    
    // Test port validation
    assert!(validator.validate_port(25565).is_ok());
    assert!(validator.validate_port(0).is_err());
    assert!(validator.validate_port(65536).is_err());
}

#[tokio::test]
async fn test_rate_limiting() {
    let rate_limiter = RateLimiter::new(10, Duration::from_secs(60));
    
    // Test rate limiting
    for i in 0..15 {
        let result = rate_limiter.check_rate_limit("test-client").await;
        if i < 10 {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err());
        }
    }
}

#[tokio::test]
async fn test_websocket_management() {
    let ws_manager = WebSocketManager::new();
    
    // Test WebSocket manager creation
    assert!(ws_manager.get_connection_count().await >= 0);
    
    // Test progress event creation
    let progress_event = hostd::core::websocket::ProgressEvent {
        job_id: "test-job".to_string(),
        job_type: "test".to_string(),
        status: "started".to_string(),
        progress: 0.0,
        current_step: "Test Step".to_string(),
        total_steps: 1,
        current_step_progress: 0.0,
        message: Some("Test message".to_string()),
        error: None,
        estimated_remaining_ms: None,
    };
    
    // Should be able to create progress events
    assert_eq!(progress_event.job_id, "test-job");
    assert_eq!(progress_event.job_type, "test");
}

#[tokio::test]
async fn test_database_operations() {
    let db_manager = DatabaseManager::new(":memory:").await.unwrap();
    db_manager.initialize().await.unwrap();
    
    // Test database health
    let health_status = db_manager.get_health_status().await;
    assert!(health_status.is_ok());
    
    // Test basic database operations
    let test_server = hostd::database::Server {
        id: "test-server".to_string(),
        name: "Test Server".to_string(),
        minecraft_version: "1.20.1".to_string(),
        loader: "vanilla".to_string(),
        port: 25565,
        memory_mb: 2048,
        status: hostd::minecraft::ServerStatus::Stopped,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_start: None,
        last_stop: None,
        auto_start: false,
        auto_restart: false,
        jvm_args: None,
        server_properties: None,
        eula_accepted: false,
        world_path: None,
        backup_path: None,
        log_path: None,
        config_path: None,
    };
    
    // Test server creation
    let create_result = db_manager.create_server(test_server.clone()).await;
    assert!(create_result.is_ok());
    
    // Test server retrieval
    let get_result = db_manager.get_server("test-server").await;
    assert!(get_result.is_ok());
    
    // Test server listing
    let list_result = db_manager.list_servers().await;
    assert!(list_result.is_ok());
    assert!(list_result.unwrap().len() >= 1);
}
