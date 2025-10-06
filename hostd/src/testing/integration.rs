/// Integration tests for Guardian Server Manager
/// Tests the complete system integration and end-to-end workflows

use anyhow::Result;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::json;
use std::collections::HashMap;
use tempfile::tempdir;
use tokio::time::{sleep, Duration};
use tower::ServiceExt;

use crate::{
    api::create_api_router,
    database::{DatabaseManager, ServerConfig},
    monitoring::MetricsCollector,
    security::{AuthService, EncryptionService},
    websocket_manager::WebSocketManager,
    minecraft::MinecraftManager,
    mod_manager::ModManager,
};
use std::sync::Arc;
use std::path::PathBuf;

/// Integration test suite
pub struct IntegrationTestSuite {
    pub database: DatabaseManager,
    pub auth_service: AuthService,
    pub encryption_service: EncryptionService,
    pub metrics_collector: MetricsCollector,
    pub app: Router,
}

impl IntegrationTestSuite {
    /// Create a new integration test suite
    pub async fn new() -> Result<Self> {
        // Create temporary database
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        let database_url = format!("sqlite:{}", db_path.display());
        
        let database = DatabaseManager::new(&database_url).await?;
        
        // Create services
        let auth_service = AuthService::new("test-secret-key".to_string());
        let encryption_service = EncryptionService::new([0u8; 32]);
        let metrics_collector = MetricsCollector::new()?;
        
        // Create API router
        let app_state = crate::api::AppState {
            websocket_manager: Arc::new(crate::websocket_manager::WebSocketManager::new()),
            minecraft_manager: crate::minecraft::MinecraftManager::new(database.clone()),
            database: database.clone(),
            mod_manager: crate::mod_manager::ModManager::new(std::path::PathBuf::from("mods")),
        };
        
        let app = create_api_router(app_state);
        
        Ok(Self {
            database,
            auth_service,
            encryption_service,
            metrics_collector,
            app,
        })
    }
    
