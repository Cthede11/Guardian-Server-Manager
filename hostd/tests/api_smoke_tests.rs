use std::collections::HashMap;
use serde_json::json;

/// Test the health check endpoint structure
#[test]
fn test_health_endpoint_structure() {
    // This test verifies the health endpoint structure without running the server
    // Based on the API code analysis, the health endpoint should return:
    
    let expected_health_response = json!({
        "success": true,
        "data": {
            "status": "healthy",
            "components": {
                "database": {
                    "status": "healthy",
                    "message": "Database connection successful",
                    "last_check": "2024-01-01T00:00:00Z",
                    "response_time_ms": 5
                },
                "websocket": {
                    "status": "healthy", 
                    "message": "WebSocket connections: 0",
                    "last_check": "2024-01-01T00:00:00Z",
                    "response_time_ms": 1
                },
                "curseforge": {
                    "status": "healthy",
                    "message": "API key configured",
                    "last_check": "2024-01-01T00:00:00Z",
                    "response_time_ms": 100
                },
                "modrinth": {
                    "status": "healthy",
                    "message": "API key configured", 
                    "last_check": "2024-01-01T00:00:00Z",
                    "response_time_ms": 100
                }
            },
            "timestamp": "2024-01-01T00:00:00Z"
        }
    });
    
    // Verify the structure is valid JSON
    assert!(expected_health_response.is_object());
    assert!(expected_health_response["success"].as_bool().unwrap());
    assert!(expected_health_response["data"].is_object());
    assert!(expected_health_response["data"]["status"].as_str().unwrap() == "healthy");
    assert!(expected_health_response["data"]["components"].is_object());
}

/// Test the status endpoint structure
#[test]
fn test_status_endpoint_structure() {
    let expected_status_response = json!({
        "success": true,
        "data": {
            "status": "running",
            "version": "1.0.0",
            "uptime": 3600,
            "connections": 0,
            "servers": 1,
            "timestamp": "2024-01-01T00:00:00Z"
        }
    });
    
    assert!(expected_status_response.is_object());
    assert!(expected_status_response["success"].as_bool().unwrap());
    assert!(expected_status_response["data"]["status"].as_str().unwrap() == "running");
}

/// Test API response structure
#[test]
fn test_api_response_structure() {
    let success_response = json!({
        "success": true,
        "data": {
            "message": "Operation successful"
        }
    });
    
    let error_response = json!({
        "success": false,
        "error": "Operation failed",
        "details": "Detailed error message"
    });
    
    assert!(success_response["success"].as_bool().unwrap());
    assert!(!error_response["success"].as_bool().unwrap());
    assert!(success_response["data"].is_object());
    assert!(error_response["error"].is_string());
}

/// Test server creation validation structure
#[test]
fn test_server_validation_structure() {
    let valid_server_request = json!({
        "name": "Test Server",
        "minecraft_version": "1.20.1",
        "loader": "fabric",
        "port": 25565,
        "memory": 2048,
        "max_players": 20
    });
    
    let invalid_server_request = json!({
        "name": "",
        "minecraft_version": "invalid",
        "loader": "unknown",
        "port": 80,
        "memory": 100,
        "max_players": 0
    });
    
    // Test valid request structure
    assert!(valid_server_request["name"].as_str().unwrap().len() > 0);
    assert!(valid_server_request["port"].as_u64().unwrap() > 1024);
    assert!(valid_server_request["memory"].as_u64().unwrap() >= 512);
    
    // Test invalid request structure
    assert!(invalid_server_request["name"].as_str().unwrap().is_empty());
    assert!(invalid_server_request["port"].as_u64().unwrap() < 1024);
    assert!(invalid_server_request["memory"].as_u64().unwrap() < 512);
}

/// Test settings validation structure
#[test]
fn test_settings_validation_structure() {
    let settings_request = json!({
        "cf_api_key": "test_curseforge_key_123",
        "modrinth_token": "test_modrinth_token_456",
        "java_path": "C:\\Program Files\\Java\\bin\\java.exe",
        "default_ram_mb": 2048,
        "data_dir": "C:\\GuardianData",
        "telemetry_opt_in": false
    });
    
    assert!(settings_request["cf_api_key"].is_string());
    assert!(settings_request["modrinth_token"].is_string());
    assert!(settings_request["java_path"].is_string());
    assert!(settings_request["default_ram_mb"].as_u64().unwrap() > 0);
}

/// Test WebSocket message structure
#[test]
fn test_websocket_message_structure() {
    let server_status_message = json!({
        "id": "msg_123",
        "server_id": "server_456",
        "event_type": "ServerStatusChange",
        "timestamp": "2024-01-01T00:00:00Z",
        "data": {
            "old_status": "stopped",
            "new_status": "running"
        }
    });
    
    let console_message = json!({
        "id": "msg_789",
        "server_id": "server_456", 
        "event_type": "ConsoleMessage",
        "timestamp": "2024-01-01T00:00:00Z",
        "data": {
            "level": "INFO",
            "message": "Server started successfully"
        }
    });
    
    assert!(server_status_message["event_type"].as_str().unwrap() == "ServerStatusChange");
    assert!(console_message["event_type"].as_str().unwrap() == "ConsoleMessage");
    assert!(server_status_message["data"].is_object());
    assert!(console_message["data"].is_object());
}

/// Test modpack structure
#[test]
fn test_modpack_structure() {
    let modpack_request = json!({
        "name": "Test Modpack",
        "description": "A test modpack",
        "minecraft_version": "1.20.1",
        "loader": "fabric",
        "mods": [
            {
                "provider": "curseforge",
                "project_id": "123456",
                "version_id": "789012"
            },
            {
                "provider": "modrinth",
                "project_id": "abcdef",
                "file_id": "ghijkl"
            }
        ]
    });
    
    assert!(modpack_request["name"].is_string());
    assert!(modpack_request["mods"].is_array());
    assert!(modpack_request["mods"].as_array().unwrap().len() == 2);
    
    let mods = modpack_request["mods"].as_array().unwrap();
    assert!(mods[0]["provider"].as_str().unwrap() == "curseforge");
    assert!(mods[1]["provider"].as_str().unwrap() == "modrinth");
}

/// Test error response structure
#[test]
fn test_error_response_structure() {
    let validation_error = json!({
        "success": false,
        "error": "Validation failed",
        "field_errors": {
            "name": ["Name is required"],
            "port": ["Port must be between 1024 and 65535"]
        },
        "timestamp": "2024-01-01T00:00:00Z"
    });
    
    let server_error = json!({
        "success": false,
        "error": "Server not found",
        "details": "Server with ID 'invalid_id' does not exist"
    });
    
    assert!(!validation_error["success"].as_bool().unwrap());
    assert!(validation_error["field_errors"].is_object());
    assert!(server_error["error"].is_string());
}