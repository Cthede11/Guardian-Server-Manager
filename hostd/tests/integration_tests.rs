use hostd::{
    api::{
        CreateServerRequest, UpdateServerRequest, UpdateSettingsRequest,
        CreatePregenerationJobRequest, CreateHotImportJobRequest,
        CreateLightingJobRequest, ModSearchRequest, CreateModPlanRequest,
        ApplyFixesRequest
    },
    database::{Database, ServerConfig, Settings, Task, Mod},
    minecraft::MinecraftManager,
    websocket::WebSocketManager,
    compatibility::CompatibilityScanner,
    pregeneration::PregenerationManager,
    hot_import::HotImportManager,
    lighting::LightingManager,
    mod_management::ModManager
};
use serde_json::json;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_database_migrations() {
    let db = Database::new(":memory:").await.unwrap();
    
    // Test that migrations run successfully
    db.run_migrations().await.unwrap();
    
    // Test that tables exist
    let servers = db.get_all_servers().await.unwrap();
    assert_eq!(servers.len(), 0);
    
    let settings = db.get_settings().await.unwrap();
    assert!(settings.is_some());
    
    let tasks = db.get_tasks_by_server(None).await.unwrap();
    assert_eq!(tasks.len(), 0);
    
    let mods = db.get_mods_by_server(None).await.unwrap();
    assert_eq!(mods.len(), 0);
}

