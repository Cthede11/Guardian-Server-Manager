use anyhow::Result;
use hostd::{
    database::DatabaseManager,
    minecraft::{MinecraftManager, ServerConfig},
    websocket::WebSocketManager,
    api::AppState,
};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

#[tokio::test]
async fn test_database_operations() -> Result<()> {
    // Create a temporary database
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let database_url = format!("sqlite:{}", db_path.display());
    
    let db = DatabaseManager::new(&database_url).await?;
    
    // Test server configuration
    let server_config = ServerConfig {
        id: "test-server".to_string(),
        name: "Test Server".to_string(),
        host: "/tmp/test-server".to_string(),
        port: 25565,
        rcon_port: 25575,
        rcon_password: "test-password".to_string(),
        java_path: "/usr/bin/java".to_string(),
        server_jar: "server.jar".to_string(),
        jvm_args: "-Xmx4G".to_string(),
        server_args: "nogui".to_string(),
        auto_start: true,
        auto_restart: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    // Create server
    db.create_server(&server_config).await?;
    
    // Retrieve server
    let retrieved = db.get_server("test-server").await?;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "Test Server");
    
    // Get all servers
    let servers = db.get_all_servers().await?;
    assert_eq!(servers.len(), 1);
    
    // Update server
    let mut updated_config = server_config.clone();
    updated_config.name = "Updated Test Server".to_string();
    updated_config.updated_at = chrono::Utc::now();
    db.update_server(&updated_config).await?;
    
    // Verify update
    let updated = db.get_server("test-server").await?;
    assert_eq!(updated.unwrap().name, "Updated Test Server");
    
    // Delete server
    db.delete_server("test-server").await?;
    
    // Verify deletion
    let deleted = db.get_server("test-server").await?;
    assert!(deleted.is_none());
    
    Ok(())
}

#[tokio::test]
async fn test_websocket_communication() -> Result<()> {
    let manager = WebSocketManager::new();
    
    // Test adding connections
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    
    manager.add_connection(id1, Some("server-1".to_string())).await;
    manager.add_connection(id2, Some("server-2".to_string())).await;
    
    assert_eq!(manager.connection_count().await, 2);
    
    // Test getting server connections
    let server1_connections = manager.get_server_connections("server-1").await;
    assert_eq!(server1_connections.len(), 1);
    assert_eq!(server1_connections[0], id1);
    
    // Test broadcasting
    let message = hostd::websocket::WebSocketMessage::HealthCheck {
        status: "test".to_string(),
        timestamp: chrono::Utc::now(),
    };
    
    let result = manager.broadcast(message).await;
    assert!(result.is_ok());
    
    // Test server-specific broadcasting
    let server_message = hostd::websocket::WebSocketMessage::ServerStatus {
        server_id: "server-1".to_string(),
        status: "running".to_string(),
        timestamp: chrono::Utc::now(),
    };
    
    manager.broadcast_to_server("server-1", server_message).await;
    
    // Test removing connections
    manager.remove_connection(id1).await;
    assert_eq!(manager.connection_count().await, 1);
    
    Ok(())
}

