use hostd::core::{
    auth::{AuthManager, UserRole, LoginRequest, RegisterRequest},
    error_handler::AppError,
    validation::InputValidator,
    monitoring::MonitoringManager,
};
use hostd::security::{
    validation::{SQLInjectionPrevention, XSSPrevention, PathTraversalPrevention, CommandInjectionPrevention},
    rate_limiting::RateLimiter,
};
use hostd::database::DatabaseManager;
use hostd::websocket_manager::WebSocketManager;
use hostd::backup_manager::BackupManager;
use hostd::rcon::RconClient;
use std::time::Duration;

#[tokio::test]
async fn test_auth_manager_initialization() {
    let auth_manager = AuthManager::new("test-secret".to_string());
    auth_manager.initialize().await.expect("Failed to initialize auth manager");
    
    // Test default admin user creation
    let users = auth_manager.get_all_users().await;
    assert!(!users.is_empty());
    assert!(users.iter().any(|u| u.username == "admin"));
}

#[tokio::test]
async fn test_user_registration() {
    let auth_manager = AuthManager::new("test-secret".to_string());
    auth_manager.initialize().await.expect("Failed to initialize auth manager");
    
    let register_request = RegisterRequest {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        role: Some(UserRole::User),
    };
    
    let result = auth_manager.register(register_request).await;
    assert!(result.is_ok());
    
    let login_response = result.unwrap();
    assert_eq!(login_response.user.username, "testuser");
    assert_eq!(login_response.user.email, "test@example.com");
    assert_eq!(login_response.user.role, UserRole::User);
}

#[tokio::test]
async fn test_user_login() {
    let auth_manager = AuthManager::new("test-secret".to_string());
    auth_manager.initialize().await.expect("Failed to initialize auth manager");
    
    let login_request = LoginRequest {
        username: "admin".to_string(),
        password: "admin123".to_string(),
    };
    
    let result = auth_manager.login(login_request).await;
    assert!(result.is_ok());
    
    let login_response = result.unwrap();
    assert_eq!(login_response.user.username, "admin");
    assert!(!login_response.token.is_empty());
}