    /// Test complete server creation workflow
    pub async fn test_server_creation_workflow(&self) -> Result<()> {
        println!("Testing server creation workflow...");
        
        // 1. Create server via API
        let server_data = json!({
            "name": "Integration Test Server",
            "loader": "vanilla",
            "version": "1.21.1",
            "minecraft_version": "1.21.1",
            "paths": {
                "world": "./world",
                "mods": "./mods",
                "config": "./config"
            },
            "max_players": 20,
            "memory": 4096
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/api/servers")
            .header("Content-Type", "application/json")
            .body(Body::from(server_data.to_string()))?;
        
        let response = self.app.clone().oneshot(request).await?;
        assert_eq!(response.status(), StatusCode::CREATED);
        
        // 2. Verify server was created in database
        let servers = self.database.get_all_servers().await?;
        assert!(!servers.is_empty());
        
        let server = servers.iter().find(|s| s.name == "Integration Test Server").unwrap();
        assert_eq!(server.minecraft_version, "1.21.1");
        assert_eq!(server.loader, "vanilla");
        assert_eq!(server.max_players, 20);
        
        // 3. Test server retrieval
        let request = Request::builder()
            .method("GET")
            .uri("/api/servers")
            .body(Body::empty())?;
        
        let response = self.app.clone().oneshot(request).await?;
        assert_eq!(response.status(), StatusCode::OK);
        
        // 4. Test server update
        let update_data = json!({
            "name": "Updated Integration Test Server",
            "max_players": 30
        });
        
        let request = Request::builder()
            .method("PUT")
            .uri(&format!("/api/servers/{}", server.id))
            .header("Content-Type", "application/json")
            .body(Body::from(update_data.to_string()))?;
        
        let response = self.app.clone().oneshot(request).await?;
        assert_eq!(response.status(), StatusCode::OK);
        
        // 5. Verify update in database
        let updated_server = self.database.get_server(&server.id).await?;
        assert!(updated_server.is_some());
        let updated_server = updated_server.unwrap();
        assert_eq!(updated_server.name, "Updated Integration Test Server");
        assert_eq!(updated_server.max_players, 30);
        
        // 6. Test server deletion
        let request = Request::builder()
            .method("DELETE")
            .uri(&format!("/api/servers/{}", server.id))
            .body(Body::empty())?;
        
        let response = self.app.clone().oneshot(request).await?;
        assert_eq!(response.status(), StatusCode::OK);
        
        // 7. Verify server was deleted
        let deleted_server = self.database.get_server(&server.id).await?;
        assert!(deleted_server.is_none());
        
        println!("✅ Server creation workflow test passed");
        Ok(())
    }
    
    /// Test authentication workflow
    pub async fn test_authentication_workflow(&self) -> Result<()> {
        println!("Testing authentication workflow...");
        
        // 1. Test login with valid credentials
        let login_data = json!({
            "username": "admin",
            "password": "admin123"
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/api/auth/login")
            .header("Content-Type", "application/json")
            .body(Body::from(login_data.to_string()))?;
        
        let response = self.app.clone().oneshot(request).await?;
        assert_eq!(response.status(), StatusCode::OK);
        
        // 2. Test login with invalid credentials
        let invalid_login_data = json!({
            "username": "admin",
            "password": "wrong_password"
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/api/auth/login")
            .header("Content-Type", "application/json")
            .body(Body::from(invalid_login_data.to_string()))?;
        
        let response = self.app.clone().oneshot(request).await?;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        
        println!("✅ Authentication workflow test passed");
        Ok(())
    }
    
    /// Test database operations
    pub async fn test_database_operations(&self) -> Result<()> {
        println!("Testing database operations...");
        
        // 1. Test server creation
        let server_config = ServerConfig {
            id: "test-server-1".to_string(),
            name: "Test Server 1".to_string(),
            minecraft_version: "1.21.1".to_string(),
            loader: "vanilla".to_string(),
            loader_version: "1.21.1".to_string(),
            port: 25565,
            rcon_port: 25575,
            query_port: 25566,
            max_players: 20,
            memory: 4096,
            java_args: "[]".to_string(),
            server_args: "[]".to_string(),
            auto_start: true,
            auto_restart: true,
            world_name: "world".to_string(),
            difficulty: "normal".to_string(),
            gamemode: "survival".to_string(),
            pvp: true,
            online_mode: true,
            whitelist: false,
            enable_command_block: false,
            view_distance: 10,
            simulation_distance: 10,
            motd: "A Test Server".to_string(),
            host: "localhost".to_string(),
            java_path: "java".to_string(),
            jvm_args: "-Xmx4G -Xms2G".to_string(),
            server_jar: "server.jar".to_string(),
            rcon_password: "test_password".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        self.database.create_server(&server_config).await?;
        
        // 2. Test server retrieval
        let retrieved_server = self.database.get_server("test-server-1").await?;
        assert!(retrieved_server.is_some());
        let retrieved_server = retrieved_server.unwrap();
        assert_eq!(retrieved_server.name, "Test Server 1");
        
        // 3. Test server update
        let mut updated_config = server_config.clone();
        updated_config.name = "Updated Test Server".to_string();
        updated_config.max_players = 30;
        updated_config.updated_at = chrono::Utc::now();
        
        self.database.update_server(&updated_config).await?;
        
        let updated_server = self.database.get_server("test-server-1").await?;
        assert!(updated_server.is_some());
        let updated_server = updated_server.unwrap();
        assert_eq!(updated_server.name, "Updated Test Server");
        assert_eq!(updated_server.max_players, 30);
        
        // 4. Test server deletion
        self.database.delete_server("test-server-1").await?;
        
        let deleted_server = self.database.get_server("test-server-1").await?;
        assert!(deleted_server.is_none());
        
        println!("✅ Database operations test passed");
        Ok(())
    }
    
    /// Test metrics collection
    pub async fn test_metrics_collection(&self) -> Result<()> {
        println!("Testing metrics collection...");
        
        // 1. Test HTTP metrics
        self.metrics_collector.record_http_request("GET", "/api/servers", 200, Duration::from_millis(100));
        self.metrics_collector.record_http_request("POST", "/api/servers", 201, Duration::from_millis(200));
        
        // 2. Test server metrics
        self.metrics_collector.update_server_counts(5, 3, 2);
        
        // 3. Test system metrics
        let system_metrics = crate::monitoring::SystemMetrics {
            memory_usage_bytes: 1024 * 1024 * 1024, // 1GB
            cpu_usage_percent: 50.0,
            disk_usage_bytes: 10 * 1024 * 1024 * 1024, // 10GB
            uptime_seconds: 3600,
        };
        self.metrics_collector.update_system_metrics(&system_metrics);
        
        // 4. Test business metrics
        self.metrics_collector.update_business_metrics(100, 2.5);
        
        // 5. Export metrics
        let metrics_export = self.metrics_collector.export_metrics()?;
        assert!(metrics_export.contains("http_requests_total"));
        assert!(metrics_export.contains("servers_total"));
        assert!(metrics_export.contains("system_memory_usage_bytes"));
        assert!(metrics_export.contains("active_users"));
        
        // 6. Get metrics summary
        let summary = self.metrics_collector.get_metrics_summary().await;
        assert_eq!(summary.servers_total, 5.0);
        assert_eq!(summary.servers_running, 3.0);
        assert_eq!(summary.servers_stopped, 2.0);
        assert_eq!(summary.active_users, 100.0);
        assert_eq!(summary.error_rate, 2.5);
        
        println!("✅ Metrics collection test passed");
        Ok(())
    }
    
    /// Test error handling
    pub async fn test_error_handling(&self) -> Result<()> {
        println!("Testing error handling...");
        
        // 1. Test invalid server creation
        let invalid_server_data = json!({
            "name": "", // Invalid empty name
            "loader": "invalid_loader", // Invalid loader
            "version": "invalid_version" // Invalid version
        });
        
        let request = Request::builder()
            .method("POST")
            .uri("/api/servers")
            .header("Content-Type", "application/json")
            .body(Body::from(invalid_server_data.to_string()))?;
        
        let response = self.app.clone().oneshot(request).await?;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        
        // 2. Test non-existent server retrieval
        let request = Request::builder()
            .method("GET")
            .uri("/api/servers/non-existent-server")
            .body(Body::empty())?;
        
        let response = self.app.clone().oneshot(request).await?;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        
        // 3. Test invalid endpoint
        let request = Request::builder()
            .method("GET")
            .uri("/api/invalid-endpoint")
            .body(Body::empty())?;
        
        let response = self.app.clone().oneshot(request).await?;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        
        println!("✅ Error handling test passed");
        Ok(())
    }
    
    /// Test concurrent operations
    pub async fn test_concurrent_operations(&self) -> Result<()> {
        println!("Testing concurrent operations...");
        
        let mut handles = Vec::new();
        
        // Create multiple servers concurrently
        for i in 0..10 {
            let app = self.app.clone();
            let handle = tokio::spawn(async move {
                let server_data = json!({
                    "name": format!("Concurrent Test Server {}", i),
                    "loader": "vanilla",
                    "version": "1.21.1",
                    "minecraft_version": "1.21.1",
                    "paths": {
                        "world": "./world",
                        "mods": "./mods",
                        "config": "./config"
                    },
                    "max_players": 20,
                    "memory": 4096
                });
                
                let request = Request::builder()
                    .method("POST")
                    .uri("/api/servers")
                    .header("Content-Type", "application/json")
                    .body(Body::from(server_data.to_string()))
                    .unwrap();
                
                let response = app.oneshot(request).await.unwrap();
                response.status()
            });
            handles.push(handle);
        }
        
        // Wait for all operations to complete
        let results = futures::future::join_all(handles).await;
        
        // Verify all operations succeeded
        for result in results {
            let status = result.unwrap();
            assert_eq!(status, StatusCode::CREATED);
        }
        
        // Verify servers were created
        let servers = self.database.get_all_servers().await?;
        assert!(servers.len() >= 10);
        
        println!("✅ Concurrent operations test passed");
        Ok(())
    }
    
    /// Test performance under load
    pub async fn test_performance_under_load(&self) -> Result<()> {
        println!("Testing performance under load...");
        
        let start_time = std::time::Instant::now();
        let mut handles = Vec::new();
        
        // Create 100 servers concurrently
        for i in 0..100 {
            let app = self.app.clone();
            let handle = tokio::spawn(async move {
                let server_data = json!({
                    "name": format!("Load Test Server {}", i),
                    "loader": "vanilla",
                    "version": "1.21.1",
                    "minecraft_version": "1.21.1",
                    "paths": {
                        "world": "./world",
                        "mods": "./mods",
                        "config": "./config"
                    },
                    "max_players": 20,
                    "memory": 4096
                });
                
                let request = Request::builder()
                    .method("POST")
                    .uri("/api/servers")
                    .header("Content-Type", "application/json")
                    .body(Body::from(server_data.to_string()))
                    .unwrap();
                
                let response = app.oneshot(request).await.unwrap();
                response.status()
            });
            handles.push(handle);
        }
        
        // Wait for all operations to complete
        let results = futures::future::join_all(handles).await;
        let duration = start_time.elapsed();
        
        // Verify all operations succeeded
        let mut success_count = 0;
        for result in results {
            let status = result.unwrap();
            if status == StatusCode::CREATED {
                success_count += 1;
            }
        }
        
        assert_eq!(success_count, 100);
        
        // Performance should be reasonable (less than 10 seconds for 100 operations)
        assert!(duration.as_secs() < 10);
        
        println!("✅ Performance under load test passed ({}ms for 100 operations)", duration.as_millis());
        Ok(())
    }
    
    /// Run all integration tests
    pub async fn run_all_tests(&self) -> Result<()> {
        println!("🚀 Starting integration test suite...");
        
        self.test_database_operations().await?;
        self.test_metrics_collection().await?;
        self.test_server_creation_workflow().await?;
        self.test_authentication_workflow().await?;
        self.test_error_handling().await?;
        self.test_concurrent_operations().await?;
        self.test_performance_under_load().await?;
        
        println!("🎉 All integration tests passed!");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_integration_suite_creation() {
        let suite = IntegrationTestSuite::new().await.unwrap();
        assert!(suite.database.get_all_servers().await.unwrap().is_empty());
    }
    
    #[tokio::test]
    async fn test_database_operations() {
        let suite = IntegrationTestSuite::new().await.unwrap();
        suite.test_database_operations().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_metrics_collection() {
        let suite = IntegrationTestSuite::new().await.unwrap();
        suite.test_metrics_collection().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_server_creation_workflow() {
        let suite = IntegrationTestSuite::new().await.unwrap();
        suite.test_server_creation_workflow().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_authentication_workflow() {
        let suite = IntegrationTestSuite::new().await.unwrap();
        suite.test_authentication_workflow().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_error_handling() {
        let suite = IntegrationTestSuite::new().await.unwrap();
        suite.test_error_handling().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_concurrent_operations() {
        let suite = IntegrationTestSuite::new().await.unwrap();
        suite.test_concurrent_operations().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_performance_under_load() {
        let suite = IntegrationTestSuite::new().await.unwrap();
        suite.test_performance_under_load().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_full_integration_suite() {
        let suite = IntegrationTestSuite::new().await.unwrap();
        suite.run_all_tests().await.unwrap();
    }
}
