use hostd::{
    core::{
        auth::{AuthManager, UserRole, LoginRequest, RegisterRequest},
        error_handler::AppError,
        monitoring::MonitoringManager,
        config::MonitoringConfig,
    },
    database::DatabaseManager,
    websocket_manager::WebSocketManager,
    backup_manager::{BackupManager, CreateBackupRequest, BackupType, CompressionType},
    rcon::RconClient,
    security::{
        validation::InputValidator,
        rate_limiting::RateLimiter,
    },
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use tower::ServiceExt;
use std::time::Duration;
use std::collections::HashMap;

#[tokio::test]
async fn test_auth_flow_integration() {
    // Initialize auth manager
    let auth_manager = AuthManager::new("test-secret".to_string());
    auth_manager.initialize().await.expect("Failed to initialize auth manager");
    
    // Test user registration
    let register_request = RegisterRequest {
        username: "integration_user".to_string(),
        email: "integration@example.com".to_string(),
        password: "password123".to_string(),
        role: Some(UserRole::User),
    };
    
    let register_result = auth_manager.register(register_request).await;
    assert!(register_result.is_ok());
    
    // Test login with registered user
    let login_request = LoginRequest {
        username: "integration_user".to_string(),
        password: "password123".to_string(),
    };
    
    let login_result = auth_manager.login(login_request).await;
    assert!(login_result.is_ok());
    
    let login_response = login_result.unwrap();
    assert_eq!(login_response.user.username, "integration_user");
    assert!(!login_response.token.is_empty());
    
    // Test token validation
    let user = auth_manager.validate_token(&login_response.token).await;
    assert!(user.is_ok());
    assert_eq!(user.unwrap().username, "integration_user");
    
    // Test logout
    let logout_result = auth_manager.logout(&login_response.token).await;
    assert!(logout_result.is_ok());
    
    // Test token validation after logout
    let user_after_logout = auth_manager.validate_token(&login_response.token).await;
    assert!(user_after_logout.is_err());
}

#[tokio::test]
async fn test_database_operations_integration() {
    // Initialize database
    let db_manager = DatabaseManager::new(":memory:").await.unwrap();
    db_manager.initialize().await.unwrap();
    
    // Create server config
    let server_config = hostd::database::ServerConfig {
        id: "integration-test-server".to_string(),
        name: "Integration Test Server".to_string(),
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
    
    // Test server config creation
    let create_result = db_manager.create_server_config(server_config.clone()).await;
    assert!(create_result.is_ok());
    
    // Test server config retrieval
    let get_result = db_manager.get_server_config("integration-test-server").await;
    assert!(get_result.is_ok());
    let retrieved_config = get_result.unwrap();
    assert_eq!(retrieved_config.name, "Integration Test Server");
    
    // Test server config update
    let mut updated_config = retrieved_config;
    updated_config.name = "Updated Integration Test Server".to_string();
    let update_result = db_manager.update_server_config(updated_config).await;
    assert!(update_result.is_ok());
    
    // Test server config deletion
    let delete_result = db_manager.delete_server_config("integration-test-server").await;
    assert!(delete_result.is_ok());
    
    // Test server config retrieval after deletion
    let get_after_delete = db_manager.get_server_config("integration-test-server").await;
    assert!(get_after_delete.is_err());
}

#[tokio::test]
async fn test_websocket_communication_integration() {
    let ws_manager = WebSocketManager::new();
    
    // Test initial state
    assert_eq!(ws_manager.get_connection_count().await, 0);
    
    // Test server-specific connection count
    assert_eq!(ws_manager.server_connection_count("test_server").await, 0);
    
    // Test message broadcasting
    let message = hostd::websocket_manager::WebSocketMessage::ConsoleMessage {
        server_id: "test_server".to_string(),
        timestamp: chrono::Utc::now(),
        level: "info".to_string(),
        message: "Test message".to_string(),
    };
    
    let broadcast_result = ws_manager.broadcast(message).await;
    assert!(broadcast_result.is_ok());
    
    // Test server-specific broadcasting
    let server_message = hostd::websocket_manager::WebSocketMessage::MetricsUpdate {
        server_id: "test_server".to_string(),
        timestamp: chrono::Utc::now(),
        tps: 20.0,
        tick_p95_ms: 45.0,
        heap_mb: 1024.0,
        players_online: 5,
        memory_usage_mb: 512.0,
        cpu_usage_percent: 50.0,
    };
    
    let server_broadcast_result = ws_manager.broadcast_to_server("test_server", server_message).await;
    assert!(server_broadcast_result.is_ok());
}

#[tokio::test]
async fn test_backup_operations_integration() {
    let backup_manager = BackupManager::new();
    
    // Test backup creation
    let create_request = CreateBackupRequest {
        name: "Integration Test Backup".to_string(),
        description: Some("Integration test backup description".to_string()),
        backup_type: BackupType::Full,
        compression: CompressionType::Zip,
        includes: vec!["world".to_string(), "server.properties".to_string()],
        metadata: HashMap::new(),
    };
    
    let create_result = backup_manager.create_backup("test_server", create_request).await;
    assert!(create_result.is_ok());
    
    let backup_info = create_result.unwrap();
    assert_eq!(backup_info.name, "Integration Test Backup");
    assert_eq!(backup_info.server_id, "test_server");
    assert_eq!(backup_info.backup_type, BackupType::Full);
    assert_eq!(backup_info.compression, CompressionType::Zip);
    
    // Test backup listing
    let list_result = backup_manager.list_backups("test_server").await;
    assert!(list_result.is_ok());
    let backups = list_result.unwrap();
    assert!(!backups.is_empty());
    assert_eq!(backups[0].name, "Integration Test Backup");
    
    // Test backup deletion
    let delete_result = backup_manager.delete_backup("test_server", &backup_info.id).await;
    assert!(delete_result.is_ok());
    
    // Test backup listing after deletion
    let list_after_delete = backup_manager.list_backups("test_server").await;
    assert!(list_after_delete.is_ok());
    let backups_after_delete = list_after_delete.unwrap();
    assert!(backups_after_delete.is_empty());
}

#[tokio::test]
async fn test_rcon_operations_integration() {
    // Test RCON client creation
    let rcon_client = RconClient::new("127.0.0.1", 25575, "password");
    
    // Test command execution (will fail without actual server, but tests structure)
    let execute_result = rcon_client.execute_command("list");
    assert!(execute_result.is_err()); // Expected to fail without actual server
    
    // Test player list parsing
    let test_response = "There are 3 of a max of 20 players online: player1, player2, player3";
    let players_result = rcon_client.parse_player_list(test_response);
    assert!(players_result.is_ok());
    let players = players_result.unwrap();
    assert_eq!(players.len(), 3);
    assert_eq!(players[0].name, "player1");
    assert_eq!(players[1].name, "player2");
    assert_eq!(players[2].name, "player3");
    
    // Test server info parsing
    let server_info_result = rcon_client.get_server_info();
    assert!(server_info_result.is_ok());
    let server_info = server_info_result.unwrap();
    assert_eq!(server_info.player_count, 3);
    assert_eq!(server_info.max_players, 20);
}

#[tokio::test]
async fn test_monitoring_integration() {
    let config = MonitoringConfig {
        enabled: true,
        metrics_interval: Duration::from_secs(10),
        health_check_interval: Duration::from_secs(30),
        alert_retention_days: 30,
    };
    
    let monitoring_manager = MonitoringManager::new(&config).unwrap();
    
    // Test monitoring start
    let start_result = monitoring_manager.start().await;
    assert!(start_result.is_ok());
    
    // Wait a bit for metrics collection
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Test health status
    let health_status = monitoring_manager.get_health_status().await;
    assert_eq!(health_status.status, "healthy");
    
    // Test system metrics
    let system_metrics = monitoring_manager.get_system_metrics().await;
    assert!(system_metrics.cpu_usage >= 0.0);
    assert!(system_metrics.memory_usage >= 0.0);
    assert!(system_metrics.disk_usage >= 0.0);
    
    // Test alert creation
    let alert_id = monitoring_manager.create_alert(
        None,
        hostd::core::monitoring::AlertLevel::Warning,
        "Integration Test Alert".to_string(),
        "This is an integration test alert".to_string(),
    ).await.unwrap();
    
    // Test alert retrieval
    let alerts = monitoring_manager.get_alerts().await;
    assert!(!alerts.is_empty());
    assert!(alerts.iter().any(|a| a.id == alert_id));
    
    // Test alert resolution
    let resolve_result = monitoring_manager.resolve_alert(alert_id).await;
    assert!(resolve_result.is_ok());
    
    // Test monitoring stop
    let stop_result = monitoring_manager.stop().await;
    assert!(stop_result.is_ok());
}

#[tokio::test]
async fn test_rate_limiting_integration() {
    let rate_limiter = RateLimiter::new(3, 60); // 3 requests per minute
    
    // Test normal usage within limits
    for i in 0..3 {
        let result = rate_limiter.check_rate_limit("test_user").await;
        assert!(result.is_ok(), "Request {} should be allowed", i + 1);
    }
    
    // Test rate limit exceeded
    let result = rate_limiter.check_rate_limit("test_user").await;
    assert!(result.is_err());
    
    // Test different user (should not be affected)
    let result = rate_limiter.check_rate_limit("different_user").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_input_validation_integration() {
    // Test comprehensive input validation
    let test_cases = vec![
        ("server_name", "My Server", true),
        ("server_name", "", false),
        ("server_name", &"a".repeat(51), false),
        ("server_name", "Server<script>", false),
        ("minecraft_version", "1.21.1", true),
        ("minecraft_version", "1.21", true),
        ("minecraft_version", "invalid", false),
        ("port", "25565", true),
        ("port", "1023", false),
        ("port", "65536", false),
        ("memory", "1024", true),
        ("memory", "511", false),
        ("memory", "32769", false),
        ("max_players", "20", true),
        ("max_players", "0", false),
        ("max_players", "1001", false),
    ];
    
    for (field, value, should_pass) in test_cases {
        let result = match field {
            "server_name" => InputValidator::validate_server_name(value),
            "minecraft_version" => InputValidator::validate_minecraft_version(value),
            "port" => InputValidator::validate_port(value.parse().unwrap_or(0)),
            "memory" => InputValidator::validate_memory(value.parse().unwrap_or(0)),
            "max_players" => InputValidator::validate_max_players(value.parse().unwrap_or(0)),
            _ => Ok(()),
        };
        
        if should_pass {
            assert!(result.is_ok(), "Validation should pass for {}: {}", field, value);
        } else {
            assert!(result.is_err(), "Validation should fail for {}: {}", field, value);
        }
    }
}

#[tokio::test]
async fn test_security_integration() {
    // Test SQL injection prevention
    let safe_queries = vec![
        "SELECT * FROM users",
        "SELECT name, email FROM users WHERE id = 1",
        "UPDATE users SET name = 'test' WHERE id = 1",
    ];
    
    for query in safe_queries {
        assert!(hostd::security::validation::SQLInjectionPrevention::is_safe_query(query));
    }
    
    let dangerous_queries = vec![
        "DROP TABLE users",
        "SELECT * FROM users; DROP TABLE users",
        "INSERT INTO users VALUES ('admin', 'password')",
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
        "config/server.properties",
    ];
    
    for path in safe_paths {
        assert!(hostd::security::validation::PathTraversalPrevention::is_safe_path(path));
    }
    
    let dangerous_paths = vec![
        "../etc/passwd",
        "../../etc/passwd",
        "~/secret",
        "/etc/passwd",
    ];
    
    for path in dangerous_paths {
        assert!(!hostd::security::validation::PathTraversalPrevention::is_safe_path(path));
    }
    
    // Test command injection prevention
    let safe_commands = vec![
        "list",
        "say hello",
        "tp player1 player2",
    ];
    
    for command in safe_commands {
        assert!(hostd::security::validation::CommandInjectionPrevention::is_safe_command(command));
    }
    
    let dangerous_commands = vec![
        "list; rm -rf /",
        "say hello & rm -rf /",
        "list | cat /etc/passwd",
        "say `rm -rf /`",
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