#[tokio::test]
async fn test_server_crud_operations() {
    let db = Database::new(":memory:").await.unwrap();
    db.run_migrations().await.unwrap();
    
    let minecraft_manager = Arc::new(MinecraftManager::new(db.clone()));
    
    // Create a server
    let server_config = ServerConfig {
        id: "test-server-1".to_string(),
        name: "Test Server".to_string(),
        mc_version: "1.20.1".to_string(),
        loader: "fabric".to_string(),
        java_path: "java".to_string(),
        min_ram_mb: 1024,
        max_ram_mb: 2048,
        jvm_args: "-Xmx2G".to_string(),
        server_args: "nogui".to_string(),
        world_name: "world".to_string(),
        server_dir: "test-server".to_string(),
        status: "stopped".to_string(),
        auto_start: false,
        auto_restart: false,
        max_players: 10,
        pregeneration_policy: json!({
            "radius": 5000,
            "dimensions": ["overworld"],
            "lighting_optimization": true
        }),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let server = db.create_server(server_config).await.unwrap();
    assert_eq!(server.name, "Test Server");
    assert_eq!(server.max_players, 10);
    
    // Get server
    let retrieved = db.get_server(&server.id).await.unwrap().unwrap();
    assert_eq!(retrieved.id, server.id);
    
    // Update server
    let update_request = UpdateServerRequest {
        name: Some("Updated Test Server".to_string()),
        max_players: Some(20),
        ..Default::default()
    };
    
    let updated = db.update_server(&server.id, update_request).await.unwrap();
    assert_eq!(updated.name, "Updated Test Server");
    assert_eq!(updated.max_players, 20);
    
    // Delete server
    db.delete_server(&server.id).await.unwrap();
    let deleted = db.get_server(&server.id).await.unwrap();
    assert!(deleted.is_none());
}

#[tokio::test]
async fn test_settings_management() {
    let db = Database::new(":memory:").await.unwrap();
    db.run_migrations().await.unwrap();
    
    // Get default settings
    let settings = db.get_settings().await.unwrap().unwrap();
    assert_eq!(settings.java_path, "java");
    assert_eq!(settings.default_ram_mb, 2048);
    
    // Update settings
    let update_request = UpdateSettingsRequest {
        cf_api_key: Some("test-cf-key".to_string()),
        modrinth_token: Some("test-modrinth-token".to_string()),
        java_path: Some("/usr/bin/java".to_string()),
        default_ram_mb: Some(4096),
        data_dir: Some("/opt/guardian".to_string()),
        telemetry_opt_in: Some(true),
    };
    
    let updated = db.update_settings(update_request).await.unwrap();
    assert_eq!(updated.cf_api_key, Some("test-cf-key".to_string()));
    assert_eq!(updated.modrinth_token, Some("test-modrinth-token".to_string()));
    assert_eq!(updated.java_path, "/usr/bin/java");
    assert_eq!(updated.default_ram_mb, 4096);
    assert_eq!(updated.telemetry_opt_in, true);
}

#[tokio::test]
async fn test_task_management() {
    let db = Database::new(":memory:").await.unwrap();
    db.run_migrations().await.unwrap();
    
    // Create a task
    let task = Task {
        id: "test-task-1".to_string(),
        server_id: Some("test-server-1".to_string()),
        kind: "worldgen".to_string(),
        status: "pending".to_string(),
        progress: 0.0,
        log: None,
        metadata: Some(json!({
            "radius": 5000,
            "dimensions": ["overworld"]
        })),
        started_at: None,
        finished_at: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let created = db.create_task(task).await.unwrap();
    assert_eq!(created.kind, "worldgen");
    assert_eq!(created.status, "pending");
    
    // Update task
    let updated = db.update_task(&created.id, "running", 0.5, Some("Processing...".to_string())).await.unwrap();
    assert_eq!(updated.status, "running");
    assert_eq!(updated.progress, 0.5);
    
    // Get tasks by server
    let server_tasks = db.get_tasks_by_server(Some("test-server-1")).await.unwrap();
    assert_eq!(server_tasks.len(), 1);
    
    // Delete task
    db.delete_task(&created.id).await.unwrap();
    let deleted = db.get_task(&created.id).await.unwrap();
    assert!(deleted.is_none());
}

#[tokio::test]
async fn test_mod_management() {
    let db = Database::new(":memory:").await.unwrap();
    db.run_migrations().await.unwrap();
    
    // Create a mod
    let mod_info = Mod {
        id: "test-mod-1".to_string(),
        provider: "curseforge".to_string(),
        project_id: "238222".to_string(),
        version_id: "latest".to_string(),
        filename: "create-fabric-1.20.1-0.5.1.jar".to_string(),
        sha1: "abc123def456".to_string(),
        server_id: Some("test-server-1".to_string()),
        enabled: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let created = db.create_mod(mod_info).await.unwrap();
    assert_eq!(created.provider, "curseforge");
    assert_eq!(created.enabled, true);
    
    // Update mod
    let updated = db.update_mod(&created.id, false).await.unwrap();
    assert_eq!(updated.enabled, false);
    
    // Get mods by server
    let server_mods = db.get_mods_by_server(Some("test-server-1")).await.unwrap();
    assert_eq!(server_mods.len(), 1);
    
    // Delete mod
    db.delete_mod(&created.id).await.unwrap();
    let deleted = db.get_mod(&created.id).await.unwrap();
    assert!(deleted.is_none());
}

#[tokio::test]
async fn test_websocket_communication() {
    let ws_manager = Arc::new(WebSocketManager::new());
    
    // Test sending different message types
    ws_manager.send_console("test-server", vec![]).await;
    ws_manager.send_connected().await;
    ws_manager.send_disconnected().await;
    ws_manager.send_error("Test error".to_string()).await;
    ws_manager.send_pong().await;
    
    // Test task update
    let task_update = hostd::websocket::TaskUpdate {
        id: "test-task".to_string(),
        kind: "worldgen".to_string(),
        status: "running".to_string(),
        progress: 0.5,
        log: Some("Processing...".to_string()),
        metadata: Some(json!({"radius": 5000})),
        started_at: Some(chrono::Utc::now()),
        finished_at: None,
    };
    
    ws_manager.send_task_update(Some("test-server".to_string()), task_update).await;
}

#[tokio::test]
async fn test_compatibility_scanning() {
    let scanner = CompatibilityScanner::new();
    
    // Test scanning a mod directory
    let result = scanner.scan_mods("test-mods").await;
    
    // This will return a mock result since we don't have real mod files
    assert!(result.is_ok());
    let scan_result = result.unwrap();
    assert!(scan_result.conflicts.is_empty());
    assert!(scan_result.suggestions.is_empty());
}

#[tokio::test]
async fn test_pregeneration_management() {
    let db = Database::new(":memory:").await.unwrap();
    db.run_migrations().await.unwrap();
    
    let pregen_manager = PregenerationManager::new(db.clone());
    
    // Test creating a pregeneration job
    let job_request = CreatePregenerationJobRequest {
        radius: 5000,
        dimensions: vec!["overworld".to_string()],
        use_gpu: true,
        lighting_optimization: true,
    };
    
    let job = pregen_manager.create_job("test-server", job_request).await.unwrap();
    assert_eq!(job.radius, 5000);
    assert_eq!(job.status, "pending");
    
    // Test starting the job
    let started = pregen_manager.start_job(&job.id).await.unwrap();
    assert_eq!(started.status, "running");
    
    // Test pausing the job
    let paused = pregen_manager.pause_job(&job.id).await.unwrap();
    assert_eq!(paused.status, "paused");
    
    // Test resuming the job
    let resumed = pregen_manager.resume_job(&job.id).await.unwrap();
    assert_eq!(resumed.status, "running");
    
    // Test canceling the job
    let canceled = pregen_manager.cancel_job(&job.id).await.unwrap();
    assert_eq!(canceled.status, "canceled");
}

#[tokio::test]
async fn test_hot_import_management() {
    let db = Database::new(":memory:").await.unwrap();
    db.run_migrations().await.unwrap();
    
    let import_manager = HotImportManager::new(db.clone());
    
    // Test creating a hot import job
    let job_request = CreateHotImportJobRequest {
        source_dir: "staging".to_string(),
        target_dir: "world".to_string(),
        safety_checks: true,
        tps_threshold: 18.0,
    };
    
    let job = import_manager.create_job("test-server", job_request).await.unwrap();
    assert_eq!(job.source_dir, "staging");
    assert_eq!(job.status, "pending");
    
    // Test starting the job
    let started = import_manager.start_job(&job.id).await.unwrap();
    assert_eq!(started.status, "running");
    
    // Test canceling the job
    let canceled = import_manager.cancel_job(&job.id).await.unwrap();
    assert_eq!(canceled.status, "canceled");
}

#[tokio::test]
async fn test_lighting_optimization() {
    let db = Database::new(":memory:").await.unwrap();
    db.run_migrations().await.unwrap();
    
    let lighting_manager = LightingManager::new(db.clone());
    
    // Test creating a lighting job
    let job_request = CreateLightingJobRequest {
        radius: 5000,
        use_gpu: true,
        optimization_level: "high".to_string(),
    };
    
    let job = lighting_manager.create_job("test-server", job_request).await.unwrap();
    assert_eq!(job.radius, 5000);
    assert_eq!(job.status, "pending");
    
    // Test starting the job
    let started = lighting_manager.start_job(&job.id).await.unwrap();
    assert_eq!(started.status, "running");
    
    // Test canceling the job
    let canceled = lighting_manager.cancel_job(&job.id).await.unwrap();
    assert_eq!(canceled.status, "canceled");
}

#[tokio::test]
async fn test_mod_search_and_management() {
    let mod_manager = ModManager::new();
    
    // Test searching for mods
    let search_request = ModSearchRequest {
        query: "create".to_string(),
        provider: Some("curseforge".to_string()),
        mc_version: Some("1.20.1".to_string()),
        loader: Some("fabric".to_string()),
        limit: Some(10),
    };
    
    let search_result = mod_manager.search_mods(search_request).await.unwrap();
    assert!(!search_result.mods.is_empty());
    
    // Test creating a mod plan
    let plan_request = CreateModPlanRequest {
        mods: vec![
            json!({
                "provider": "curseforge",
                "project_id": "238222",
                "version_id": "latest"
            })
        ],
        server_id: "test-server".to_string(),
    };
    
    let plan = mod_manager.create_plan(plan_request).await.unwrap();
    assert_eq!(plan.mods.len(), 1);
    assert_eq!(plan.status, "pending");
}

#[tokio::test]
async fn test_end_to_end_workflow() {
    let db = Database::new(":memory:").await.unwrap();
    db.run_migrations().await.unwrap();
    
    let minecraft_manager = Arc::new(MinecraftManager::new(db.clone()));
    let ws_manager = Arc::new(WebSocketManager::new());
    minecraft_manager.set_websocket_manager(ws_manager.clone());
    
    // Create a server
    let server_config = ServerConfig {
        id: "e2e-test-server".to_string(),
        name: "E2E Test Server".to_string(),
        mc_version: "1.20.1".to_string(),
        loader: "fabric".to_string(),
        java_path: "java".to_string(),
        min_ram_mb: 1024,
        max_ram_mb: 2048,
        jvm_args: "-Xmx2G".to_string(),
        server_args: "nogui".to_string(),
        world_name: "world".to_string(),
        server_dir: "e2e-test-server".to_string(),
        status: "stopped".to_string(),
        auto_start: false,
        auto_restart: false,
        max_players: 10,
        pregeneration_policy: json!({
            "radius": 1000,
            "dimensions": ["overworld"],
            "lighting_optimization": true
        }),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let server = db.create_server(server_config).await.unwrap();
    
    // Test server lifecycle
    let start_result = minecraft_manager.start_server(&server.id).await;
    assert!(start_result.is_ok());
    
    // Wait a bit for server to start
    sleep(Duration::from_secs(2)).await;
    
    // Test server status
    let servers = minecraft_manager.get_servers().await;
    assert!(!servers.is_empty());
    
    // Test stopping server
    let stop_result = minecraft_manager.stop_server(&server.id).await;
    assert!(stop_result.is_ok());
    
    // Clean up
    db.delete_server(&server.id).await.unwrap();
}

#[tokio::test]
async fn test_concurrent_operations() {
    let db = Database::new(":memory:").await.unwrap();
    db.run_migrations().await.unwrap();
    
    // Test concurrent task creation
    let mut handles = vec![];
    
    for i in 0..10 {
        let db_clone = db.clone();
        let handle = tokio::spawn(async move {
            let task = Task {
                id: format!("concurrent-task-{}", i),
                server_id: Some("test-server".to_string()),
                kind: "worldgen".to_string(),
                status: "pending".to_string(),
                progress: 0.0,
                log: None,
                metadata: None,
                started_at: None,
                finished_at: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            
            db_clone.create_task(task).await
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
    
    // Verify all tasks were created
    let tasks = db.get_tasks_by_server(Some("test-server")).await.unwrap();
    assert_eq!(tasks.len(), 10);
}

#[tokio::test]
async fn test_error_handling() {
    let db = Database::new(":memory:").await.unwrap();
    db.run_migrations().await.unwrap();
    
    // Test getting non-existent server
    let result = db.get_server("non-existent").await.unwrap();
    assert!(result.is_none());
    
    // Test updating non-existent server
    let update_request = UpdateServerRequest {
        name: Some("Updated".to_string()),
        ..Default::default()
    };
    
    let result = db.update_server("non-existent", update_request).await;
    assert!(result.is_err());
    
    // Test deleting non-existent server
    let result = db.delete_server("non-existent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_data_consistency() {
    let db = Database::new(":memory:").await.unwrap();
    db.run_migrations().await.unwrap();
    
    // Create a server with specific configuration
    let server_config = ServerConfig {
        id: "consistency-test".to_string(),
        name: "Consistency Test".to_string(),
        mc_version: "1.20.1".to_string(),
        loader: "fabric".to_string(),
        java_path: "java".to_string(),
        min_ram_mb: 1024,
        max_ram_mb: 2048,
        jvm_args: "-Xmx2G".to_string(),
        server_args: "nogui".to_string(),
        world_name: "world".to_string(),
        server_dir: "consistency-test".to_string(),
        status: "stopped".to_string(),
        auto_start: false,
        auto_restart: false,
        max_players: 15,
        pregeneration_policy: json!({
            "radius": 3000,
            "dimensions": ["overworld", "nether"],
            "lighting_optimization": false
        }),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    let created = db.create_server(server_config).await.unwrap();
    
    // Verify all fields are preserved
    assert_eq!(created.max_players, 15);
    assert_eq!(created.pregeneration_policy["radius"], 3000);
    assert_eq!(created.pregeneration_policy["dimensions"], json!(["overworld", "nether"]));
    assert_eq!(created.pregeneration_policy["lighting_optimization"], false);
    
    // Update and verify consistency
    let update_request = UpdateServerRequest {
        max_players: Some(25),
        pregeneration_policy: Some(json!({
            "radius": 5000,
            "dimensions": ["overworld"],
            "lighting_optimization": true
        })),
        ..Default::default()
    };
    
    let updated = db.update_server(&created.id, update_request).await.unwrap();
    assert_eq!(updated.max_players, 25);
    assert_eq!(updated.pregeneration_policy["radius"], 5000);
    assert_eq!(updated.pregeneration_policy["dimensions"], json!(["overworld"]));
    assert_eq!(updated.pregeneration_policy["lighting_optimization"], true);
}