#[tokio::test]
async fn test_invalid_login() {
    let auth_manager = AuthManager::new("test-secret".to_string());
    auth_manager.initialize().await.expect("Failed to initialize auth manager");
    
    let login_request = LoginRequest {
        username: "admin".to_string(),
        password: "wrongpassword".to_string(),
    };
    
    let result = auth_manager.login(login_request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_token_validation() {
    let auth_manager = AuthManager::new("test-secret".to_string());
    auth_manager.initialize().await.expect("Failed to initialize auth manager");
    
    let login_request = LoginRequest {
        username: "admin".to_string(),
        password: "admin123".to_string(),
    };
    
    let login_response = auth_manager.login(login_request).await.unwrap();
    
    let user = auth_manager.validate_token(&login_response.token).await;
    assert!(user.is_ok());
    assert_eq!(user.unwrap().username, "admin");
}

#[tokio::test]
async fn test_permission_checking() {
    let auth_manager = AuthManager::new("test-secret".to_string());
    auth_manager.initialize().await.expect("Failed to initialize auth manager");
    
    let login_request = LoginRequest {
        username: "admin".to_string(),
        password: "admin123".to_string(),
    };
    
    let login_response = auth_manager.login(login_request).await.unwrap();
    let user = auth_manager.validate_token(&login_response.token).await.unwrap();
    
    // Admin should have all permissions
    assert!(auth_manager.has_permission(user.id, &hostd::core::auth::Permission::CreateServer).await);
    assert!(auth_manager.has_permission(user.id, &hostd::core::auth::Permission::DeleteServer).await);
    assert!(auth_manager.has_permission(user.id, &hostd::core::auth::Permission::SystemSettings).await);
}

#[tokio::test]
fn test_input_validation() {
    // Test server name validation
    assert!(InputValidator::validate_server_name("My Server").is_ok());
    assert!(InputValidator::validate_server_name("").is_err());
    assert!(InputValidator::validate_server_name(&"a".repeat(51)).is_err());
    assert!(InputValidator::validate_server_name("Server<script>").is_err());
    
    // Test Minecraft version validation
    assert!(InputValidator::validate_minecraft_version("1.21.1").is_ok());
    assert!(InputValidator::validate_minecraft_version("1.21").is_ok());
    assert!(InputValidator::validate_minecraft_version("invalid").is_err());
    
    // Test port validation
    assert!(InputValidator::validate_port(25565).is_ok());
    assert!(InputValidator::validate_port(1023).is_err());
    assert!(InputValidator::validate_port(65536).is_err());
    
    // Test memory validation
    assert!(InputValidator::validate_memory(1024).is_ok());
    assert!(InputValidator::validate_memory(511).is_err());
    assert!(InputValidator::validate_memory(32769).is_err());
    
    // Test max players validation
    assert!(InputValidator::validate_max_players(20).is_ok());
    assert!(InputValidator::validate_max_players(0).is_err());
    assert!(InputValidator::validate_max_players(1001).is_err());
}

#[tokio::test]
fn test_sql_injection_prevention() {
    // Test safe queries
    assert!(SQLInjectionPrevention::is_safe_query("SELECT * FROM users"));
    assert!(SQLInjectionPrevention::is_safe_query("SELECT name, email FROM users WHERE id = 1"));
    
    // Test dangerous queries
    assert!(!SQLInjectionPrevention::is_safe_query("DROP TABLE users"));
    assert!(!SQLInjectionPrevention::is_safe_query("SELECT * FROM users; DROP TABLE users"));
    assert!(!SQLInjectionPrevention::is_safe_query("INSERT INTO users VALUES ('admin', 'password')"));
    
    // Test sanitization
    let dangerous_query = "SELECT * FROM users; DROP TABLE users";
    let sanitized = SQLInjectionPrevention::sanitize_query(dangerous_query);
    assert!(SQLInjectionPrevention::is_safe_query(&sanitized));
}

#[tokio::test]
fn test_xss_prevention() {
    let malicious_input = "<script>alert('XSS')</script>";
    let sanitized = XSSPrevention::sanitize_html(malicious_input);
    assert!(!sanitized.contains("<script>"));
    assert!(sanitized.contains("&lt;script&gt;"));
    
    let js_input = "console.log('test'); alert('XSS');";
    let sanitized_js = XSSPrevention::sanitize_js(js_input);
    assert!(sanitized_js.contains("\\'"));
}

#[tokio::test]
fn test_path_traversal_prevention() {
    // Test safe paths
    assert!(PathTraversalPrevention::is_safe_path("servers/world"));
    assert!(PathTraversalPrevention::is_safe_path("data/backups"));
    
    // Test dangerous paths
    assert!(!PathTraversalPrevention::is_safe_path("../etc/passwd"));
    assert!(!PathTraversalPrevention::is_safe_path("../../etc/passwd"));
    assert!(!PathTraversalPrevention::is_safe_path("~/secret"));
    assert!(!PathTraversalPrevention::is_safe_path("/etc/passwd"));
    
    // Test sanitization
    let dangerous_path = "../../etc/passwd";
    let sanitized = PathTraversalPrevention::sanitize_path(dangerous_path);
    assert!(PathTraversalPrevention::is_safe_path(&sanitized));
}

#[tokio::test]
fn test_command_injection_prevention() {
    // Test safe commands
    assert!(CommandInjectionPrevention::is_safe_command("list"));
    assert!(CommandInjectionPrevention::is_safe_command("say hello"));
    assert!(CommandInjectionPrevention::is_safe_command("tp player1 player2"));
    
    // Test dangerous commands
    assert!(!CommandInjectionPrevention::is_safe_command("list; rm -rf /"));
    assert!(!CommandInjectionPrevention::is_safe_command("say hello & rm -rf /"));
    assert!(!CommandInjectionPrevention::is_safe_command("list | cat /etc/passwd"));
    assert!(!CommandInjectionPrevention::is_safe_command("say `rm -rf /`"));
    
    // Test sanitization
    let dangerous_command = "list; rm -rf /";
    let sanitized = CommandInjectionPrevention::sanitize_command(dangerous_command);
    assert!(CommandInjectionPrevention::is_safe_command(&sanitized));
}

#[tokio::test]
async fn test_rate_limiting() {
    let rate_limiter = RateLimiter::new(5, 60); // 5 requests per minute
    
    // Test normal usage
    for i in 0..5 {
        let result = rate_limiter.check_rate_limit("test_user").await;
        assert!(result.is_ok(), "Request {} should be allowed", i + 1);
    }
    
    // Test rate limit exceeded
    let result = rate_limiter.check_rate_limit("test_user").await;
    assert!(result.is_err());
    
    // Test different user
    let result = rate_limiter.check_rate_limit("different_user").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_websocket_manager() {
    let ws_manager = WebSocketManager::new();
    
    // Test initial state
    assert_eq!(ws_manager.get_connection_count().await, 0);
    
    // Test server connection count
    assert_eq!(ws_manager.server_connection_count("test_server").await, 0);
}

#[tokio::test]
async fn test_backup_manager() {
    let backup_manager = BackupManager::new();
    
    // Test backup creation
    let request = hostd::backup_manager::CreateBackupRequest {
        name: "Test Backup".to_string(),
        description: Some("Test backup description".to_string()),
        backup_type: hostd::backup_manager::BackupType::Full,
        compression: hostd::backup_manager::CompressionType::Zip,
        includes: vec!["world".to_string(), "server.properties".to_string()],
        metadata: std::collections::HashMap::new(),
    };
    
    let result = backup_manager.create_backup("test_server", request).await;
    assert!(result.is_ok());
    
    let backup_info = result.unwrap();
    assert_eq!(backup_info.name, "Test Backup");
    assert_eq!(backup_info.server_id, "test_server");
}

#[tokio::test]
async fn test_rcon_client() {
    // Test RCON client creation
    let rcon_client = RconClient::new("127.0.0.1", 25575, "password");
    
    // Test command execution (this will fail in test environment, but we can test the structure)
    let result = rcon_client.execute_command("list");
    assert!(result.is_err()); // Expected to fail without actual server
    
    // Test server info parsing
    let test_response = "There are 2 of a max of 20 players online: player1, player2";
    let players = rcon_client.parse_player_list(test_response).unwrap();
    assert_eq!(players.len(), 2);
    assert_eq!(players[0].name, "player1");
    assert_eq!(players[1].name, "player2");
}

#[tokio::test]
async fn test_monitoring_manager() {
    let config = hostd::core::config::MonitoringConfig {
        enabled: true,
        metrics_interval: Duration::from_secs(10),
        health_check_interval: Duration::from_secs(30),
        alert_retention_days: 30,
    };
    
    let monitoring_manager = MonitoringManager::new(&config).unwrap();
    
    // Test health status
    let health_status = monitoring_manager.get_health_status().await;
    assert_eq!(health_status.status, "healthy");
    
    // Test system metrics
    let system_metrics = monitoring_manager.get_system_metrics().await;
    assert!(system_metrics.cpu_usage >= 0.0);
    assert!(system_metrics.memory_usage >= 0.0);
    
    // Test alert creation
    let alert_id = monitoring_manager.create_alert(
        None,
        hostd::core::monitoring::AlertLevel::Warning,
        "Test Alert".to_string(),
        "This is a test alert".to_string(),
    ).await.unwrap();
    
    let alerts = monitoring_manager.get_alerts().await;
    assert!(!alerts.is_empty());
    assert!(alerts.iter().any(|a| a.id == alert_id));
}

#[tokio::test]
async fn test_database_manager() {
    let db_manager = DatabaseManager::new(":memory:").await.unwrap();
    
    // Test database initialization
    db_manager.initialize().await.unwrap();
    
    // Test server config creation
    let server_config = hostd::database::ServerConfig {
        id: "test-server".to_string(),
        name: "Test Server".to_string(),
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
    
    let result = db_manager.create_server_config(server_config).await;
    assert!(result.is_ok());
    
    // Test server config retrieval
    let configs = db_manager.get_all_server_configs().await.unwrap();
    assert!(!configs.is_empty());
    assert_eq!(configs[0].name, "Test Server");
}

#[tokio::test]
fn test_error_handling() {
    // Test error creation
    let error = AppError::validation_error("username", "test", "required", "Username is required");
    assert_eq!(error.status_code(), axum::http::StatusCode::BAD_REQUEST);
    assert!(error.user_message().contains("Username is required"));
    assert_eq!(error.category(), "validation");
    assert!(!error.is_retryable());
    
    // Test database error
    let db_error = AppError::database_error("create", "Connection failed");
    assert_eq!(db_error.status_code(), axum::http::StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(db_error.category(), "database");
    assert!(db_error.is_retryable());
    assert_eq!(db_error.retry_delay_ms(), 1000);
    
    // Test authentication error
    let auth_error = AppError::authentication_error(
        hostd::core::error_handler::AuthErrorReason::InvalidCredentials,
        "Invalid username or password"
    );
    assert_eq!(auth_error.status_code(), axum::http::StatusCode::UNAUTHORIZED);
    assert_eq!(auth_error.category(), "authentication");
    assert!(!auth_error.is_retryable());
}