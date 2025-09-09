use hostd::{
    database::{Database, ServerConfig, Task, Mod},
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
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::test]
async fn test_database_performance() {
    let db = Database::new(":memory:").await.unwrap();
    db.run_migrations().await.unwrap();
    
    // Test bulk server creation
    let start = Instant::now();
    let mut servers = Vec::new();
    
    for i in 0..1000 {
        let server_config = ServerConfig {
            id: format!("perf-server-{}", i),
            name: format!("Performance Test Server {}", i),
            mc_version: "1.20.1".to_string(),
            loader: "fabric".to_string(),
            java_path: "java".to_string(),
            min_ram_mb: 1024,
            max_ram_mb: 2048,
            jvm_args: "-Xmx2G".to_string(),
            server_args: "nogui".to_string(),
            world_name: "world".to_string(),
            server_dir: format!("perf-server-{}", i),
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
        servers.push(server);
    }
    
    let duration = start.elapsed();
    println!("Created 1000 servers in {:?}", duration);
    assert!(duration < Duration::from_secs(10), "Server creation too slow");
    
    // Test bulk server retrieval
    let start = Instant::now();
    let all_servers = db.get_all_servers().await.unwrap();
    let duration = start.elapsed();
    println!("Retrieved 1000 servers in {:?}", duration);
    assert_eq!(all_servers.len(), 1000);
    assert!(duration < Duration::from_millis(100), "Server retrieval too slow");
    
    // Test concurrent server updates
    let start = Instant::now();
    let mut handles = vec![];
    
    for server in &servers[0..100] {
        let db_clone = db.clone();
        let server_id = server.id.clone();
        let handle = tokio::spawn(async move {
            db_clone.update_server(&server_id, hostd::api::UpdateServerRequest {
                name: Some("Updated Server".to_string()),
                ..Default::default()
            }).await
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap().unwrap();
    }
    
    let duration = start.elapsed();
    println!("Updated 100 servers concurrently in {:?}", duration);
    assert!(duration < Duration::from_secs(5), "Concurrent updates too slow");
}

#[tokio::test]
async fn test_task_performance() {
    let db = Database::new(":memory:").await.unwrap();
    db.run_migrations().await.unwrap();
    
    // Test bulk task creation
    let start = Instant::now();
    let mut tasks = Vec::new();
    
    for i in 0..5000 {
        let task = Task {
            id: format!("perf-task-{}", i),
            server_id: Some("test-server".to_string()),
            kind: "worldgen".to_string(),
            status: "pending".to_string(),
            progress: 0.0,
            log: None,
            metadata: Some(json!({
                "radius": 5000,
                "dimensions": ["overworld"],
                "chunk_count": 10000
            })),
            started_at: None,
            finished_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        let created = db.create_task(task).await.unwrap();
        tasks.push(created);
    }
    
    let duration = start.elapsed();
    println!("Created 5000 tasks in {:?}", duration);
    assert!(duration < Duration::from_secs(15), "Task creation too slow");
    
    // Test task progress updates
    let start = Instant::now();
    let mut handles = vec![];
    
    for (i, task) in tasks.iter().enumerate().take(1000) {
        let db_clone = db.clone();
        let task_id = task.id.clone();
        let progress = (i as f64) / 1000.0;
        let handle = tokio::spawn(async move {
            db_clone.update_task(&task_id, "running", progress, Some("Processing...".to_string())).await
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap().unwrap();
    }
    
    let duration = start.elapsed();
    println!("Updated 1000 tasks concurrently in {:?}", duration);
    assert!(duration < Duration::from_secs(5), "Task updates too slow");
}

#[tokio::test]
async fn test_mod_performance() {
    let db = Database::new(":memory:").await.unwrap();
    db.run_migrations().await.unwrap();
    
    // Test bulk mod creation
    let start = Instant::now();
    let mut mods = Vec::new();
    
    for i in 0..2000 {
        let mod_info = Mod {
            id: format!("perf-mod-{}", i),
            provider: "curseforge".to_string(),
            project_id: format!("{}", 100000 + i),
            version_id: "latest".to_string(),
            filename: format!("mod-{}.jar", i),
            sha1: format!("sha1hash{}", i),
            server_id: Some("test-server".to_string()),
            enabled: i % 2 == 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        let created = db.create_mod(mod_info).await.unwrap();
        mods.push(created);
    }
    
    let duration = start.elapsed();
    println!("Created 2000 mods in {:?}", duration);
    assert!(duration < Duration::from_secs(10), "Mod creation too slow");
    
    // Test mod search performance
    let start = Instant::now();
    let server_mods = db.get_mods_by_server(Some("test-server")).await.unwrap();
    let duration = start.elapsed();
    println!("Retrieved 2000 mods in {:?}", duration);
    assert_eq!(server_mods.len(), 2000);
    assert!(duration < Duration::from_millis(50), "Mod retrieval too slow");
}

#[tokio::test]
async fn test_websocket_performance() {
    let ws_manager = Arc::new(WebSocketManager::new());
    
    // Test high-frequency message sending
    let start = Instant::now();
    let mut handles = vec![];
    
    for i in 0..1000 {
        let ws_clone = ws_manager.clone();
        let handle = tokio::spawn(async move {
            for j in 0..100 {
                ws_clone.send_console("test-server", vec![
                    hostd::websocket::ConsoleMessage {
                        timestamp: chrono::Utc::now(),
                        level: "info".to_string(),
                        message: format!("Test message {} {}", i, j),
                    }
                ]).await;
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    let duration = start.elapsed();
    println!("Sent 100,000 WebSocket messages in {:?}", duration);
    assert!(duration < Duration::from_secs(30), "WebSocket performance too slow");
}

#[tokio::test]
async fn test_pregeneration_performance() {
    let db = Database::new(":memory:").await.unwrap();
    db.run_migrations().await.unwrap();
    
    let pregen_manager = PregenerationManager::new(db.clone());
    
    // Test job creation performance
    let start = Instant::now();
    let mut jobs = Vec::new();
    
    for i in 0..100 {
        let job_request = hostd::api::CreatePregenerationJobRequest {
            radius: 1000 + (i * 100),
            dimensions: vec!["overworld".to_string()],
            use_gpu: i % 2 == 0,
            lighting_optimization: i % 3 == 0,
        };
        
        let job = pregen_manager.create_job(&format!("server-{}", i), job_request).await.unwrap();
        jobs.push(job);
    }
    
    let duration = start.elapsed();
    println!("Created 100 pregeneration jobs in {:?}", duration);
    assert!(duration < Duration::from_secs(5), "Job creation too slow");
    
    // Test concurrent job operations
    let start = Instant::now();
    let mut handles = vec![];
    
    for job in &jobs[0..50] {
        let pregen_clone = pregen_manager.clone();
        let job_id = job.id.clone();
        let handle = tokio::spawn(async move {
            pregen_clone.start_job(&job_id).await
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap().unwrap();
    }
    
    let duration = start.elapsed();
    println!("Started 50 jobs concurrently in {:?}", duration);
    assert!(duration < Duration::from_secs(10), "Concurrent job operations too slow");
}

#[tokio::test]
async fn test_hot_import_performance() {
    let db = Database::new(":memory:").await.unwrap();
    db.run_migrations().await.unwrap();
    
    let import_manager = HotImportManager::new(db.clone());
    
    // Test job creation performance
    let start = Instant::now();
    let mut jobs = Vec::new();
    
    for i in 0..50 {
        let job_request = hostd::api::CreateHotImportJobRequest {
            source_dir: format!("staging-{}", i),
            target_dir: "world".to_string(),
            safety_checks: true,
            tps_threshold: 18.0,
        };
        
        let job = import_manager.create_job(&format!("server-{}", i), job_request).await.unwrap();
        jobs.push(job);
    }
    
    let duration = start.elapsed();
    println!("Created 50 hot import jobs in {:?}", duration);
    assert!(duration < Duration::from_secs(3), "Import job creation too slow");
}

#[tokio::test]
async fn test_lighting_performance() {
    let db = Database::new(":memory:").await.unwrap();
    db.run_migrations().await.unwrap();
    
    let lighting_manager = LightingManager::new(db.clone());
    
    // Test job creation performance
    let start = Instant::now();
    let mut jobs = Vec::new();
    
    for i in 0..50 {
        let job_request = hostd::api::CreateLightingJobRequest {
            radius: 1000 + (i * 100),
            use_gpu: i % 2 == 0,
            optimization_level: if i % 3 == 0 { "high".to_string() } else { "medium".to_string() },
        };
        
        let job = lighting_manager.create_job(&format!("server-{}", i), job_request).await.unwrap();
        jobs.push(job);
    }
    
    let duration = start.elapsed();
    println!("Created 50 lighting jobs in {:?}", duration);
    assert!(duration < Duration::from_secs(3), "Lighting job creation too slow");
}

#[tokio::test]
async fn test_mod_management_performance() {
    let mod_manager = ModManager::new();
    
    // Test search performance
    let start = Instant::now();
    let mut handles = vec![];
    
    for i in 0..100 {
        let mod_manager_clone = mod_manager.clone();
        let query = format!("mod{}", i);
        let handle = tokio::spawn(async move {
            let search_request = hostd::api::ModSearchRequest {
                query,
                provider: Some("curseforge".to_string()),
                mc_version: Some("1.20.1".to_string()),
                loader: Some("fabric".to_string()),
                limit: Some(10),
            };
            
            mod_manager_clone.search_mods(search_request).await
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap().unwrap();
    }
    
    let duration = start.elapsed();
    println!("Performed 100 mod searches concurrently in {:?}", duration);
    assert!(duration < Duration::from_secs(30), "Mod search performance too slow");
}

#[tokio::test]
async fn test_memory_usage() {
    let db = Database::new(":memory:").await.unwrap();
    db.run_migrations().await.unwrap();
    
    // Create a large number of servers with complex configurations
    let start = Instant::now();
    
    for i in 0..500 {
        let server_config = ServerConfig {
            id: format!("memory-test-server-{}", i),
            name: format!("Memory Test Server {}", i),
            mc_version: "1.20.1".to_string(),
            loader: "fabric".to_string(),
            java_path: "java".to_string(),
            min_ram_mb: 1024,
            max_ram_mb: 2048,
            jvm_args: "-Xmx2G -XX:+UseG1GC".to_string(),
            server_args: "nogui --world-dir world".to_string(),
            world_name: "world".to_string(),
            server_dir: format!("memory-test-server-{}", i),
            status: "stopped".to_string(),
            auto_start: false,
            auto_restart: false,
            max_players: 20,
            pregeneration_policy: json!({
                "radius": 10000,
                "dimensions": ["overworld", "nether", "end"],
                "lighting_optimization": true,
                "gpu_acceleration": true,
                "fallback_cpu": true,
                "chunk_size": 16,
                "batch_size": 1000
            }),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        db.create_server(server_config).await.unwrap();
    }
    
    let duration = start.elapsed();
    println!("Created 500 complex servers in {:?}", duration);
    assert!(duration < Duration::from_secs(20), "Memory test too slow");
    
    // Test retrieval performance with complex data
    let start = Instant::now();
    let all_servers = db.get_all_servers().await.unwrap();
    let duration = start.elapsed();
    println!("Retrieved 500 complex servers in {:?}", duration);
    assert_eq!(all_servers.len(), 500);
    assert!(duration < Duration::from_millis(200), "Complex server retrieval too slow");
}

#[tokio::test]
async fn test_concurrent_operations() {
    let db = Database::new(":memory:").await.unwrap();
    db.run_migrations().await.unwrap();
    
    // Test concurrent operations on the same server
    let server_config = ServerConfig {
        id: "concurrent-test-server".to_string(),
        name: "Concurrent Test Server".to_string(),
        mc_version: "1.20.1".to_string(),
        loader: "fabric".to_string(),
        java_path: "java".to_string(),
        min_ram_mb: 1024,
        max_ram_mb: 2048,
        jvm_args: "-Xmx2G".to_string(),
        server_args: "nogui".to_string(),
        world_name: "world".to_string(),
        server_dir: "concurrent-test-server".to_string(),
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
    
    // Test concurrent task creation
    let start = Instant::now();
    let mut handles = vec![];
    
    for i in 0..100 {
        let db_clone = db.clone();
        let server_id = server.id.clone();
        let handle = tokio::spawn(async move {
            let task = Task {
                id: format!("concurrent-task-{}", i),
                server_id: Some(server_id),
                kind: "worldgen".to_string(),
                status: "pending".to_string(),
                progress: 0.0,
                log: None,
                metadata: Some(json!({
                    "radius": 5000,
                    "dimensions": ["overworld"],
                    "chunk_count": 1000
                })),
                started_at: None,
                finished_at: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            
            db_clone.create_task(task).await
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap().unwrap();
    }
    
    let duration = start.elapsed();
    println!("Created 100 tasks concurrently in {:?}", duration);
    assert!(duration < Duration::from_secs(10), "Concurrent task creation too slow");
    
    // Test concurrent mod creation
    let start = Instant::now();
    let mut handles = vec![];
    
    for i in 0..50 {
        let db_clone = db.clone();
        let server_id = server.id.clone();
        let handle = tokio::spawn(async move {
            let mod_info = Mod {
                id: format!("concurrent-mod-{}", i),
                provider: "curseforge".to_string(),
                project_id: format!("{}", 200000 + i),
                version_id: "latest".to_string(),
                filename: format!("concurrent-mod-{}.jar", i),
                sha1: format!("concurrentsha1{}", i),
                server_id: Some(server_id),
                enabled: i % 2 == 0,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            
            db_clone.create_mod(mod_info).await
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap().unwrap();
    }
    
    let duration = start.elapsed();
    println!("Created 50 mods concurrently in {:?}", duration);
    assert!(duration < Duration::from_secs(5), "Concurrent mod creation too slow");
}

#[tokio::test]
async fn test_stress_test() {
    let db = Database::new(":memory:").await.unwrap();
    db.run_migrations().await.unwrap();
    
    // Stress test with mixed operations
    let start = Instant::now();
    let mut handles = vec![];
    
    // Create servers
    for i in 0..50 {
        let db_clone = db.clone();
        let handle = tokio::spawn(async move {
            let server_config = ServerConfig {
                id: format!("stress-server-{}", i),
                name: format!("Stress Test Server {}", i),
                mc_version: "1.20.1".to_string(),
                loader: "fabric".to_string(),
                java_path: "java".to_string(),
                min_ram_mb: 1024,
                max_ram_mb: 2048,
                jvm_args: "-Xmx2G".to_string(),
                server_args: "nogui".to_string(),
                world_name: "world".to_string(),
                server_dir: format!("stress-server-{}", i),
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
            
            db_clone.create_server(server_config).await
        });
        handles.push(handle);
    }
    
    // Create tasks
    for i in 0..200 {
        let db_clone = db.clone();
        let handle = tokio::spawn(async move {
            let task = Task {
                id: format!("stress-task-{}", i),
                server_id: Some(format!("stress-server-{}", i % 50)),
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
            
            db_clone.create_task(task).await
        });
        handles.push(handle);
    }
    
    // Create mods
    for i in 0..100 {
        let db_clone = db.clone();
        let handle = tokio::spawn(async move {
            let mod_info = Mod {
                id: format!("stress-mod-{}", i),
                provider: "curseforge".to_string(),
                project_id: format!("{}", 300000 + i),
                version_id: "latest".to_string(),
                filename: format!("stress-mod-{}.jar", i),
                sha1: format!("stresssha1{}", i),
                server_id: Some(format!("stress-server-{}", i % 50)),
                enabled: i % 2 == 0,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            
            db_clone.create_mod(mod_info).await
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    for handle in handles {
        handle.await.unwrap().unwrap();
    }
    
    let duration = start.elapsed();
    println!("Stress test completed in {:?}", duration);
    assert!(duration < Duration::from_secs(60), "Stress test too slow");
    
    // Verify data integrity
    let servers = db.get_all_servers().await.unwrap();
    assert_eq!(servers.len(), 50);
    
    let tasks = db.get_tasks_by_server(None).await.unwrap();
    assert_eq!(tasks.len(), 200);
    
    let mods = db.get_mods_by_server(None).await.unwrap();
    assert_eq!(mods.len(), 100);
}