#[tokio::test]
async fn test_minecraft_manager() -> Result<()> {
    // Create temporary database
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let database_url = format!("sqlite:{}", db_path.display());
    
    let db = DatabaseManager::new(&database_url).await?;
    let manager = MinecraftManager::new(db);
    
    // Test initial state
    let servers = manager.get_all_servers().await;
    assert_eq!(servers.len(), 0);
    
    // Add a server
    let server_config = ServerConfig {
        id: "test-server".to_string(),
        name: "Test Server".to_string(),
        host: "/tmp/test-server".to_string(),
        port: 25565,
        rcon_port: 25575,
        rcon_password: "test-password".to_string(),
        java_path: "/usr/bin/java".to_string(),
        server_jar: "server.jar".to_string(),
        jvm_args: "-Xmx4G".to_string(),
        server_args: "nogui".to_string(),
        auto_start: true,
        auto_restart: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    manager.add_server(server_config).await?;
    
    // Verify server was added
    let servers = manager.get_all_servers().await;
    assert_eq!(servers.len(), 1);
    
    // Get specific server
    let server = manager.get_server("test-server").await;
    assert!(server.is_some());
    assert_eq!(server.unwrap().id, "test-server");
    
    Ok(())
}

#[tokio::test]
async fn test_api_state() -> Result<()> {
    let websocket_manager = WebSocketManager::new();
    let app_state = AppState {
        websocket_manager: websocket_manager.clone(),
    };
    
    // Test that the state contains the websocket manager
    assert_eq!(app_state.websocket_manager.connection_count().await, 0);
    
    Ok(())
}

#[tokio::test]
async fn test_server_lifecycle() -> Result<()> {
    // Create temporary database
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let database_url = format!("sqlite:{}", db_path.display());
    
    let db = DatabaseManager::new(&database_url).await?;
    let manager = MinecraftManager::new(db);
    
    // Add a server
    let server_config = ServerConfig {
        id: "lifecycle-test".to_string(),
        name: "Lifecycle Test Server".to_string(),
        host: "/tmp/lifecycle-test".to_string(),
        port: 25565,
        rcon_port: 25575,
        rcon_password: "test-password".to_string(),
        java_path: "/usr/bin/java".to_string(),
        server_jar: "server.jar".to_string(),
        jvm_args: "-Xmx4G".to_string(),
        server_args: "nogui".to_string(),
        auto_start: false,
        auto_restart: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    manager.add_server(server_config).await?;
    
    // Test server operations (these will fail without a real server, but we can test the API)
    let start_result = manager.start_server("lifecycle-test").await;
    // This will fail because there's no real server, but we can test the error handling
    assert!(start_result.is_err());
    
    let stop_result = manager.stop_server("lifecycle-test").await;
    assert!(stop_result.is_err());
    
    let restart_result = manager.restart_server("lifecycle-test").await;
    assert!(restart_result.is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_operations() -> Result<()> {
    let manager = WebSocketManager::new();
    
    // Test concurrent connection additions
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let manager = manager.clone();
            tokio::spawn(async move {
                let id = Uuid::new_v4();
                manager.add_connection(id, Some(format!("server-{}", i))).await;
                id
            })
        })
        .collect();
    
    let results = futures::future::join_all(handles).await;
    assert_eq!(results.len(), 10);
    
    // Verify all connections were added
    assert_eq!(manager.connection_count().await, 10);
    
    // Test concurrent broadcasts
    let broadcast_handles: Vec<_> = (0..5)
        .map(|i| {
            let manager = manager.clone();
            tokio::spawn(async move {
                let message = hostd::websocket::WebSocketMessage::HealthCheck {
                    status: format!("test-{}", i),
                    timestamp: chrono::Utc::now(),
                };
                manager.broadcast(message).await
            })
        })
        .collect();
    
    let broadcast_results = futures::future::join_all(broadcast_handles).await;
    for result in broadcast_results {
        assert!(result.is_ok());
    }
    
    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<()> {
    // Test database with invalid path
    let result = DatabaseManager::new("invalid://path").await;
    assert!(result.is_err());
    
    // Test WebSocket manager with invalid operations
    let manager = WebSocketManager::new();
    
    // Try to get connections for non-existent server
    let connections = manager.get_server_connections("non-existent").await;
    assert_eq!(connections.len(), 0);
    
    // Try to remove non-existent connection
    manager.remove_connection(Uuid::new_v4()).await;
    // This should not panic
    
    Ok(())
}

#[tokio::test]
async fn test_performance() -> Result<()> {
    let manager = WebSocketManager::new();
    
    // Test adding many connections
    let start = std::time::Instant::now();
    
    for i in 0..1000 {
        let id = Uuid::new_v4();
        manager.add_connection(id, Some(format!("server-{}", i % 10))).await;
    }
    
    let duration = start.elapsed();
    println!("Added 1000 connections in {:?}", duration);
    
    // Should complete in reasonable time (less than 1 second)
    assert!(duration < Duration::from_secs(1));
    
    // Test broadcasting to many connections
    let start = std::time::Instant::now();
    
    let message = hostd::websocket::WebSocketMessage::HealthCheck {
        status: "performance-test".to_string(),
        timestamp: chrono::Utc::now(),
    };
    
    let result = manager.broadcast(message).await;
    assert!(result.is_ok());
    
    let duration = start.elapsed();
    println!("Broadcasted to 1000 connections in {:?}", duration);
    
    // Should complete in reasonable time
    assert!(duration < Duration::from_secs(1));
    
    Ok(())
}

#[tokio::test]
async fn test_cleanup() -> Result<()> {
    let manager = WebSocketManager::new();
    
    // Add some connections
    let ids: Vec<_> = (0..5)
        .map(|i| {
            let id = Uuid::new_v4();
            manager.add_connection(id, Some(format!("server-{}", i))).await;
            id
        })
        .collect();
    
    assert_eq!(manager.connection_count().await, 5);
    
    // Remove all connections
    for id in ids {
        manager.remove_connection(id).await;
    }
    
    assert_eq!(manager.connection_count().await, 0);
    
    Ok(())
}